#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern char **environ;

static const char *direct_lookup(const char *name) {
    size_t n = strlen(name);

    for (char **p = environ; p && *p; p++) {
        if (strncmp(*p, name, n) == 0 && (*p)[n] == '=') {
            return *p + n + 1;
        }
    }

    return NULL;
}

int main(int argc, char **argv) {
    if (argc != 2) return 2;

    if (strcmp(argv[1], "getenv") == 0) {
        const char *v = getenv("F9_GROUND_TRUTH");
        printf("getenv:%s\n", v ? v : "missing");
        return v ? 0 : 1;
    }

    if (strcmp(argv[1], "environ") == 0) {
        const char *v = direct_lookup("F9_GROUND_TRUTH");
        printf("environ:%s\n", v ? v : "missing");
        return v ? 0 : 1;
    }

    return 2;
}
