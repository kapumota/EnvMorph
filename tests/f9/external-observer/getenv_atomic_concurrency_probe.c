#define _GNU_SOURCE
#include <pthread.h>
#include <stdlib.h>

enum {
    THREADS = 8,
    ITERATIONS = 1000
};

typedef struct {
    int id;
    int failed;
} worker_t;

static void *worker_main(void *opaque) {
    worker_t *worker = (worker_t *)opaque;

    for (int i = 0; i < ITERATIONS; i++) {
        const char *name =
            ((i + worker->id) & 1)
            ? "F9_ATOMIC_A"
            : "F9_ATOMIC_B";

        if (getenv(name) == NULL) {
            worker->failed = 1;
            return NULL;
        }
    }

    return NULL;
}

int main(void) {
    pthread_t threads[THREADS];
    worker_t workers[THREADS] = {0};

    for (int i = 0; i < THREADS; i++) {
        workers[i].id = i;
        if (pthread_create(
            &threads[i],
            NULL,
            worker_main,
            &workers[i]
        ) != 0) {
            return 2;
        }
    }

    for (int i = 0; i < THREADS; i++) {
        if (pthread_join(threads[i], NULL) != 0) {
            return 3;
        }
        if (workers[i].failed) {
            return 4;
        }
    }

    return 0;
}
