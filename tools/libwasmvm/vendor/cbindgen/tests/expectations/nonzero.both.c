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

typedef struct Option_i64 Option_i64;

typedef struct NonZeroTest {
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
} NonZeroTest;

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
