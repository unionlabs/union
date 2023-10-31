#if 0
''' '
#endif

#ifdef __cplusplus
struct NonZeroI64;
#endif

#if 0
' '''
#endif


#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

template<typename T = void>
struct Option;

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
  const Option<int64_t> *j;
};

extern "C" {

void root(NonZeroTest test,
          uint8_t a,
          uint16_t b,
          uint32_t c,
          uint64_t d,
          int8_t e,
          int16_t f,
          int32_t g,
          int64_t h,
          int64_t i,
          const Option<int64_t> *j);

} // extern "C"
