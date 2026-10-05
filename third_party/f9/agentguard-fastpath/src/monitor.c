/**
 * @file monitor.c
 * @brief Monitoreo de procesos e intercepción de syscalls via ptrace
 * 
 * Inspirado en:
 *   - Tsinghua ClawGuard: monitoreo en tiempo de ejecución basado en ptrace
 *   - ETH Zurich eBPF-PATROL: monitoreo de syscalls con sobrecarga menor al 2.5%
 *   - CMU Aegis-4: validación de sensores y telemetría
 */

#include "agentguard.h"
#include "utils.h"
#include "config.h"
#include "monitor.h"
#include "audit.h"
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

ag_monitor_t g_monitor = {0};

static void ag_monitor_join_path(char *dst, size_t dst_size,
                                 const char *dirpath, const char *relpath) {
    if (!dst || dst_size == 0) return;

    const char *safe_dir = dirpath ? dirpath : "";
    const char *safe_rel = relpath ? relpath : "";
    size_t pos = 0;

    while (safe_dir[pos] && pos + 1 < dst_size) {
        dst[pos] = safe_dir[pos];
        pos++;
    }

    if (pos + 1 < dst_size && (pos == 0 || dst[pos - 1] != '/')) {
        dst[pos++] = '/';
    }

    for (size_t i = 0; safe_rel[i] && pos + 1 < dst_size; i++) {
        dst[pos++] = safe_rel[i];
    }

    dst[pos] = '\0';
}


/* Acceso a registros específico de la arquitectura */
#ifdef __x86_64__
    #define REG_SYSCALL(regs) ((regs).orig_rax)
    #define REG_ARG0(regs)    ((regs).rdi)
    #define REG_ARG1(regs)    ((regs).rsi)
    #define REG_ARG2(regs)    ((regs).rdx)
    #define REG_ARG3(regs)    ((regs).r10)
    #define REG_ARG4(regs)    ((regs).r8)
    #define REG_ARG5(regs)    ((regs).r9)
    #define REG_RET(regs)     ((regs).rax)
#elif __i386__
    #define REG_SYSCALL(regs) ((regs).orig_eax)
    #define REG_ARG0(regs)    ((regs).ebx)
    #define REG_ARG1(regs)    ((regs).ecx)
    #define REG_ARG2(regs)    ((regs).edx)
    #define REG_ARG3(regs)    ((regs).esi)
    #define REG_ARG4(regs)    ((regs).edi)
    #define REG_ARG5(regs)    ((regs).ebp)
    #define REG_RET(regs)     ((regs).eax)
#else
    #error "Arquitectura no soportada"
#endif

int ag_monitor_init(pid_t target_pid) {
    memset(&g_monitor, 0, sizeof(g_monitor));

    g_monitor.target_pid = target_pid;
    g_monitor.state = AG_MON_STATE_INIT;
    g_monitor.ptrace_options = PTRACE_O_TRACESYSGOOD |
                               PTRACE_O_TRACECLONE |
                               PTRACE_O_TRACEFORK |
                               PTRACE_O_TRACEVFORK |
                               PTRACE_O_TRACEEXIT;
    g_monitor.follow_forks = true;
    g_monitor.follow_vforks = true;
    g_monitor.follow_clones = true;
    g_monitor.last_rate_reset = time(NULL);

    ag_log(AG_LOG_INFO, "Monitor inicializado para PID %d", target_pid);
    return 0;
}

void ag_monitor_shutdown(void) {
    if (g_monitor.state == AG_MON_STATE_RUNNING) {
        ag_log(AG_LOG_INFO, "Apagando monitor");

        if (g_monitor.target_pid > 0) {
            ptrace(PTRACE_DETACH, g_monitor.target_pid, NULL, NULL);
        }
    }

    g_monitor.state = AG_MON_STATE_STOPPED;
}

int ag_monitor_attach(pid_t pid) {
    ag_log(AG_LOG_DEBUG, "Adjuntándose al PID %d", pid);

    if (ptrace(PTRACE_ATTACH, pid, NULL, NULL) < 0) {
        ag_perror("ptrace(PTRACE_ATTACH)");
        return -1;
    }

    /* Espera a que el proceso se detenga */
    int status;
    if (waitpid(pid, &status, 0) < 0) {
        ag_perror("waitpid");
        return -1;
    }

    if (!WIFSTOPPED(status)) {
        ag_log(AG_LOG_ERROR, "El proceso %d no está detenido (estado=%d)", pid, status);
        return -1;
    }

    /* Configura opciones de ptrace */
    if (ptrace(PTRACE_SETOPTIONS, pid, NULL, g_monitor.ptrace_options) < 0) {
        ag_perror("ptrace(PTRACE_SETOPTIONS)");
        return -1;
    }

    ag_log(AG_LOG_DEBUG, "Adjuntado correctamente al PID %d", pid);
    return 0;
}

int ag_monitor_detach(pid_t pid) {
    ag_log(AG_LOG_DEBUG, "Desadjuntándose del PID %d", pid);

    if (ptrace(PTRACE_DETACH, pid, NULL, NULL) < 0) {
        ag_perror("ptrace(PTRACE_DETACH)");
        return -1;
    }

    return 0;
}

int ag_monitor_run(void) {
    int status;
    pid_t pid;

    g_monitor.state = AG_MON_STATE_RUNNING;

    /* Si estamos iniciando un nuevo proceso, ya está detenido desde PTRACE_TRACEME. */
    if (g_ctx.target_pid > 0 && g_monitor.target_pid == g_ctx.target_pid) {
        /* Espera la señal SIGSTOP inicial */
        if (waitpid(g_monitor.target_pid, &status, 0) < 0) {
            ag_perror("waitpid");
            return -1;
        }

        /* Opciones de ptrace */
        if (ptrace(PTRACE_SETOPTIONS, g_monitor.target_pid, NULL,
                   g_monitor.ptrace_options) < 0) {
            ag_perror("ptrace(PTRACE_SETOPTIONS)");
            return -1;
        }

        /* Continua el proceso */
        if (ptrace(PTRACE_SYSCALL, g_monitor.target_pid, NULL, NULL) < 0) {
            ag_perror("ptrace(PTRACE_SYSCALL)");
            return -1;
        }
    } else {
        /* Adjunta a un proceso existente */
        if (ag_monitor_attach(g_monitor.target_pid) != 0) {
            return -1;
        }

        /* Continua el proceso */
        if (ptrace(PTRACE_SYSCALL, g_monitor.target_pid, NULL, NULL) < 0) {
            ag_perror("ptrace(PTRACE_SYSCALL)");
            return -1;
        }
    }

    ag_log(AG_LOG_INFO, "Monitor en ejecución. Presiona Ctrl+C para detener.");

    /* Bucle principal de eventos */
    while (g_ctx.running) {
        pid = waitpid(-1, &status, __WALL);

        if (pid < 0) {
            if (errno == EINTR) {
                continue;
            }
            ag_perror("waitpid");
            break;
        }

        if (WIFEXITED(status)) {
            ag_log(AG_LOG_INFO, "El proceso %d salió con estado %d",
                   pid, WEXITSTATUS(status));

            if (pid == g_monitor.target_pid) {
                break;
            }
            continue;
        }

        if (WIFSIGNALED(status)) {
            ag_log(AG_LOG_INFO, "El proceso %d fue terminado por la señal %d",
                   pid, WTERMSIG(status));

            if (pid == g_monitor.target_pid) {
                break;
            }
            continue;
        }

        if (WIFSTOPPED(status)) {
            int sig = WSTOPSIG(status);

            /* Verifica si es una pausa de syscall (PTRACE_O_TRACESYSGOOD). */
            if (sig == (SIGTRAP | 0x80)) {
                /* Syscall entry/exit */
                ag_monitor_handle_syscall(pid);
            } else if (sig == SIGTRAP) {
                /* Maneja fork/clone/vfork/exit eventos */
                unsigned long event = status >> 16;

                switch (event) {
                    case PTRACE_EVENT_FORK:
                    case PTRACE_EVENT_VFORK:
                    case PTRACE_EVENT_CLONE: {
                        unsigned long child_pid;
                        ptrace(PTRACE_GETEVENTMSG, pid, NULL, &child_pid);
                        ag_log(AG_LOG_INFO, "Nuevo proceso hijo: %lu", child_pid);
                        break;
                    }
                    case PTRACE_EVENT_EXIT:
                        ag_log(AG_LOG_DEBUG, "Proceso %d saliendo", pid);
                        break;
                }

                /* Continua sin entregar SIGTRAP*/
                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
            } else if (sig == SIGSTOP) {
                /* Parada inicial desde PTRACE_TRACEME o PTRACE_ATTACH*/
                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
            } else {
                /* Entrega otras señales al proceso. */
                ptrace(PTRACE_SYSCALL, pid, NULL, sig);
            }
        }
    }

    g_monitor.state = AG_MON_STATE_STOPPED;
    return AG_EXIT_SUCCESS;
}

int ag_monitor_handle_syscall(pid_t pid) {
    struct user_regs_struct regs;
    ag_syscall_info_t info = {0};
    ag_file_op_t fop = {0};
    ag_net_op_t nop = {0};
    ag_action_t action;

    /* Lee registros */
    if (ptrace(PTRACE_GETREGS, pid, NULL, &regs) < 0) {
        ag_perror("ptrace(PTRACE_GETREGS)");
        return -1;
    }

    /* Completa información de syscall */
    info.syscall_num = REG_SYSCALL(regs);
    info.args[0] = REG_ARG0(regs);
    info.args[1] = REG_ARG1(regs);
    info.args[2] = REG_ARG2(regs);
    info.args[3] = REG_ARG3(regs);
    info.args[4] = REG_ARG4(regs);
    info.args[5] = REG_ARG5(regs);
    info.pid = pid;
    info.timestamp_ns = ag_timestamp_ns();
    ag_strlcpy(info.syscall_name, ag_syscall_name(info.syscall_num), sizeof(info.syscall_name));

    /* Obtiene información del proceso */
    ag_proc_get_comm(pid, info.comm, sizeof(info.comm));
    ag_proc_get_exe(pid, info.exe, sizeof(info.exe));
    ag_proc_get_uid_gid(pid, &info.uid, &info.gid);

    g_ctx.event_count++;
    g_monitor.total_syscalls++;

    /* Limitación de tasa de verificacion */
    time_t now = time(NULL);
    if (now > g_monitor.last_rate_reset) {
        g_monitor.syscall_count_last_sec = 0;
        g_monitor.last_rate_reset = now;
    }
    g_monitor.syscall_count_last_sec++;

    /* Obtiene política para este proceso */
    ag_policy_t *policy = ag_config_get_policy_for_process(info.comm);

    /* Maneja syscalls específicas */
    switch (info.syscall_num) {
        case SYS_open:
        case SYS_openat:
        case SYS_creat: {
            char path[AG_MAX_PATH_LEN];
            int flags = 0;
            if (info.syscall_num == SYS_openat) {
                int dirfd = (int)info.args[0];
                uint64_t path_addr = info.args[1];
                flags = (int)info.args[2];

                if (dirfd == AT_FDCWD) {
                    ag_monitor_read_string(pid, path_addr, path, sizeof(path));
                } else {
                    char dirpath[AG_MAX_PATH_LEN];
                    ag_proc_get_fd_path(pid, dirfd, dirpath, sizeof(dirpath));
                    ag_monitor_read_string(pid, path_addr, path, sizeof(path));
                    if (path[0] != '/') {
                        char fullpath[AG_MAX_PATH_LEN];
                        ag_monitor_join_path(fullpath, sizeof(fullpath), dirpath, path);
                        ag_strlcpy(path, fullpath, sizeof(path));
                    }
                }

                fop.is_read = (flags & O_RDONLY) && !(flags & O_WRONLY);
                fop.is_write = (flags & O_WRONLY) || (flags & O_RDWR);
                fop.is_create = (flags & O_CREAT);
            } else {
                ag_monitor_read_string(pid, info.args[0], path, sizeof(path));
                flags = (int)info.args[1];
                fop.is_read = (flags & O_RDONLY) && !(flags & O_WRONLY);
                fop.is_write = (flags & O_WRONLY) || (flags & O_RDWR);
                fop.is_create = (flags & O_CREAT);
            }

            ag_strlcpy(fop.path, path, sizeof(fop.path));

            action = ag_policy_evaluate_file(policy, path,
                                              info.syscall_num == SYS_openat ?
                                              (int)info.args[2] : (int)info.args[1]);

            if (action == AG_ACTION_BLOCK) {
                g_ctx.block_count++;
                g_monitor.blocked_files++;

                /* Reemplaza syscall con getpid (sin efecto) */
                regs.orig_rax = SYS_getpid;
                ptrace(PTRACE_SETREGS, pid, NULL, &regs);

                ag_log(AG_LOG_WARN, "[BLOQUEADO] Acceso a archivo: %s por %s (PID %d)",
                       path, info.comm, pid);
                ag_audit_add_file_op(path, flags, AG_ACTION_BLOCK);

                /* Continua el proceso */
                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
                return 0;
            }
            break;
        }

        case SYS_execve:
        case SYS_execveat: {
            char path[AG_MAX_PATH_LEN];
            if (info.syscall_num == SYS_execveat) {
                int dirfd = (int)info.args[0];
                uint64_t path_addr = info.args[1];

                if (dirfd == AT_FDCWD) {
                    ag_monitor_read_string(pid, path_addr, path, sizeof(path));
                } else {
                    char dirpath[AG_MAX_PATH_LEN];
                    ag_proc_get_fd_path(pid, dirfd, dirpath, sizeof(dirpath));
                    ag_monitor_read_string(pid, path_addr, path, sizeof(path));
                    if (path[0] != '/') {
                        char fullpath[AG_MAX_PATH_LEN];
                        ag_monitor_join_path(fullpath, sizeof(fullpath), dirpath, path);
                        ag_strlcpy(path, fullpath, sizeof(path));
                    }
                }
            } else {
                ag_monitor_read_string(pid, info.args[0], path, sizeof(path));
            }

            action = ag_policy_evaluate_exec(policy, path);

            if (action == AG_ACTION_BLOCK) {
                g_ctx.block_count++;
                g_monitor.blocked_execs++;

                /* Reemplaza syscall con getpid */
                regs.orig_rax = SYS_getpid;
                ptrace(PTRACE_SETREGS, pid, NULL, &regs);

                ag_log(AG_LOG_WARN, "[BLOQUEADO] Ejecución: %s por %s (PID %d)",
                       path, info.comm, pid);
                ag_audit_add(AG_AUDIT_PROCESS, AG_SEV_HIGH,
                             "execve", path, "BLOQUEAR", "violacion_politica",
                             "Ejecución bloqueada por política");

                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
                return 0;
            }
            break;
        }

        case SYS_connect: {
            uint64_t addr = info.args[1];

            /* Lee sockaddr */
            struct sockaddr_storage ss;
            ag_monitor_read_buffer(pid, addr, (uint8_t*)&ss, sizeof(ss));

            char host[64] = "desconocido";
            int port = 0;

            if (ss.ss_family == AF_INET) {
                struct sockaddr_in *sin = (struct sockaddr_in*)&ss;
                inet_ntop(AF_INET, &sin->sin_addr, host, sizeof(host));
                port = ntohs(sin->sin_port);
            } else if (ss.ss_family == AF_INET6) {
                struct sockaddr_in6 *sin6 = (struct sockaddr_in6*)&ss;
                inet_ntop(AF_INET6, &sin6->sin6_addr, host, sizeof(host));
                port = ntohs(sin6->sin6_port);
            }

            ag_strlcpy(nop.remote_addr, host, sizeof(nop.remote_addr));
            nop.remote_port = port;
            nop.is_connect = true;

            action = ag_policy_evaluate_network(policy, host, port);

            if (action == AG_ACTION_BLOCK) {
                g_ctx.block_count++;
                g_monitor.blocked_net_ops++;

                regs.orig_rax = SYS_getpid;
                ptrace(PTRACE_SETREGS, pid, NULL, &regs);

                ag_log(AG_LOG_WARN,
                       "[BLOQUEADO] Conexión de red: %s:%d por %s (PID %d)",
                       host, port, info.comm, pid);
                ag_audit_add_net_op(&nop, AG_ACTION_BLOCK);

                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
                return 0;
            }
            break;
        }

        case SYS_ptrace: {
            /* Bloquea siempre intentos de ptrace */
            g_ctx.block_count++;

            regs.orig_rax = SYS_getpid;
            ptrace(PTRACE_SETREGS, pid, NULL, &regs);

            ag_log(AG_LOG_WARN, "[BLOQUEADO] Intento de ptrace por %s (PID %d)",
                   info.comm, pid);
            ag_audit_add(AG_AUDIT_SYSCALL, AG_SEV_CRITICAL,
                         "ptrace", "process", "BLOQUEAR", "violacion_seguridad",
                         "syscall ptrace bloqueada: posible intento de depuración");

            ptrace(PTRACE_SYSCALL, pid, NULL, 0);
            return 0;
        }

        case SYS_kill:
        case SYS_tgkill:
        case SYS_tkill: {
            pid_t target = (pid_t)info.args[0];
            int sig = (int)info.args[1];

            /* Bloquea intentos de terminar el monitor o init. */
            if (target == 1 || target == getpid()) {
                g_ctx.block_count++;

                regs.orig_rax = SYS_getpid;
                ptrace(PTRACE_SETREGS, pid, NULL, &regs);

                ag_log(AG_LOG_WARN,
                       "[BLOQUEADO] Señal %d a PID %d por %s (PID %d)",
                       sig, target, info.comm, pid);

                ptrace(PTRACE_SYSCALL, pid, NULL, 0);
                return 0;
            }
            break;
        }
    }

    /* Evalua la política general de llamadas al sistema */
    action = ag_policy_evaluate_syscall(policy, info.syscall_num);

    if (action == AG_ACTION_BLOCK) {
        g_ctx.block_count++;
        g_monitor.blocked_syscalls++;

        /* Reemplaza con  getpid */
        regs.orig_rax = SYS_getpid;
        ptrace(PTRACE_SETREGS, pid, NULL, &regs);

        ag_log(AG_LOG_WARN, "[BLOQUEADO] Syscall %s por %s (PID %d)",
               info.syscall_name, info.comm, pid);
        ag_audit_add_syscall(&info, AG_ACTION_BLOCK);

        ptrace(PTRACE_SYSCALL, pid, NULL, 0);
        return 0;
    }

    /* Registrar llamadas syscalls en el nivel de depuración*/
    if (g_config.log_level <= AG_LOG_DEBUG) {
        ag_log(AG_LOG_DEBUG, "Syscall %s(%lu, %lu, %lu, %lu, %lu, %lu) por %s",
               info.syscall_name,
               info.args[0], info.args[1], info.args[2],
               info.args[3], info.args[4], info.args[5],
               info.comm);
    }

    /* Continua el proceso */
    ptrace(PTRACE_SYSCALL, pid, NULL, 0);
    return 0;
}

int ag_monitor_read_string(pid_t pid, uint64_t addr, char *buf, size_t maxlen) {
    size_t i = 0;

    while (i < maxlen - 1) {
        long word = ptrace(PTRACE_PEEKDATA, pid, addr + i, NULL);
        if (word < 0 && errno) {
            buf[i] = '\0';
            return -1;
        }

        for (size_t j = 0; j < sizeof(long) && i < maxlen - 1; j++, i++) {
            char c = ((char*)&word)[j];
            if (c == '\0') {
                buf[i] = '\0';
                return 0;
            }
            buf[i] = c;
        }
    }

    buf[maxlen - 1] = '\0';
    return 0;
}

int ag_monitor_read_buffer(pid_t pid, uint64_t addr, uint8_t *buf, size_t len) {
    size_t i = 0;

    while (i < len) {
        long word = ptrace(PTRACE_PEEKDATA, pid, addr + i, NULL);
        if (word < 0 && errno) {
            return -1;
        }

        size_t copy_len = sizeof(long);
        if (i + copy_len > len) {
            copy_len = len - i;
        }

        memcpy(buf + i, &word, copy_len);
        i += sizeof(long);
    }

    return 0;
}

/* El manejador de señales global está implementado en main.c. */

#if 0
void ag_signal_handler(int sig) {
    switch (sig) {
        case SIGINT:
        case SIGTERM:
        case SIGQUIT:
            ag_log(AG_LOG_INFO, "Señal %d recibida; apagando...", sig);
            g_ctx.running = false;
            break;

        case SIGCHLD:
            while (waitpid(-1, NULL, WNOHANG) > 0);
            break;

        case SIGUSR1:
            ag_log(AG_LOG_INFO, "SIGUSR1 recibida; recargando configuración...");
            if (strlen(g_ctx.config_path) > 0) {
                ag_config_reload(g_ctx.config_path);
            }
            break;

        case SIGUSR2:
            ag_log(AG_LOG_INFO, "SIGUSR2 recibida; imprimiendo estadísticas...");
            ag_print_stats();
            break;
    }
}

#endif
