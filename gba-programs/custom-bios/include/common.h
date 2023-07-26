#ifndef COMMON_H
#define COMMON_H

#define FILE_FOO_SEEN
#include <stddef.h>

#ifdef __GNUC__
#  define UNUSED(x) UNUSED_ ## x __attribute__((__unused__))
#else
#  define UNUSED(x) UNUSED_ ## x
#endif

#ifdef __GNUC__
#  define UNUSED_FUNCTION(x) __attribute__((__unused__)) UNUSED_ ## x
#else
#  define UNUSED_FUNCTION(x) UNUSED_ ## x
#endif

#ifdef __GNUC__
#   define NO_RETURN __attribute__((noreturn))
#endif

void* ep_memset(void *dest, int val, size_t len);
void* ep_memmove(void *dest, const void *src, size_t len);
void* ep_memmove(void *dest, const void *src, size_t len);

#define SWI_RETURN() asm volatile ("movs pc, lr" : /* NO OUTPUTS*/ : /* NO INPUTS */: /* NO CLOBBERS */);

#endif /* COMMON_H */