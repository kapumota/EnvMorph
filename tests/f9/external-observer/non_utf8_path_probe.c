#define _GNU_SOURCE
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <unistd.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

int main(int argc, char **argv) {
    if (argc != 2) return 2;

    const unsigned char leaf[] = {
        'n', 'o', 'n', 'u', 't', 'f', '8', '-', 0xfe,
        '.', 't', 'x', 't', '\0'
    };

    char path[PATH_MAX];

    int written = snprintf(
        path,
        sizeof(path),
        "%s/%s",
        argv[1],
        (const char *)leaf
    );

    if (written < 0 || (size_t)written >= sizeof(path)) return 3;

    int fd = open(path, O_CREAT | O_WRONLY | O_TRUNC, 0600);
    if (fd < 0) return 4;

    const char payload[] = "envmorph-f9\n";

    if (write(fd, payload, sizeof(payload) - 1) < 0) {
        close(fd);
        return 5;
    }

    if (close(fd) != 0) return 6;

    fd = open(path, O_RDONLY);
    if (fd < 0) return 7;

    if (close(fd) != 0) return 8;

    return 0;
}
