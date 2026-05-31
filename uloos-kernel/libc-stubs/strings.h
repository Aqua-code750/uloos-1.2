// UloOS bare-metal libc stub: strings.h (BSD-style string functions)
#ifndef _STRINGS_H
#define _STRINGS_H

#include <stddef.h>
#include <string.h>

int strcasecmp(const char *s1, const char *s2);
int strncasecmp(const char *s1, const char *s2, size_t n);

static inline void bzero(void *s, size_t n) { memset(s, 0, n); }
static inline void bcopy(const void *src, void *dst, size_t n) { memmove(dst, src, n); }

#endif
