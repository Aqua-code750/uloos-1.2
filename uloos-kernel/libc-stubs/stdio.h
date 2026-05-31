// UloOS bare-metal libc stub: stdio.h
#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>
#include <stdarg.h>

typedef void FILE;

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

#define EOF (-1)
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

int printf(const char *fmt, ...);
int fprintf(FILE *stream, const char *fmt, ...);
int sprintf(char *buf, const char *fmt, ...);
int snprintf(char *buf, size_t n, const char *fmt, ...);
int sscanf(const char *buf, const char *fmt, ...);
int puts(const char *s);
int vprintf(const char *fmt, va_list ap);
int vfprintf(FILE *stream, const char *fmt, va_list ap);
int vsprintf(char *buf, const char *fmt, va_list ap);
int vsnprintf(char *buf, size_t n, const char *fmt, va_list ap);

FILE *fopen(const char *path, const char *mode);
int fclose(FILE *f);
size_t fread(void *buf, size_t size, size_t count, FILE *f);
size_t fwrite(const void *buf, size_t size, size_t count, FILE *f);
int fseek(FILE *f, long offset, int whence);
long ftell(FILE *f);
int fflush(FILE *f);
int feof(FILE *f);
int ferror(FILE *f);
char *fgets(char *buf, int n, FILE *f);
int fgetc(FILE *f);
int fputc(int c, FILE *f);
int fputs(const char *s, FILE *f);
int remove(const char *path);
int rename(const char *old, const char *new_name);
void rewind(FILE *f);
int ungetc(int c, FILE *f);

#define getchar() fgetc(stdin)
#define putchar(c) fputc(c, stdout)

#endif
