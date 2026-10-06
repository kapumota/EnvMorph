#define _GNU_SOURCE
#include <fcntl.h>
#include <stdatomic.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <unistd.h>

extern char **environ;

static int log_fd = -1;
static __thread int in_hook;
static _Atomic unsigned long long sequence_no;

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

static size_t append_u00xx(
    char *dst,
    size_t pos,
    unsigned char value
) {
    static const char hex[] = "0123456789abcdef";

    dst[pos++] = '\\';
    dst[pos++] = 'u';
    dst[pos++] = '0';
    dst[pos++] = '0';
    dst[pos++] = hex[(value >> 4) & 0x0f];
    dst[pos++] = hex[value & 0x0f];
    return pos;
}

static size_t append_json_string(
    char *dst,
    size_t pos,
    const char *value
) {
    dst[pos++] = '"';

    for (
        const unsigned char *p =
            (const unsigned char *)(value ? value : "");
        *p;
        p++
    ) {
        if (*p == '"' || *p == '\\') {
            dst[pos++] = '\\';
            dst[pos++] = (char)*p;
        } else if (*p == '\n') {
            dst[pos++] = '\\';
            dst[pos++] = 'n';
        } else if (*p == '\r') {
            dst[pos++] = '\\';
            dst[pos++] = 'r';
        } else if (*p == '\t') {
            dst[pos++] = '\\';
            dst[pos++] = 't';
        } else if (*p < 0x20 || *p >= 0x80) {
            pos = append_u00xx(dst, pos, *p);
        } else {
            dst[pos++] = (char)*p;
        }
    }

    dst[pos++] = '"';
    return pos;
}

static void emit_record(
    const char *name,
    int present
) {
    size_t name_len = strlen(name ? name : "");

    if (name_len > (SIZE_MAX - 512U) / 6U) {
        return;
    }

    size_t cap = 512U + name_len * 6U;
    char local[1024];
    char *record = local;
    int mapped = 0;

    if (cap > sizeof(local)) {
        void *memory = (void *)syscall(
            SYS_mmap,
            NULL,
            cap,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0
        );

        if (memory == MAP_FAILED) {
            return;
        }

        record = (char *)memory;
        mapped = 1;
    }

    unsigned long long seq =
        atomic_fetch_add_explicit(
            &sequence_no,
            1ULL,
            memory_order_relaxed
        ) + 1ULL;

    int n = snprintf(
        record,
        cap,
        "{\"kind\":\"getenv\",\"pid\":%ld,\"ppid\":%ld,"
        "\"seq\":%llu,\"name\":",
        (long)syscall(SYS_getpid),
        (long)syscall(SYS_getppid),
        seq
    );

    if (n <= 0 || (size_t)n >= cap) {
        if (mapped) {
            syscall(SYS_munmap, record, cap);
        }
        return;
    }

    size_t pos = (size_t)n;
    pos = append_json_string(record, pos, name);

    const char *suffix = present
        ? ",\"present\":true}\n"
        : ",\"present\":false}\n";
    size_t suffix_len = strlen(suffix);

    if (pos + suffix_len > cap) {
        if (mapped) {
            syscall(SYS_munmap, record, cap);
        }
        return;
    }

    memcpy(record + pos, suffix, suffix_len);
    pos += suffix_len;

    /*
     * Un evento lógico se emite con una sola llamada write.
     * El archivo se abrió con O_APPEND.
     */
    (void)syscall(SYS_write, log_fd, record, pos);

    if (mapped) {
        syscall(SYS_munmap, record, cap);
    }
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
        in_hook = 1;
        emit_record(name, value != NULL);
        in_hook = 0;
    }

    return value;
}
