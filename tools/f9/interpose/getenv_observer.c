#define _GNU_SOURCE
#include <fcntl.h>
#include <stddef.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

extern char **environ;

static int log_fd = -1;
static __thread int in_hook;

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
    static const char quote[] = "\"";
    raw_write_all(fd, quote, sizeof(quote) - 1);

    for (const unsigned char *p = (const unsigned char *)(s ? s : ""); *p; p++) {
        if (*p == '"' || *p == '\\') {
            char pair[2] = {'\\', (char)*p};
            raw_write_all(fd, pair, sizeof(pair));
        } else if (*p == '\n') {
            static const char esc[] = "\\n";
            raw_write_all(fd, esc, sizeof(esc) - 1);
        } else if (*p == '\r') {
            static const char esc[] = "\\r";
            raw_write_all(fd, esc, sizeof(esc) - 1);
        } else if (*p == '\t') {
            static const char esc[] = "\\t";
            raw_write_all(fd, esc, sizeof(esc) - 1);
        } else if (*p >= 0x20) {
            char c = (char)*p;
            raw_write_all(fd, &c, 1);
        }
    }

    raw_write_all(fd, quote, sizeof(quote) - 1);
}

__attribute__((constructor))
static void init_observer(void) {
    const char *path = env_direct("ENVMORPH_F9_INTERPOSE_LOG");

    if (!path || !*path) return;

    log_fd = (int)syscall(
        SYS_openat,
        AT_FDCWD,
        path,
        O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC,
        0600
    );
}

char *getenv(const char *name) {
    char *value = (char *)env_direct(name);

    if (!in_hook && log_fd >= 0) {
        static const char prefix[] = "{\"kind\":\"getenv\",\"name\":";
        static const char present[] = ",\"present\":";
        static const char ending[] = "}\n";

        in_hook = 1;
        raw_write_all(log_fd, prefix, sizeof(prefix) - 1);
        json_escape_write(log_fd, name);
        raw_write_all(log_fd, present, sizeof(present) - 1);

        if (value) {
            static const char yes[] = "true";
            raw_write_all(log_fd, yes, sizeof(yes) - 1);
        } else {
            static const char no[] = "false";
            raw_write_all(log_fd, no, sizeof(no) - 1);
        }

        raw_write_all(log_fd, ending, sizeof(ending) - 1);
        in_hook = 0;
    }

    return value;
}
