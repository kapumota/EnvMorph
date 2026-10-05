/**
 * @file utils.c
 * @brief Funciones utilitarias implementation
 */

#include "agentguard.h"
#include "utils.h"
#include <stdarg.h>
#include <ctype.h>
#include <sys/time.h>
#include <openssl/sha.h>
#include <openssl/evp.h>

static ag_log_level_t g_log_level = AG_LOG_INFO;
static int g_log_fd = STDERR_FILENO;
static bool g_log_color = true;

/* Utilidades de marca de tiempo */

char* ag_timestamp_iso8601(char *buf, size_t buflen) {
    struct timespec ts;
    struct tm tm;

    clock_gettime(CLOCK_REALTIME, &ts);
    localtime_r(&ts.tv_sec, &tm);

    snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02d.%03ld",
             tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday,
             tm.tm_hour, tm.tm_min, tm.tm_sec,
             ts.tv_nsec / 1000000);

    return buf;
}

char* ag_timestamp_human(char *buf, size_t buflen) {
    struct timespec ts;
    struct tm tm;

    clock_gettime(CLOCK_REALTIME, &ts);
    localtime_r(&ts.tv_sec, &tm);

    snprintf(buf, buflen, "%04d-%02d-%02d %02d:%02d:%02d",
             tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday,
             tm.tm_hour, tm.tm_min, tm.tm_sec);

    return buf;
}

uint64_t ag_timestamp_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + ts.tv_nsec;
}

/*  Utilidades de cadenas*/

char* ag_str_trim(char *str) {
    char *end;

    while (isspace((unsigned char)*str)) str++;

    if (*str == 0) return str;

    end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) end--;

    end[1] = '\0';
    return str;
}

char* ag_str_dup(const char *src) {
    if (!src) return NULL;
    size_t len = strlen(src);
    char *dst = ag_malloc(len + 1);
    memcpy(dst, src, len + 1);
    return dst;
}

size_t ag_strlcpy(char *dst, const char *src, size_t dst_size) {
    const char *safe_src = src ? src : "";
    size_t src_len = strlen(safe_src);

    if (dst_size > 0) {
        size_t copy_len = src_len >= dst_size ? dst_size - 1 : src_len;
        memcpy(dst, safe_src, copy_len);
        dst[copy_len] = '\0';
    }

    return src_len;
}

int ag_str_startswith(const char *str, const char *prefix) {
    return strncmp(str, prefix, strlen(prefix)) == 0;
}

int ag_str_endswith(const char *str, const char *suffix) {
    size_t str_len = strlen(str);
    size_t suffix_len = strlen(suffix);
    if (suffix_len > str_len) return 0;
    return strcmp(str + str_len - suffix_len, suffix) == 0;
}

void ag_str_tolower(char *str) {
    for (int i = 0; str[i]; i++) {
        str[i] = tolower((unsigned char)str[i]);
    }
}

void ag_str_replace_char(char *str, char from, char to) {
    for (int i = 0; str[i]; i++) {
        if (str[i] == from) str[i] = to;
    }
}

 /* Utilidades de archivos*/


int ag_file_exists(const char *path) {
    struct stat st;
    return stat(path, &st) == 0;
}

int ag_file_readable(const char *path) {
    return access(path, R_OK) == 0;
}

off_t ag_file_size(const char *path) {
    struct stat st;
    if (stat(path, &st) != 0) return -1;
    return st.st_size;
}

int ag_file_copy(const char *src, const char *dst) {
    int fd_src, fd_dst;
    char buf[4096];
    ssize_t n;

    fd_src = open(src, O_RDONLY);
    if (fd_src < 0) return -1;

    fd_dst = open(dst, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd_dst < 0) {
        close(fd_src);
        return -1;
    }

    while ((n = read(fd_src, buf, sizeof(buf))) > 0) {
        if (write(fd_dst, buf, n) != n) {
            close(fd_src);
            close(fd_dst);
            return -1;
        }
    }

    close(fd_src);
    close(fd_dst);
    return 0;
}

int ag_file_mkdir_p(const char *path) {
    char tmp[AG_MAX_PATH_LEN];
    char *p = NULL;
    size_t len;

    snprintf(tmp, sizeof(tmp), "%s", path);
    len = strlen(tmp);
    if (tmp[len - 1] == '/') tmp[len - 1] = '\0';

    for (p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            mkdir(tmp, 0755);
            *p = '/';
        }
    }
    mkdir(tmp, 0755);

    return 0;
}

char* ag_file_read_all(const char *path, size_t *outlen) {
    FILE *fp = fopen(path, "rb");
    if (!fp) return NULL;

    fseek(fp, 0, SEEK_END);
    size_t len = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    char *data = ag_malloc(len + 1);
    if (fread(data, 1, len, fp) != len) {
        ag_free(data);
        fclose(fp);
        return NULL;
    }

    data[len] = '\0';
    fclose(fp);

    if (outlen) *outlen = len;
    return data;
}

int ag_file_write_all(const char *path, const char *data, size_t len) {
    FILE *fp = fopen(path, "wb");
    if (!fp) return -1;

    size_t written = fwrite(data, 1, len, fp);
    fclose(fp);

    return written == len ? 0 : -1;
}

 /* Utilidades hash (usando SHA256 de OpenSSL)*/

void ag_hash_sha256(const uint8_t *data, size_t len, uint8_t out[32]) {
    SHA256(data, len, out);
}

void ag_hash_to_hex(const uint8_t hash[32], char hex[AG_HASH_LEN + 1]) {
    for (int i = 0; i < 32; i++) {
        sprintf(hex + i * 2, "%02x", hash[i]);
    }
    hex[AG_HASH_LEN] = '\0';
}

int ag_hash_file(const char *path, char hex[AG_HASH_LEN + 1]) {
    FILE *fp = fopen(path, "rb");
    if (!fp) return -1;

    EVP_MD_CTX *ctx = EVP_MD_CTX_new();
    if (!ctx) {
        fclose(fp);
        return -1;
    }

    if (EVP_DigestInit_ex(ctx, EVP_sha256(), NULL) != 1) {
        EVP_MD_CTX_free(ctx);
        fclose(fp);
        return -1;
    }

    uint8_t buf[8192];
    size_t n;
    while ((n = fread(buf, 1, sizeof(buf), fp)) > 0) {
        if (EVP_DigestUpdate(ctx, buf, n) != 1) {
            EVP_MD_CTX_free(ctx);
            fclose(fp);
            return -1;
        }
    }

    uint8_t hash[32];
    unsigned int hash_len = 0;
    if (EVP_DigestFinal_ex(ctx, hash, &hash_len) != 1 || hash_len != sizeof(hash)) {
        EVP_MD_CTX_free(ctx);
        fclose(fp);
        return -1;
    }

    EVP_MD_CTX_free(ctx);
    fclose(fp);

    ag_hash_to_hex(hash, hex);
    return 0;
}

 /* Utilidades de procesos*/

pid_t ag_proc_find_by_name(const char *name) {
    DIR *dir = opendir("/proc");
    if (!dir) return -1;

    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (!isdigit(entry->d_name[0])) continue;

        pid_t pid = atoi(entry->d_name);
        char comm[AG_MAX_COMM_LEN];

        if (ag_proc_get_comm(pid, comm, sizeof(comm)) == 0) {
            if (strcmp(comm, name) == 0) {
                closedir(dir);
                return pid;
            }
        }
    }

    closedir(dir);
    return -1;
}

int ag_proc_get_comm(pid_t pid, char *buf, size_t buflen) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/comm", pid);

    FILE *fp = fopen(path, "r");
    if (!fp) return -1;

    if (fgets(buf, buflen, fp) != NULL) {
        size_t len = strlen(buf);
        if (len > 0 && buf[len - 1] == '\n') buf[len - 1] = '\0';
    }

    fclose(fp);
    return 0;
}

int ag_proc_get_exe(pid_t pid, char *buf, size_t buflen) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/exe", pid);

    ssize_t len = readlink(path, buf, buflen - 1);
    if (len < 0) return -1;

    buf[len] = '\0';
    return 0;
}

int ag_proc_get_cwd(pid_t pid, char *buf, size_t buflen) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/cwd", pid);

    ssize_t len = readlink(path, buf, buflen - 1);
    if (len < 0) return -1;

    buf[len] = '\0';
    return 0;
}

int ag_proc_get_uid_gid(pid_t pid, uid_t *uid, gid_t *gid) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/status", pid);

    FILE *fp = fopen(path, "r");
    if (!fp) return -1;

    char line[256];
    while (fgets(line, sizeof(line), fp)) {
        if (strncmp(line, "Uid:", 4) == 0) {
            sscanf(line, "Uid:\t%d", uid);
        } else if (strncmp(line, "Gid:", 4) == 0) {
            sscanf(line, "Gid:\t%d", gid);
        }
    }

    fclose(fp);
    return 0;
}

int ag_proc_list_fds(pid_t pid, ag_int_array_t *fds) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/fd", pid);

    DIR *dir = opendir(path);
    if (!dir) return -1;

    fds->len = 0;
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (isdigit(entry->d_name[0])) {
            if (fds->len < fds->cap) {
                fds->data[fds->len++] = atoi(entry->d_name);
            }
        }
    }

    closedir(dir);
    return 0;
}

int ag_proc_get_fd_path(pid_t pid, int fd, char *buf, size_t buflen) {
    char path[AG_MAX_PATH_LEN];
    snprintf(path, sizeof(path), "/proc/%d/fd/%d", pid, fd);

    ssize_t len = readlink(path, buf, buflen - 1);
    if (len < 0) return -1;

    buf[len] = '\0';
    return 0;
}

/* Utilidades de memoria*/

void* ag_malloc(size_t size) {
    void *ptr = malloc(size);
    if (!ptr && size > 0) {
        ag_die("Memoria agotada (malloc %zu bytes)", size);
    }
    return ptr;
}

void* ag_calloc(size_t nmemb, size_t size) {
    void *ptr = calloc(nmemb, size);
    if (!ptr && nmemb > 0 && size > 0) {
        ag_die("Memoria agotada (calloc %zu x %zu bytes)", nmemb, size);
    }
    return ptr;
}

void* ag_realloc(void *ptr, size_t size) {
    void *new_ptr = realloc(ptr, size);
    if (!new_ptr && size > 0) {
        ag_die("Memoria agotada (realloc %zu bytes)", size);
    }
    return new_ptr;
}

void ag_free(void *ptr) {
    free(ptr);
}


/* Utilidades de registro*/

static const char* log_level_names[] = {
    "DEBUG", "INFO", "WARN", "ERROR", "FATAL", "AUDIT"
};

static const char* log_level_colors[] = {
    "\033[36m",  /* DEBUG - cian */
    "\033[32m",  /* INFO - verde */
    "\033[33m",  /* WARN - amarillo */
    "\033[31m",  /* ERROR - rojo */
    "\033[35m",  /* FATAL - magenta */
    "\033[34m"   /* AUDIT - azul */
};

static const char* color_reset = "\033[0m";

void ag_set_log_level(ag_log_level_t level) {
    g_log_level = level;
}

void ag_set_log_file(const char *path) {
    int fd = open(path, O_WRONLY | O_CREAT | O_APPEND, 0644);
    if (fd >= 0) {
        if (g_log_fd >= 0 && g_log_fd != STDERR_FILENO) {
            close(g_log_fd);
        }
        g_log_fd = fd;
        g_log_color = false;
    }
}

void ag_close_log_file(void) {
    if (g_log_fd >= 0 && g_log_fd != STDERR_FILENO) {
        close(g_log_fd);
        g_log_fd = STDERR_FILENO;
    }
}

void ag_log(ag_log_level_t level, const char *fmt, ...) {
    if (level < g_log_level) return;

    char timestamp[32];
    ag_timestamp_human(timestamp, sizeof(timestamp));

    va_list args;
    va_start(args, fmt);

    if (g_log_color && g_log_fd == STDERR_FILENO) {
        dprintf(g_log_fd, "%s[%s] [%s] ", 
                log_level_colors[level],
                timestamp,
                log_level_names[level]);
        vdprintf(g_log_fd, fmt, args);
        dprintf(g_log_fd, "%s\n", color_reset);
    } else {
        dprintf(g_log_fd, "[%s] [%s] ", timestamp, log_level_names[level]);
        vdprintf(g_log_fd, fmt, args);
        dprintf(g_log_fd, "\n");
    }

    va_end(args);

    if (level == AG_LOG_FATAL) {
        exit(AG_EXIT_ERROR);
    }
}

void ag_log_raw(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vdprintf(g_log_fd, fmt, args);
    va_end(args);
}

/* Manejo de errores*/

void ag_perror(const char *msg) {
    ag_log(AG_LOG_ERROR, "%s: %s", msg, strerror(errno));
}

void ag_die(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);

    char buf[AG_MAX_COMM_LEN];
    vsnprintf(buf, sizeof(buf), fmt, args);
    ag_log(AG_LOG_FATAL, "%s", buf);

    va_end(args);
    exit(AG_EXIT_ERROR);
}

/*Formateador JSON */

void ag_json_begin_object(char *buf, size_t buflen, size_t *pos) {
    *pos += snprintf(buf + *pos, buflen - *pos, "{");
}

void ag_json_end_object(char *buf, size_t buflen, size_t *pos) {
    if (*pos > 0 && buf[*pos - 1] == ',') {
        (*pos)--;
    }
    *pos += snprintf(buf + *pos, buflen - *pos, "}");
}

void ag_json_add_string(char *buf, size_t buflen, size_t *pos,
                        const char *key, const char *val) {
    *pos += snprintf(buf + *pos, buflen - *pos, 
                     "\"%s\":\"%s\",", key, val);
}

void ag_json_add_int(char *buf, size_t buflen, size_t *pos,
                     const char *key, int64_t val) {
    *pos += snprintf(buf + *pos, buflen - *pos, 
                     "\"%s\":%ld,", key, val);
}

void ag_json_add_bool(char *buf, size_t buflen, size_t *pos,
                      const char *key, bool val) {
    *pos += snprintf(buf + *pos, buflen - *pos, 
                     "\"%s\":%s,", key, val ? "true" : "false");
}
