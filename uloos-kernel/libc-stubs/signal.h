// UloOS bare-metal libc stub: signal.h
#ifndef _SIGNAL_H
#define _SIGNAL_H

#define SIGINT 2
#define SIGTERM 15

typedef void (*sighandler_t)(int);
sighandler_t signal(int sig, sighandler_t handler);

#endif
