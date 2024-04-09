#include <windows.h>

#if defined(_MSC_VER)
/*
 * Even though we don't have memcpy/memset anywhere, MSVC compiler
 * generates calls to them as it recognizes corresponding patterns.
 */
void *memcpy(unsigned char *dst, const unsigned char *src, size_t n)
{
    void *ret = dst;

    while(n--)
        *dst++ = *src++;

    return ret;
}

void *memset(unsigned char *dst, int c, size_t n)
{
    void *ret = dst;

    while(n--)
        *dst++ = (unsigned char)c;

    return ret;
}
#elif defined(__GNUC__)
# pragma GCC diagnostic ignored "-Wunused-parameter"
#endif

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved)
{   return TRUE;   }
