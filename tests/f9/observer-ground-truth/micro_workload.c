#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

static int consume_fd(int fd) {
    char c;
    ssize_t n = read(fd, &c, 1);
    close(fd);
    return n == 1 ? 0 : 1;
}

static int case_absolute(const char *root) {
    char path[PATH_MAX];
    snprintf(path, sizeof(path), "%s/fixture.txt", root);
    int fd = open(path, O_RDONLY);
    if (fd < 0) return 1;
    if (consume_fd(fd) != 0) return 1;
    puts("absolute:OK");
    return 0;
}

static int case_relative(const char *root) {
    if (chdir(root) != 0) return 1;
    int fd = open("fixture.txt", O_RDONLY);
    if (fd < 0) return 1;
    if (consume_fd(fd) != 0) return 1;
    puts("relative:OK");
    return 0;
}

static int case_dirfd(const char *root) {
    int dirfd = open(root, O_RDONLY | O_DIRECTORY);
    if (dirfd < 0) return 1;

    int fd = openat(dirfd, "fixture.txt", O_RDONLY);
    close(dirfd);
    if (fd < 0) return 1;
    if (consume_fd(fd) != 0) return 1;
    puts("dirfd:OK");
    return 0;
}

static int case_missing(const char *root) {
    char path[PATH_MAX];
    snprintf(path, sizeof(path), "%s/missing.txt", root);

    errno = 0;
    int fd = open(path, O_RDONLY);
    if (fd >= 0) {
        close(fd);
        return 1;
    }
    if (errno != ENOENT) return 1;

    puts("missing:ENOENT");
    return 0;
}

static int case_forkexec(const char *root) {
    char path[PATH_MAX];
    snprintf(path, sizeof(path), "%s/fixture.txt", root);

    pid_t pid = fork();
    if (pid < 0) return 1;

    if (pid == 0) {
        execl("/bin/cat", "cat", path, (char *)NULL);
        _exit(127);
    }

    int status = 0;
    if (waitpid(pid, &status, 0) < 0) return 1;
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) return 1;

    puts("forkexec:OK");
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "Uso: %s CASO ROOT\n", argv[0]);
        return 2;
    }

    if (strcmp(argv[1], "absolute") == 0) return case_absolute(argv[2]);
    if (strcmp(argv[1], "relative") == 0) return case_relative(argv[2]);
    if (strcmp(argv[1], "dirfd") == 0) return case_dirfd(argv[2]);
    if (strcmp(argv[1], "missing") == 0) return case_missing(argv[2]);
    if (strcmp(argv[1], "forkexec") == 0) return case_forkexec(argv[2]);

    fprintf(stderr, "Caso desconocido: %s\n", argv[1]);
    return 2;
}
