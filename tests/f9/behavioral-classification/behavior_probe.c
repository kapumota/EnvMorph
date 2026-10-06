#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int run_relevant(void) {
    const char *value = getenv("F9_RELEVANT");
    if (!value) return 2;

    printf("relevant:%s\n", value);
    return 0;
}

static int run_stable(void) {
    const char *value = getenv("F9_STABLE");
    if (!value) return 3;

    printf("stable\n");
    return 0;
}

static int run_unresolved(void) {
    const char *value = getenv("F9_UNRESOLVED");
    if (!value) return 4;

    if (strcmp(value, "fail") == 0) {
        fprintf(stderr, "controlled-treatment-failure\n");
        return 7;
    }

    printf("unresolved:%s\n", value);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) return 1;

    if (strcmp(argv[1], "relevant") == 0) {
        return run_relevant();
    }

    if (strcmp(argv[1], "stable") == 0) {
        return run_stable();
    }

    if (strcmp(argv[1], "unresolved") == 0) {
        return run_unresolved();
    }

    return 5;
}
