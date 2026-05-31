// UloOS bare-metal libc stub: time.h
#ifndef _TIME_H
#define _TIME_H

#include <stddef.h>

typedef long long time_t;
typedef long long clock_t;

#define CLOCKS_PER_SEC 1000

time_t time(time_t *t);
clock_t clock(void);

struct tm {
    int tm_sec;
    int tm_min;
    int tm_hour;
    int tm_mday;
    int tm_mon;
    int tm_year;
    int tm_wday;
    int tm_yday;
    int tm_isdst;
};

#endif
