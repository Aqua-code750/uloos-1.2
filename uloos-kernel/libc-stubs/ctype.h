// UloOS bare-metal libc stub: ctype.h
#ifndef _CTYPE_H
#define _CTYPE_H

int toupper(int c);
int tolower(int c);

static inline int isalpha(int c) { return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'); }
static inline int isdigit(int c) { return c >= '0' && c <= '9'; }
static inline int isalnum(int c) { return isalpha(c) || isdigit(c); }
static inline int isspace(int c) { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v'; }
static inline int isupper(int c) { return c >= 'A' && c <= 'Z'; }
static inline int islower(int c) { return c >= 'a' && c <= 'z'; }
static inline int isprint(int c) { return c >= 0x20 && c < 0x7F; }
static inline int isxdigit(int c) { return isdigit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f'); }

#endif
