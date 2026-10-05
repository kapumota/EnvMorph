#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

static int consume(const char *path) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;

    char c;
    ssize_t n = read(fd, &c, 1);
    close(fd);

    return n == 1 ? 0 : -1;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;

    const char *value = getenv("F9_GROUND_TRUTH");
    if (!value) return 3;

    char fixture[PATH_MAX];
    char missing[PATH_MAX];

    snprintf(fixture, sizeof(fixture), "%s/fixture.txt", argv[1]);
    snprintf(missing, sizeof(missing), "%s/missing.txt", argv[1]);

    if (consume(fixture) != 0) return 4;

    errno = 0;
    int fd = open(missing, O_RDONLY);

    if (fd >= 0) {
        close(fd);
        return 5;
    }

    if (errno != ENOENT) return 6;

    printf("value=%s\n", value);
    return 0;
}
