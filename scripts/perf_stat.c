/* perf_stat: perf_event_open-based hardware counter harness for hosts without
 * the `perf` binary (issue #49: "Profile branch misses and IPC using perf
 * stat"). Counts for a command: cycles, instructions, branches, branch-misses,
 * and L1-icache-load-misses — the same events
 * `perf stat -e cycles,instructions,branches,branch-misses,L1-icache-load-misses`
 * would report. Userspace-only (perf_event_paranoid >= 2 compatible).
 *
 * Events are attached directly to the forked child pid (perf-stat style)
 * rather than via inherit=1: inherited child counts do not reliably
 * propagate to the parent's read() on this kernel, which silently reported
 * only parent noise (~2.5M cycles regardless of workload).
 *
 * Build & run:
 *   gcc -O2 -o /tmp/perf_stat scripts/perf_stat.c
 *   /tmp/perf_stat -- ./target/release/deps/tag_dispatch-<hash> <filter>
 */
#define _GNU_SOURCE
#include <linux/perf_event.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <unistd.h>

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static long open_ev(pid_t pid, __u64 config, __u32 type) {
    struct perf_event_attr pe;
    memset(&pe, 0, sizeof(pe));
    pe.type = type;
    pe.size = sizeof(pe);
    pe.config = config;
    pe.disabled = 1;        /* enabled together right before child release */
    pe.exclude_kernel = 1;  /* required at perf_event_paranoid >= 2 */
    return syscall(__NR_perf_event_open, &pe, pid, -1, -1, 0);
}

struct ev {
    const char *name;
    long fd;
};

int main(int argc, char **argv) {
    if (argc < 3 || strcmp(argv[1], "--") != 0) {
        fprintf(stderr, "usage: perf_stat -- <command> [args...]\n");
        return 2;
    }

    int sync[2];
    if (pipe(sync) != 0) {
        perror("pipe");
        return 4;
    }

    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        return 4;
    }
    if (pid == 0) {
        /* Block until the parent has the counters armed on this pid. */
        close(sync[1]);
        char c;
        if (read(sync[0], &c, 1) != 1) {
            _exit(4);
        }
        close(sync[0]);
        execvp(argv[2], &argv[2]);
        perror("exec");
        _exit(127);
    }

    close(sync[0]);

    const __u64 l1i_read_miss =
        (__u64)PERF_COUNT_HW_CACHE_L1I |
        ((__u64)PERF_COUNT_HW_CACHE_OP_READ << 8) |
        ((__u64)PERF_COUNT_HW_CACHE_RESULT_MISS << 16);

    struct ev evs[] = {
        {"cycles", open_ev(pid, PERF_COUNT_HW_CPU_CYCLES, PERF_TYPE_HARDWARE)},
        {"instructions", open_ev(pid, PERF_COUNT_HW_INSTRUCTIONS, PERF_TYPE_HARDWARE)},
        {"branches", open_ev(pid, PERF_COUNT_HW_BRANCH_INSTRUCTIONS, PERF_TYPE_HARDWARE)},
        {"branch-misses", open_ev(pid, PERF_COUNT_HW_BRANCH_MISSES, PERF_TYPE_HARDWARE)},
        {"L1-icache-load-misses", open_ev(pid, l1i_read_miss, PERF_TYPE_HW_CACHE)},
    };
    const int n = (int)(sizeof(evs) / sizeof(evs[0]));

    int avail = 0;
    for (int i = 0; i < n; i++) {
        if (evs[i].fd < 0) {
            fprintf(stderr, "perf_stat: %s unavailable (%s)\n", evs[i].name,
                    strerror(errno));
        } else {
            avail++;
            ioctl(evs[i].fd, PERF_EVENT_IOC_ENABLE, 0);
        }
    }
    if (avail == 0) {
        fprintf(stderr, "perf_stat: no PMU counters available\n");
        /* Don't strand the child. */
        (void)!write(sync[1], "x", 1);
        close(sync[1]);
        int st = 0;
        waitpid(pid, &st, 0);
        return 3;
    }

    /* Release the child: counters are attached and enabled. */
    if (write(sync[1], "x", 1) != 1) {
        perror("write");
        return 4;
    }
    close(sync[1]);

    int status = 0;
    waitpid(pid, &status, 0);
    for (int i = 0; i < n; i++) {
        if (evs[i].fd >= 0) {
            ioctl(evs[i].fd, PERF_EVENT_IOC_DISABLE, 0);
        }
    }

    unsigned long long vals[5] = {0, 0, 0, 0, 0};
    for (int i = 0; i < n; i++) {
        if (evs[i].fd >= 0 &&
            read(evs[i].fd, &vals[i], sizeof(vals[i])) != (ssize_t)sizeof(vals[i])) {
            vals[i] = 0;
        }
    }

    fprintf(stderr, "\n");
    for (int i = 0; i < n; i++) {
        if (evs[i].fd >= 0) {
            fprintf(stderr, "%18llu  %s\n", vals[i], evs[i].name);
        }
    }
    if (vals[0] > 0) {
        fprintf(stderr, "\nIPC: %.3f", (double)vals[1] / (double)vals[0]);
        if (vals[2] > 0) {
            fprintf(stderr, "   branch-miss rate: %.4f%%",
                    100.0 * (double)vals[3] / (double)vals[2]);
        }
        fprintf(stderr, "\n");
    }

    return WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
}
