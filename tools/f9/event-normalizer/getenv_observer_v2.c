#define _GNU_SOURCE
#include <fcntl.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

extern char **environ;

static int log_fd = -1;
static __thread int in_hook;
static unsigned long long sequence_no;

static const char *env_direct(const char *name) {
    size_t n = strlen(name);

    if (!environ) return NULL;

    for (char **p = environ; *p; p++) {
        if (strncmp(*p, name, n) == 0 && (*p)[n] == '=') {
            return *p + n + 1;
        }
    }

    return NULL;
}

static void raw_write_all(int fd, const char *buf, size_t len) {
    while (len > 0) {
        ssize_t n = syscall(SYS_write, fd, buf, len);
        if (n <= 0) return;
        buf += (size_t)n;
        len -= (size_t)n;
    }
}

static void json_escape_write(int fd, const char *s) {
    raw_write_all(fd, "\"", 1);

    for (const unsigned char *p = (const unsigned char *)(s ? s : ""); *p; p++) {
        if (*p == '"' || *p == '\\') {
            char pair[2] = {'\\', (char)*p};
            raw_write_all(fd, pair, sizeof(pair));
        } else if (*p == '\n') {
            raw_write_all(fd, "\\n", 2);
        } else if (*p == '\r') {
            raw_write_all(fd, "\\r", 2);
        } else if (*p == '\t') {
            raw_write_all(fd, "\\t", 2);
        } else if (*p >= 0x20) {
            char c = (char)*p;
            raw_write_all(fd, &c, 1);
        }
    }

    raw_write_all(fd, "\"", 1);
}

__attribute__((constructor))
static void init_observer(void) {
    const char *path = env_direct("ENVMORPH_F9_INTERPOSE_LOG");

    if (!path || !*path) return;

    log_fd = (int)syscall(
        SYS_openat,
        AT_FDCWD,
        path,
        O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC,
        0600
    );
}

char *getenv(const char *name) {
    char *value = (char *)env_direct(name);

    if (!in_hook && log_fd >= 0) {
        char numeric[160];
        int n;

        in_hook = 1;
        sequence_no++;

        n = snprintf(
            numeric,
            sizeof(numeric),
            "{\"kind\":\"getenv\",\"pid\":%ld,\"ppid\":%ld,\"seq\":%llu,\"name\":",
            (long)syscall(SYS_getpid),
            (long)syscall(SYS_getppid),
            sequence_no
        );

        if (n > 0) {
            raw_write_all(log_fd, numeric, (size_t)n);
        }

        json_escape_write(log_fd, name);

        if (value) {
            static const char present_true[] = ",\"present\":true}\n";
            raw_write_all(log_fd, present_true, sizeof(present_true) - 1);
        } else {
            static const char present_false[] = ",\"present\":false}\n";
            raw_write_all(log_fd, present_false, sizeof(present_false) - 1);
        }

        in_hook = 0;
    }

    return value;
}
