/**
 * @file main.c
 * @brief AgentGuard-C - Punto de entrada principal
 * 
 * AgentGuard-C es un monitor de seguridad en tiempo de ejecución para agentes de IA,
 * inspirado en investigación de Tsinghua, ETH Zurich, CMU, NUS,
 * Stanford y TUM.
 * 
 * Uso:
 *   ./agentguard -p <pid>              # Monitorear proceso existente
 *   ./agentguard -c <command> [args]   # Lanzar y monitorear comando
 *   ./agentguard -f <config.yml>       # Usar archivo de configuración
 *   ./agentguard --integrity <path>    # Verificar integridad de archivo
 *   ./agentguard --baseline            # Crear línea base de integridad
 *   ./agentguard --report              # Generar reporte de auditoría
 * 
 * @author Proyecto académico de software
 * @date 2026-05-23
 */

#include "agentguard.h"
#include "utils.h"
#include "config.h"
#include "monitor.h"
#include "integrity.h"
#include "audit.h"
#include "sandbox.h"

/* Contexto global */
ag_context_t g_ctx = {0};

static struct option long_options[] = {
    {"pid",          required_argument, 0, 'p'},
    {"command",      required_argument, 0, 'c'},
    {"config",       required_argument, 0, 'f'},
    {"log",          required_argument, 0, 'l'},
    {"report",       required_argument, 0, 'r'},
    {"integrity",    required_argument, 0, 'i'},
    {"baseline",     no_argument,       0, 'b'},
    {"audit",        no_argument,       0, 'a'},
    {"daemon",       no_argument,       0, 'd'},
    {"json",         no_argument,       0, 'j'},
    {"verbose",      no_argument,       0, 'v'},
    {"help",         no_argument,       0, 'h'},
    {"version",      no_argument,       0, 'V'},
    {0, 0, 0, 0}
};

static void parse_args(int argc, char **argv);
static int  launch_and_monitor(const char *cmd, char **args);
static int  monitor_existing(pid_t pid);
static int  run_integrity_check(const char *path);
static int  create_baseline(void);
static int  generate_report(const char *path);
void ag_signal_handler(int sig);

int main(int argc, char **argv) {
    int ret;

    /* Inicializa variables globales */
    memset(&g_ctx, 0, sizeof(g_ctx));
    g_ctx.running = true;
    g_ctx.start_time = time(NULL);
    g_ctx.log_fd = STDERR_FILENO;

    /* Analiza línea de comandos */
    parse_args(argc, argv);

    /* Imprime banner */
    ag_print_banner();

    /* Configura manejadores de señales */
    signal(SIGINT,  ag_signal_handler);
    signal(SIGTERM, ag_signal_handler);
    signal(SIGQUIT, ag_signal_handler);
    signal(SIGCHLD, ag_signal_handler);
    signal(SIGUSR1, ag_signal_handler);
    signal(SIGUSR2, ag_signal_handler);

    /* Inicializa subsistemas */
    ag_log(AG_LOG_INFO, "Inicializando AgentGuard-C v%s", AG_VERSION_STRING);
    ag_log(AG_LOG_INFO, "Inspirado en investigación de Tsinghua, ETH Zurich, "
                         "CMU, NUS, Stanford y TUM");

    if (strlen(g_ctx.config_path) > 0) {
        if (ag_config_init(g_ctx.config_path) != 0) {
            ag_die("No se pudo cargar la configuración: %s", g_ctx.config_path);
        }
    } else {
        ag_config_set_defaults();
        ag_log(AG_LOG_WARN, "Usando configuración predeterminada (sin archivo de configuración)");
    }

    /* Inicializa sistema de auditoría */
    if (ag_audit_init(g_config.audit_file) != 0) {
        ag_log(AG_LOG_WARN, "No se pudo inicializar el registro de auditoría");
    }

    /* Inicializa subsistema de aislamiento */
    if (ag_sandbox_init() != 0) {
        ag_log(AG_LOG_WARN, "No se pudo inicializar el subsistema de aislamiento");
    }

    /* Dirige al modo correspondiente */
    if (g_ctx.integrity_mode) {
        ret = run_integrity_check(g_ctx.config_path);
    } else if (g_ctx.audit_mode) {
        ret = create_baseline();
    } else if (strlen(g_ctx.report_path) > 0) {
        ret = generate_report(g_ctx.report_path);
    } else if (g_ctx.target_pid > 0) {
        ret = monitor_existing(g_ctx.target_pid);
    } else if (strlen(g_ctx.target_comm) > 0) {
        /* Analiza comando y argumentos */
        char *cmd = g_ctx.target_comm;
        char *args[AG_MAX_ARGS] = {0};
        int arg_count = 0;

        args[arg_count++] = cmd;
        for (int i = optind; i < argc && arg_count < AG_MAX_ARGS - 1; i++) {
            args[arg_count++] = argv[i];
        }
        args[arg_count] = NULL;

        ret = launch_and_monitor(cmd, args);
    } else {
        ag_print_usage(argv[0]);
        ret = AG_EXIT_ERROR;
    }

    /* Limpieza */
    ag_audit_free();
    ag_config_free();
    ag_sandbox_free();
    ag_close_log_file();

    ag_log(AG_LOG_INFO, "AgentGuard-C saliendo con código %d", ret);
    ag_print_stats();

    return ret;
}

static void parse_args(int argc, char **argv) {
    int c;
    int option_index = 0;

    while ((c = getopt_long(argc, argv, "p:c:f:l:r:i:badjvVh",
                            long_options, &option_index)) != -1) {
        switch (c) {
            case 'p':
                g_ctx.target_pid = atoi(optarg);
                if (g_ctx.target_pid <= 0) {
                    ag_die("PID inválido: %s", optarg);
                }
                break;

            case 'c':
                ag_strlcpy(g_ctx.target_comm, optarg, sizeof(g_ctx.target_comm));
                break;

            case 'f':
                ag_strlcpy(g_ctx.config_path, optarg, sizeof(g_ctx.config_path));
                break;

            case 'l':
                ag_strlcpy(g_ctx.log_path, optarg, sizeof(g_ctx.log_path));
                ag_set_log_file(g_ctx.log_path);
                break;

            case 'r':
                ag_strlcpy(g_ctx.report_path, optarg, sizeof(g_ctx.report_path));
                break;

            case 'i':
                g_ctx.integrity_mode = true;
                ag_strlcpy(g_ctx.config_path, optarg, sizeof(g_ctx.config_path));
                break;

            case 'b':
                g_ctx.audit_mode = true;
                break;

            case 'a':
                g_ctx.audit_mode = true;
                break;

            case 'd':
                g_ctx.interactive_mode = false;
                break;

            case 'j':
                g_config.json_output = true;
                break;

            case 'v':
                ag_set_log_level(AG_LOG_DEBUG);
                break;

            case 'V':
                printf("AgentGuard-C v%s\n", AG_VERSION_STRING);
                printf("Compilado para investigación en DevSecOps de agentes\n");
                exit(AG_EXIT_SUCCESS);

            case 'h':
            default:
                ag_print_usage(argv[0]);
                exit(AG_EXIT_SUCCESS);
        }
    }
}

static int launch_and_monitor(const char *cmd, char **args) {
    pid_t pid;
    int ret;

    ag_log(AG_LOG_INFO, "Lanzando comando: %s", cmd);

    pid = fork();
    if (pid < 0) {
        ag_perror("fork falló");
        return AG_EXIT_ERROR;
    }

    if (pid == 0) {
        /* Proceso hijo */
        /* Solicita trazado por el proceso padre */
        if (ptrace(PTRACE_TRACEME, 0, NULL, NULL) < 0) {
            ag_perror("ptrace(PTRACE_TRACEME) falló");
            _exit(AG_EXIT_ERROR);
        }

        /*  Se detiene  para que el proceso padre pueda adjuntarse */
        raise(SIGSTOP);

        /* Ejecuta comando objetivo */
        execvp(cmd, args);
        ag_perror("execvp falló");
        _exit(AG_EXIT_ERROR);
    }

    /* Proceso padre */
    g_ctx.target_pid = pid;
    ag_log(AG_LOG_INFO, "Proceso hijo started: PID %d", pid);

    ret = ag_monitor_init(pid);
    if (ret != 0) {
        ag_log(AG_LOG_ERROR, "No se pudo inicializar el monitor");
        kill(pid, SIGKILL);
        return AG_EXIT_ERROR;
    }

    ret = ag_monitor_run();

    ag_monitor_shutdown();

    return ret;
}

static int monitor_existing(pid_t pid) {
    int ret;

    ag_log(AG_LOG_INFO, "Adjuntándose al proceso existente: PID %d", pid);

    /* Verifica que el proceso exista */
    if (kill(pid, 0) != 0) {
        ag_die("El proceso %d no existe o no se puede acceder", pid);
    }

    ret = ag_monitor_init(pid);
    if (ret != 0) {
        ag_log(AG_LOG_ERROR, "No se pudo inicializar el monitor");
        return AG_EXIT_ERROR;
    }

    ret = ag_monitor_run();

    ag_monitor_shutdown();

    return ret;
}

static int run_integrity_check(const char *path) {
    int ret;

    ag_log(AG_LOG_INFO, "Ejecutando verificación de integridad en: %s", path);

    ret = ag_integrity_init(NULL);
    if (ret != 0) {
        ag_log(AG_LOG_ERROR, "No se pudo inicializar el subsistema de integridad");
        return AG_EXIT_ERROR;
    }

    if (ag_file_exists(path)) {
        ag_integrity_result_t result;
        ret = ag_integrity_check_file(path, &result);
        if (ret == 0) {
            ag_integrity_print_report();
        }
    } else {
        ag_log(AG_LOG_ERROR, "Ruta no encontrada: %s", path);
        ret = AG_EXIT_ERROR;
    }

    ag_integrity_free();
    return ret;
}

static int create_baseline(void) {
    int ret;

    ag_log(AG_LOG_INFO, "Creando línea base de integridad");

    ret = ag_integrity_init(NULL);
    if (ret != 0) {
        ag_log(AG_LOG_ERROR, "No se pudo inicializar el subsistema de integridad");
        return AG_EXIT_ERROR;
    }

    ret = ag_integrity_create_baseline();

    ag_integrity_free();
    return ret;
}

static int generate_report(const char *path) {
    ag_log(AG_LOG_INFO, "Generando reporte de auditoría: %s", path);

    /* Genera evidencia de cumplimiento */
    ag_audit_generate_evidence("SOC2", path);
    ag_audit_generate_evidence("ISO27001", path);
    ag_audit_generate_evidence("NIST", path);

    ag_log(AG_LOG_INFO, "Reporte generado correctamente");
    return AG_EXIT_SUCCESS;
}

void ag_print_banner(void) {
    printf("\n");
    printf("╔══════════════════════════════════════════════════════════════╗\n");
    printf("║                                                              ║\n");
    printf("║   █████╗  ██████╗ ███████╗███╗   ██╗████████╗               ║\n");
    printf("║  ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝               ║\n");
    printf("║  ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║                  ║\n");
    printf("║  ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║                  ║\n");
    printf("║  ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║                  ║\n");
    printf("║  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝                  ║\n");
    printf("║                                                              ║\n");
    printf("║   ██████╗ ██╗   ██╗ █████╗ ██████╗ ██████╗                  ║\n");
    printf("║  ██╔════╝ ██║   ██║██╔══██╗██╔══██╗██╔══██╗                 ║\n");
    printf("║  ██║  ███╗██║   ██║███████║██████╔╝██║  ██║                 ║\n");
    printf("║  ██║   ██║██║   ██║██╔══██║██╔══██╗██║  ██║                 ║\n");
    printf("║  ╚██████╔╝╚██████╔╝██║  ██║██║  ██║██████╔╝                 ║\n");
    printf("║   ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝                  ║\n");
    printf("║                                                              ║\n");
    printf("║   AgentGuard-C v%-6s  Monitor de seguridad en ejecución   ║\n", AG_VERSION_STRING);
    printf("║   Software - Investigación DevSecOps de agentes    ║\n");
    printf("║                                                              ║\n");
    printf("║   Inspirado en:                                             ║\n");
    printf("║   • Tsinghua University  (OpenClaw, SDFUZZ, LABRADOR)        ║\n");
    printf("║   • ETH Zurich           (eBPF-PATROL, Sistemas seguros)     ║\n");
    printf("║   • CMU                  (Aegis-4, SEI)                      ║\n");
    printf("║   • NUS / NTU            (Agentic-VAPT, AutoSOC)             ║\n");
    printf("║   • Stanford             (Marco de riesgo de IA)             ║\n");
    printf("║   • TUM                  (UniBPF, Sistemas confiables)       ║\n");
    printf("║                                                              ║\n");
    printf("╚══════════════════════════════════════════════════════════════╝\n");
    printf("\n");
}

void ag_print_usage(const char *prog) {
    printf("Uso: %s [OPTIONS]\n\n", prog);
    printf("AgentGuard-C - Monitor de seguridad en tiempo de ejecución para agentes de IA\n\n");
    printf("Opciones:\n");
    printf("  -p, --pid <pid>           Monitorear proceso existente\n");
    printf("  -c, --command <cmd>       Lanzar y monitorear comando\n");
    printf("  -f, --config <archivo>    Cargar archivo de configuración\n");
    printf("  -l, --log <archivo>       Escribir registros en archivo\n");
    printf("  -r, --report <archivo>    Generar reporte de auditoría\n");
    printf("  -i, --integrity <ruta>    Verificar integridad de archivo\n");
    printf("  -b, --baseline            Crear línea base de integridad\n");
    printf("  -a, --audit               Modo auditoría\n");
    printf("  -d, --daemon              Ejecutar como demonio\n");
    printf("  -j, --json                Salida en formato JSON\n");
    printf("  -v, --verbose             Salida detallada (nivel debug)\n");
    printf("  -V, --version             Mostrar versión\n");
    printf("  -h, --help                Mostrar esta ayuda\n\n");
    printf("Ejemplos:\n");
    printf("  %s -c python agent.py              # Monitorear agente Python\n", prog);
    printf("  %s -p 1234                         # Adjuntarse al PID 1234\n", prog);
    printf("  %s -c ./my-agent -f config.yml     # Con configuración\n", prog);
    printf("  %s -i /etc/passwd                  # Verificación de integridad\n", prog);
    printf("  %s -b                              # Crear línea base\n", prog);
    printf("  %s -r report.json                  # Generar reporte\n\n", prog);
}

void ag_print_stats(void) {
    time_t now = time(NULL);
    double elapsed = difftime(now, g_ctx.start_time);

    ag_log(AG_LOG_INFO, "->Sesión Estadísticas");
    ag_log(AG_LOG_INFO, "Duración: %.0f segundos", elapsed);
    ag_log(AG_LOG_INFO, "Eventos totales: %lu", g_ctx.event_count);
    ag_log(AG_LOG_INFO, "Acciones bloqueadas: %lu", g_ctx.block_count);
    ag_log(AG_LOG_INFO, "Alertas generadas: %lu", g_ctx.alert_count);
    ag_log(AG_LOG_INFO, "Eventos/seg: %.2f", 
           elapsed > 0 ? (double)g_ctx.event_count / elapsed : 0);
}

void ag_signal_handler(int sig) {
    switch (sig) {
        case SIGINT:
        case SIGTERM:
        case SIGQUIT:
            ag_log(AG_LOG_INFO, "Señal %d recibida; apagando...", sig);
            g_ctx.running = false;
            break;

        case SIGCHLD:
            /* Recolecta procesos hijo terminados */
            while (waitpid(-1, NULL, WNOHANG) > 0);
            break;

        case SIGUSR1:
            ag_log(AG_LOG_INFO, "SIGUSR1 recibida; recargando configuración...");
            if (strlen(g_ctx.config_path) > 0) {
                ag_config_reload(g_ctx.config_path);
            }
            break;

        case SIGUSR2:
            ag_log(AG_LOG_INFO, "SIGUSR2 recibida; imprimiendo estadísticas...");
            ag_print_stats();
            break;
    }
}
