// UloOS bare-metal libc stub: stddef.h
#ifndef _STDDEF_H
#define _STDDEF_H

typedef unsigned long long size_t;
typedef long long ssize_t;
typedef long long ptrdiff_t;
typedef int wchar_t;

#define NULL ((void*)0)
#define offsetof(type, member) __builtin_offsetof(type, member)

#endif
