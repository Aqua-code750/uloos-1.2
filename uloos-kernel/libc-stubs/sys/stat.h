// UloOS bare-metal libc stub: sys/stat.h
#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <stddef.h>

struct stat {
    unsigned long st_size;
    unsigned long st_mode;
};

int stat(const char *path, struct stat *buf);
int mkdir(const char *path, int mode);

#endif
