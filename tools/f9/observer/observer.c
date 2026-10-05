#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ptrace.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/user.h>
#include <sys/wait.h>
#include <unistd.h>

/*
 * EnvMorph F9 observer-only candidate.
 *
 * Derivado técnicamente de AgentGuard-FastPath, commit
 * 2891a512e265cce07c64fc6a0367bd8a78bba3cb, licencia MIT.
 *
 * Se reutilizan las ideas de PTRACE_TRACEME, seguimiento de fork/vfork/clone,
 * lectura de memoria del tracee y resolución de openat. Se eliminan políticas,
 * bloqueo, PTRACE_SETREGS, sustitución de syscalls, riesgo y enforcement.
 */

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

#define MAX_TRACEES 8192

typedef struct {
    bool active;
    char syscall_name[32];
    char kind[32];
    char path[PATH_MAX];
} pending_event_t;

typedef struct {
    pid_t tid;
    bool used;
    bool entering;
    pending_event_t pending;
} trace_state_t;

static trace_state_t states[MAX_TRACEES];
static FILE *event_out;
static unsigned long long sequence_no;

static void die_errno(const char *msg) {
    perror(msg);
    exit(2);
}

static trace_state_t *state_for(pid_t tid, bool create) {
    for (size_t i = 0; i < MAX_TRACEES; i++) {
        if (states[i].used && states[i].tid == tid) return &states[i];
    }

    if (!create) return NULL;

    for (size_t i = 0; i < MAX_TRACEES; i++) {
        if (!states[i].used) {
            memset(&states[i], 0, sizeof(states[i]));
            states[i].used = true;
            states[i].tid = tid;
            states[i].entering = true;
            return &states[i];
        }
    }
    return NULL;
}

static void remove_state(pid_t tid) {
    trace_state_t *st = state_for(tid, false);
    if (st) memset(st, 0, sizeof(*st));
}

static int read_tracee_string(pid_t pid, unsigned long addr, char *buf, size_t cap) {
    size_t off = 0;

    if (!buf || cap == 0) return -1;

    while (off + 1 < cap) {
        errno = 0;
        long word = ptrace(PTRACE_PEEKDATA, pid, (void *)(addr + off), NULL);
        if (word == -1 && errno != 0) {
            buf[off] = '\0';
            return -1;
        }

        for (size_t j = 0; j < sizeof(long) && off + 1 < cap; j++) {
            char c = ((char *)&word)[j];
            buf[off++] = c;
            if (c == '\0') return 0;
        }
    }

    buf[cap - 1] = '\0';
    return 0;
}

static int proc_link(pid_t pid, const char *suffix, char *buf, size_t cap) {
    char link_path[128];
    snprintf(link_path, sizeof(link_path), "/proc/%d/%s", pid, suffix);

    ssize_t n = readlink(link_path, buf, cap - 1);
    if (n < 0) return -1;

    buf[n] = '\0';
    return 0;
}

static void join_path(const char *base, const char *rel, char *out, size_t cap) {
    const char *safe_base = base ? base : "";
    const char *safe_rel = rel ? rel : "";
    size_t pos = 0;

    if (cap == 0) return;

    if (safe_rel[0] == '/') {
        while (safe_rel[pos] && pos + 1 < cap) {
            out[pos] = safe_rel[pos];
            pos++;
        }
        out[pos] = '\0';
        return;
    }

    while (safe_base[pos] && pos + 1 < cap) {
        out[pos] = safe_base[pos];
        pos++;
    }

    if (pos > 0 && out[pos - 1] != '/' && pos + 1 < cap) {
        out[pos++] = '/';
    }

    size_t i = 0;
    while (safe_rel[i] && pos + 1 < cap) {
        out[pos++] = safe_rel[i++];
    }

    out[pos] = '\0';
}

static void resolve_at_path(pid_t pid, int dirfd, const char *raw, char *out, size_t cap) {
    if (!raw || raw[0] == '/') {
        snprintf(out, cap, "%s", raw ? raw : "");
        return;
    }

    char base[PATH_MAX] = {0};

    if (dirfd == AT_FDCWD) {
        if (proc_link(pid, "cwd", base, sizeof(base)) != 0) {
            snprintf(out, cap, "%s", raw);
            return;
        }
    } else {
        char suffix[64];
        snprintf(suffix, sizeof(suffix), "fd/%d", dirfd);
        if (proc_link(pid, suffix, base, sizeof(base)) != 0) {
            snprintf(out, cap, "%s", raw);
            return;
        }
    }

    join_path(base, raw, out, cap);
}

static pid_t read_ppid(pid_t pid) {
    char path[64];
    char line[256];
    pid_t ppid = -1;

    snprintf(path, sizeof(path), "/proc/%d/status", pid);
    FILE *fp = fopen(path, "r");
    if (!fp) return -1;

    while (fgets(line, sizeof(line), fp)) {
        if (sscanf(line, "PPid:\t%d", &ppid) == 1) break;
    }

    fclose(fp);
    return ppid;
}

static void json_string(FILE *fp, const char *s) {
    fputc('"', fp);

    for (const unsigned char *p = (const unsigned char *)(s ? s : ""); *p; p++) {
        switch (*p) {
            case '\\': fputs("\\\\", fp); break;
            case '"': fputs("\\\"", fp); break;
            case '\n': fputs("\\n", fp); break;
            case '\r': fputs("\\r", fp); break;
            case '\t': fputs("\\t", fp); break;
            default:
                if (*p < 0x20) fprintf(fp, "\\u%04x", *p);
                else fputc(*p, fp);
        }
    }

    fputc('"', fp);
}

static const char *syscall_name(long no) {
#ifdef SYS_open
    if (no == SYS_open) return "open";
#endif
#ifdef SYS_openat
    if (no == SYS_openat) return "openat";
#endif
#ifdef SYS_creat
    if (no == SYS_creat) return "creat";
#endif
#ifdef SYS_execve
    if (no == SYS_execve) return "execve";
#endif
#ifdef SYS_execveat
    if (no == SYS_execveat) return "execveat";
#endif
    return "other";
}

static void set_pending(trace_state_t *st, long no, const char *kind, const char *path) {
    memset(&st->pending, 0, sizeof(st->pending));
    st->pending.active = true;
    snprintf(st->pending.syscall_name, sizeof(st->pending.syscall_name), "%s", syscall_name(no));
    snprintf(st->pending.kind, sizeof(st->pending.kind), "%s", kind);
    snprintf(st->pending.path, sizeof(st->pending.path), "%s", path ? path : "");
}

static void begin_pending(pid_t pid, trace_state_t *st, const struct user_regs_struct *regs) {
    long no = (long)regs->orig_rax;
    char raw[PATH_MAX] = {0};
    char resolved[PATH_MAX] = {0};

    memset(&st->pending, 0, sizeof(st->pending));

#ifdef SYS_open
    if (no == SYS_open) {
        read_tracee_string(pid, regs->rdi, raw, sizeof(raw));
        resolve_at_path(pid, AT_FDCWD, raw, resolved, sizeof(resolved));
        set_pending(st, no, "file_open", resolved);
        return;
    }
#endif

#ifdef SYS_openat
    if (no == SYS_openat) {
        read_tracee_string(pid, regs->rsi, raw, sizeof(raw));
        resolve_at_path(pid, (int)regs->rdi, raw, resolved, sizeof(resolved));
        set_pending(st, no, "file_open", resolved);
        return;
    }
#endif

#ifdef SYS_creat
    if (no == SYS_creat) {
        read_tracee_string(pid, regs->rdi, raw, sizeof(raw));
        resolve_at_path(pid, AT_FDCWD, raw, resolved, sizeof(resolved));
        set_pending(st, no, "file_open", resolved);
        return;
    }
#endif

#ifdef SYS_execve
    if (no == SYS_execve) {
        read_tracee_string(pid, regs->rdi, raw, sizeof(raw));
        resolve_at_path(pid, AT_FDCWD, raw, resolved, sizeof(resolved));
        set_pending(st, no, "exec", resolved);
        return;
    }
#endif

#ifdef SYS_execveat
    if (no == SYS_execveat) {
        read_tracee_string(pid, regs->rsi, raw, sizeof(raw));
        resolve_at_path(pid, (int)regs->rdi, raw, resolved, sizeof(resolved));
        set_pending(st, no, "exec", resolved);
    }
#endif
}

static void emit_pending(pid_t pid, trace_state_t *st, long ret) {
    if (!st->pending.active) return;

    long err = ret < 0 ? -ret : 0;
    sequence_no++;

    fputc('{', event_out);
    fprintf(event_out, "\"seq\":%llu,\"pid\":%d,\"ppid\":%d,\"kind\":",
            sequence_no, pid, read_ppid(pid));
    json_string(event_out, st->pending.kind);
    fputs(",\"syscall\":", event_out);
    json_string(event_out, st->pending.syscall_name);
    fputs(",\"resource\":", event_out);
    json_string(event_out, st->pending.path);
    fprintf(event_out, ",\"return_value\":%ld,\"errno\":%ld,\"result\":", ret, err);
    json_string(event_out, ret < 0 ? "error" : "success");
    fputs("}\n", event_out);
    fflush(event_out);

    memset(&st->pending, 0, sizeof(st->pending));
}

static void handle_syscall_stop(pid_t pid) {
    trace_state_t *st = state_for(pid, true);
    if (!st) {
        fprintf(stderr, "ERROR: demasiados tracees\n");
        exit(2);
    }

    struct user_regs_struct regs;
    if (ptrace(PTRACE_GETREGS, pid, NULL, &regs) != 0) return;

    if (st->entering) begin_pending(pid, st, &regs);
    else emit_pending(pid, st, (long)regs.rax);

    st->entering = !st->entering;
}

static int trace_command(char **cmd, const char *output) {
    event_out = fopen(output, "w");
    if (!event_out) die_errno("fopen");

    pid_t root = fork();
    if (root < 0) die_errno("fork");

    if (root == 0) {
        if (ptrace(PTRACE_TRACEME, 0, NULL, NULL) != 0) {
            perror("PTRACE_TRACEME");
            _exit(126);
        }

        raise(SIGSTOP);
        execvp(cmd[0], cmd);
        perror("execvp");
        _exit(127);
    }

    int status = 0;
    if (waitpid(root, &status, 0) < 0) die_errno("waitpid");
    if (!WIFSTOPPED(status)) {
        fprintf(stderr, "ERROR: proceso inicial no detenido\n");
        fclose(event_out);
        return 2;
    }

    long options = PTRACE_O_TRACESYSGOOD |
                   PTRACE_O_TRACEFORK |
                   PTRACE_O_TRACEVFORK |
                   PTRACE_O_TRACECLONE;

    if (ptrace(PTRACE_SETOPTIONS, root, NULL, options) != 0) die_errno("PTRACE_SETOPTIONS");
    if (!state_for(root, true)) {
        fprintf(stderr, "ERROR: sin espacio para tracee inicial\n");
        fclose(event_out);
        return 2;
    }
    if (ptrace(PTRACE_SYSCALL, root, NULL, NULL) != 0) die_errno("PTRACE_SYSCALL");

    int root_exit = 0;
    bool root_exit_seen = false;

    for (;;) {
        pid_t pid = waitpid(-1, &status, __WALL);

        if (pid < 0) {
            if (errno == EINTR) continue;
            if (errno == ECHILD) break;
            die_errno("waitpid");
        }

        if (WIFEXITED(status)) {
            if (pid == root) {
                root_exit = WEXITSTATUS(status);
                root_exit_seen = true;
            }
            remove_state(pid);
            continue;
        }

        if (WIFSIGNALED(status)) {
            if (pid == root) {
                root_exit = 128 + WTERMSIG(status);
                root_exit_seen = true;
            }
            remove_state(pid);
            continue;
        }

        if (!WIFSTOPPED(status)) continue;

        int sig = WSTOPSIG(status);
        unsigned int event = (unsigned int)status >> 16;

        if (sig == (SIGTRAP | 0x80)) {
            handle_syscall_stop(pid);
            ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            continue;
        }

        if (sig == SIGTRAP &&
            (event == PTRACE_EVENT_FORK ||
             event == PTRACE_EVENT_VFORK ||
             event == PTRACE_EVENT_CLONE)) {
            unsigned long child = 0;
            if (ptrace(PTRACE_GETEVENTMSG, pid, NULL, &child) == 0) {
                if (!state_for((pid_t)child, true)) {
                    fprintf(stderr, "ERROR: sin espacio para tracee hijo\n");
                    fclose(event_out);
                    return 2;
                }
            }
            ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            continue;
        }

        if (sig == SIGSTOP) {
            ptrace(PTRACE_SETOPTIONS, pid, NULL, options);
            ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            continue;
        }

        if (sig == SIGTRAP) {
            ptrace(PTRACE_SYSCALL, pid, NULL, NULL);
            continue;
        }

        ptrace(PTRACE_SYSCALL, pid, NULL, (void *)(long)sig);
    }

    fclose(event_out);
    return root_exit_seen ? root_exit : 2;
}

static void usage(const char *prog) {
    fprintf(stderr, "Uso: %s --output ARCHIVO -- COMANDO [ARGS...]\n", prog);
}

int main(int argc, char **argv) {
    if (argc < 5 || strcmp(argv[1], "--output") != 0 || strcmp(argv[3], "--") != 0) {
        usage(argv[0]);
        return 2;
    }

    return trace_command(&argv[4], argv[2]);
}
