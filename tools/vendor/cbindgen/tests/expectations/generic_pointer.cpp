#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

template<typename T>
struct Foo {
  T a;
};

using Boo = Foo<uint8_t*>;

extern "C" {

void root(Boo x);

} // extern "C"
