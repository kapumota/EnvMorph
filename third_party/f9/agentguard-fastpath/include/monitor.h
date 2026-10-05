/**
 * @file monitor.h
 * @brief Monitoreo de procesos e intercepción de syscalls
 * 
 * Inspirado en:
 *   - Tsinghua ClawGuard: monitoreo en tiempo de ejecución con ptrace
 *   - ETH Zurich eBPF-PATROL: monitoreo de syscalls con sobrecarga menor al 2.5%
 *   - CMU Aegis-4: validación de sensores y telemetría
 */

#ifndef AG_MONITOR_H
#define AG_MONITOR_H

#include "agentguard.h"
#include "config.h"

/* Estados del monitor */
typedef enum {
    AG_MON_STATE_INIT = 0,
    AG_MON_STATE_ATTACHING = 1,
    AG_MON_STATE_RUNNING = 2,
    AG_MON_STATE_PAUSED = 3,
    AG_MON_STATE_DETACHING = 4,
    AG_MON_STATE_STOPPED = 5,
    AG_MON_STATE_ERROR = 6
} ag_mon_state_t;

/* Información de syscall */
typedef struct {
    int syscall_num;
    char syscall_name[64];
    uint64_t args[6];
    uint64_t ret;
    pid_t pid;
    pid_t tid;
    uid_t uid;
    gid_t gid;
    char comm[AG_MAX_COMM_LEN];
    char exe[AG_MAX_PATH_LEN];
    bool is_enter;
    uint64_t timestamp_ns;
    uint64_t duration_ns;
} ag_syscall_info_t;

/* Información de operación de archivo */
typedef struct {
    char path[AG_MAX_PATH_LEN];
    int flags;
    mode_t mode;
    int fd;
    bool is_read;
    bool is_write;
    bool is_create;
    bool is_delete;
    bool is_execute;
} ag_file_op_t;

/* Información de operación de red */
typedef struct {
    int family;
    int type;
    int protocol;
    char local_addr[64];
    int local_port;
    char remote_addr[64];
    int remote_port;
    bool is_connect;
    bool is_bind;
    bool is_listen;
    bool is_accept;
} ag_net_op_t;

/* Instantánea de información del proceso */
typedef struct {
    pid_t pid;
    pid_t ppid;
    pid_t pgid;
    pid_t sid;
    uid_t uid;
    gid_t gid;
    char comm[AG_MAX_COMM_LEN];
    char exe[AG_MAX_PATH_LEN];
    char cwd[AG_MAX_PATH_LEN];
    size_t vm_size;
    size_t vm_rss;
    size_t vm_swap;
    long num_hilos;
    long open_fds;
    uint64_t start_time;
    char state;
} ag_proc_snapshot_t;

/* Contexto del monitor */
typedef struct {
    ag_mon_state_t state;
    pid_t target_pid;
    int   ptrace_options;
    bool  follow_forks;
    bool  follow_vforks;
    bool  follow_clones;

    /* Estadísticas */
    uint64_t total_syscalls;
    uint64_t blocked_syscalls;
    uint64_t alerted_syscalls;
    uint64_t total_files;
    uint64_t blocked_files;
    uint64_t total_net_ops;
    uint64_t blocked_net_ops;
    uint64_t total_execs;
    uint64_t blocked_execs;

    /* Limitación de tasa */
    uint64_t syscall_count_last_sec;
    uint64_t file_op_count_last_sec;
    uint64_t net_conn_count_last_min;
    time_t last_rate_reset;
} ag_monitor_t;

extern ag_monitor_t g_monitor;

/* Ciclo de vida del monitor */
int  ag_monitor_init(pid_t target_pid);
void ag_monitor_shutdown(void);
int  ag_monitor_attach(pid_t pid);
int  ag_monitor_detach(pid_t pid);
int  ag_monitor_run(void);
void ag_monitor_pause(void);
void ag_monitor_resume(void);

/* Bucle de eventos */
int  ag_monitor_wait_event(pid_t *pid, int *status);
int  ag_monitor_handle_syscall(pid_t pid);
int  ag_monitor_handle_signal(pid_t pid, int sig);

/* Manejadores de syscalls */
int  ag_monitor_handle_open(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_openat(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_execve(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_execveat(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_connect(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_bind(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_socket(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_clone(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_fork(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_mmap(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_mprotect(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_ptrace(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_kill(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_chmod(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_chown(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_unlink(pid_t pid, ag_syscall_info_t *info);
int  ag_monitor_handle_rename(pid_t pid, ag_syscall_info_t *info);

/* Funciones utilitarias */
int  ag_monitor_get_syscall_args(pid_t pid, struct user_regs_struct *regs, 
                                  ag_syscall_info_t *info);
int  ag_monitor_read_string(pid_t pid, uint64_t addr, char *buf, size_t maxlen);
int  ag_monitor_read_buffer(pid_t pid, uint64_t addr, uint8_t *buf, size_t len);
ag_action_t ag_monitor_evaluate_and_act(ag_syscall_info_t *info, 
                                         ag_file_op_t *fop,
                                         ag_net_op_t *nop);

#endif /* AG_MONITOR_H */
