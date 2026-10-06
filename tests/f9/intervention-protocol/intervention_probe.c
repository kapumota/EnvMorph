#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int read_file(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) return 1;

    char buf[256];
    size_t n = fread(buf, 1, sizeof(buf) - 1, fp);
    fclose(fp);

    buf[n] = '\0';
    printf("file:%s\n", buf);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) return 2;

    if (strcmp(argv[1], "env") == 0) {
        const char *v = getenv("F9_GROUND_TRUTH");
        if (!v) return 3;
        printf("env:%s\n", v);
        return 0;
    }

    if (strcmp(argv[1], "file") == 0) {
        if (argc != 3) return 4;
        return read_file(argv[2]);
    }

    return 5;
}
