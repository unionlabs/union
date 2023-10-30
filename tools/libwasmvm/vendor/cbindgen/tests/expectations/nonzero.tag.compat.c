#if 0
''' '
#endif

#ifdef __cplusplus
struct NonZeroI64;
#endif

#if 0
' '''
#endif


#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

struct Option_i64;

struct NonZeroTest {
  uint8_t a;
  uint16_t b;
  uint32_t c;
  uint64_t d;
  int8_t e;
  int16_t f;
  int32_t g;
  int64_t h;
  int64_t i;
  const struct Option_i64 *j;
};

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

void root(struct NonZeroTest test,
          uint8_t a,
          uint16_t b,
          uint32_t c,
          uint64_t d,
          int8_t e,
          int16_t f,
          int32_t g,
          int64_t h,
          int64_t i,
          const struct Option_i64 *j);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
