/**
 * @file agentguard.h
 * @brief AgentGuard-C - Monitor de seguridad en tiempo de ejecución para agentes de IA
 * 
 * Software inspirado en:
 *   - Tsinghua University (OpenClaw/ClawGuard, SDFUZZ, LABRADOR)
 *   - ETH Zurich (eBPF-PATROL, Grupo de sistemas seguros y confiables)
 *   - CMU (Aegis-4, SEI)
 *   - NUS (Agentic-VAPT)
 *   - Stanford (AI Risk Framework)
 *   - TUM (UniBPF, Grupo de sistemas confiables)
 * 
 * @version 1.0.0
 * @date 2026-05-23
 * @license MIT
 */

#ifndef AGENTGUARD_H
#define AGENTGUARD_H

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <unistd.h>
#include <errno.h>
#include <signal.h>
#include <time.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/ptrace.h>
#include <sys/user.h>
#include <sys/syscall.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>
#include <pwd.h>
#include <grp.h>
#include <getopt.h>
#include <assert.h>

/* Información de versión */
#define AG_VERSION_MAJOR 1
#define AG_VERSION_MINOR 0
#define AG_VERSION_PATCH 0
#define AG_VERSION_STRING "1.0.0"

/* Límites */
#define AG_MAX_PATH_LEN       4096
#define AG_MAX_COMM_LEN        256
#define AG_MAX_ARGS            128
#define AG_MAX_POLICIES         64
#define AG_MAX_SYSCALLS        512
#define AG_MAX_LOG_ENTRIES     10000
#define AG_MAX_INTEGRITY_FILES   256
#define AG_MAX_NETWORK_CONNS     512
#define AG_CONFIG_LINE_LEN      1024
#define AG_HASH_LEN              64
#define AG_MAX_REPORT_SIZE     65536

/* Códigos de salida */
#define AG_EXIT_SUCCESS          0
#define AG_EXIT_ERROR            1
#define AG_EXIT_POLICY_VIOLATION 2
#define AG_EXIT_SIGNAL           3

/* Niveles de registro */
typedef enum {
    AG_LOG_DEBUG = 0,
    AG_LOG_INFO  = 1,
    AG_LOG_WARN  = 2,
    AG_LOG_ERROR = 3,
    AG_LOG_FATAL = 4,
    AG_LOG_AUDIT = 5
} ag_log_level_t;

/* Tipos de evento */
typedef enum {
    AG_EVENT_SYSCALL_ENTER = 0,
    AG_EVENT_SYSCALL_EXIT  = 1,
    AG_EVENT_FILE_ACCESS   = 2,
    AG_EVENT_PROC_EXEC     = 3,
    AG_EVENT_NET_CONNECT   = 4,
    AG_EVENT_NET_BIND      = 5,
    AG_EVENT_INTEGRITY     = 6,
    AG_EVENT_POLICY_BLOCK  = 7,
    AG_EVENT_CONFIG_RELOAD = 8,
    AG_EVENT_HEARTBEAT     = 9
} ag_event_type_t;

/* Contexto global */
typedef struct {
    pid_t target_pid;
    char target_comm[AG_MAX_COMM_LEN];
    bool interactive_mode;
    bool audit_mode;
    bool integrity_mode;
    bool network_mode;
    char config_path[AG_MAX_PATH_LEN];
    char log_path[AG_MAX_PATH_LEN];
    char report_path[AG_MAX_PATH_LEN];
    int  log_fd;
    bool running;
    uint64_t event_count;
    uint64_t block_count;
    uint64_t alert_count;
    time_t start_time;
} ag_context_t;

extern ag_context_t g_ctx;

/* Ciclo de vida principal */
int  ag_init(int argc, char **argv);
void ag_shutdown(void);
int  ag_run(void);
void ag_print_banner(void);
void ag_print_usage(const char *prog);
void ag_print_stats(void);

#endif /* AGENTGUARD_H */
