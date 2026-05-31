// UloOS bare-metal libc stub: stdlib.h
#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1
#define RAND_MAX 0x7FFF

void *malloc(size_t size);
void free(void *ptr);
void *calloc(size_t count, size_t size);
void *realloc(void *ptr, size_t size);

void exit(int code);
void abort(void);
int atexit(void (*func)(void));

int atoi(const char *s);
long atol(const char *s);
double atof(const char *s);
int abs(int x);
long labs(long x);

int rand(void);
void srand(unsigned int seed);

char *getenv(const char *name);
int system(const char *command);

void qsort(void *base, size_t num, size_t size, int (*compar)(const void *, const void *));

#endif
