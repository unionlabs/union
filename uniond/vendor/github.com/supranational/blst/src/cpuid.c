/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#if (defined(__GNUC__) || defined(__clang__) || defined(__SUNPRO_C)) && !defined(_WIN32)
__attribute__((visibility("hidden")))
#endif
int __blst_platform_cap = 0;

#if defined(__x86_64__) || defined(__x86_64) || (defined(_M_X64) && !defined(_M_ARM64EC))

# if defined(__GNUC__) || defined(__clang__) || defined(__SUNPRO_C)
static void __cpuidex(int info[4], int func, int sub)
{
    int eax, ebx, ecx, edx;

    __asm__("cpuid" : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
                    : "a"(func), "c"(sub));

    info[0] = eax;
    info[1] = ebx;
    info[2] = ecx;
    info[3] = edx;
}
# else
#  include <intrin.h>
# endif

# if defined(__GNUC__) || defined(__clang__)
__attribute__((constructor))
# endif
static int __blst_cpuid(void)
{
    int info[4], cap = 0;

    __cpuidex(info, 0, 0);
    if (info[0] > 6) {
        __cpuidex(info, 7, 0);
        cap |= (info[1]>>19) & 1; /* ADX */
        cap |= (info[1]>>28) & 2; /* SHA */
    }

    __blst_platform_cap = cap;

    return 0;
}

# if defined(_MSC_VER) && !defined(__clang__)
#  pragma section(".CRT$XCU",read)
__declspec(allocate(".CRT$XCU")) static int (*p)(void) = __blst_cpuid;
# elif defined(__SUNPRO_C)
#  pragma init(__blst_cpuid)
# endif

#elif defined(__aarch64__) || defined(__aarch64) || defined(_M_ARM64)

# if defined(__linux__) && (defined(__GNUC__) || defined(__clang__))
extern unsigned long getauxval(unsigned long type) __attribute__ ((weak));

__attribute__((constructor))
static int __blst_cpuid(void)
{
    int cap = 0;

    if (getauxval) {
        unsigned long hwcap_ce = getauxval(16);
        cap = (hwcap_ce>>6) & 1; /* SHA256 */
    }

    __blst_platform_cap = cap;

    return 0;
}
# elif defined(__APPLE__) && (defined(__GNUC__) || defined(__clang__))
__attribute__((constructor))
static int __blst_cpuid()
{
    __blst_platform_cap = 1; /* SHA256 */
    return 0;
}
# elif defined(__FreeBSD__) && __FreeBSD__ >= 12
#  include <sys/auxv.h>
__attribute__((constructor))
static int __blst_cpuid()
{
    unsigned long cap;

    if (elf_aux_info(AT_HWCAP, &cap, sizeof(cap)) == 0)
        __blst_platform_cap = (cap & HWCAP_SHA2) != 0;

    return 0;
}
# elif defined(_WIN64)
int IsProcessorFeaturePresent(int);

#  if defined(__GNUC__) || defined(__clang__)
__attribute__((constructor))
#  endif
static int __blst_cpuid(void)
{
    __blst_platform_cap = IsProcessorFeaturePresent(30); /* AES, SHA1, SHA2 */

    return 0;
}

#  if defined(_MSC_VER) && !defined(__clang__)
#   pragma section(".CRT$XCU",read)
__declspec(allocate(".CRT$XCU")) static int (*p)(void) = __blst_cpuid;
#  endif
# endif

#endif
