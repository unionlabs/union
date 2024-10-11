#include "benchmarks/cpp/raw/raw_bench.h"

#include <cstdint>
#include <cstring>
#include <memory>

#include "benchmarks/cpp/bench.h"

namespace {
const int64_t kStringLength = 32;
const int64_t kVectorLength = 3;

enum Enum { Apples, Pears, Bananas };

struct Foo {
  int64_t id;
  short count;
  char prefix;
  int length;
};

struct Bar {
  Foo parent;
  int time;
  float ratio;
  unsigned short size;
};

struct FooBar {
  Bar sibling;
  // We have to stick this in, otherwise strlen() will make it slower than
  // FlatBuffers:
  int name_len;
  char name[kStringLength];
  double rating;
  unsigned char postfix;
};

struct FooBarContainer {
  FooBar list[kVectorLength];  // 3 copies of the above
  bool initialized;
  Enum fruit;
  int location_len;
  char location[kStringLength];
};

struct RawBench : Bench {
  uint8_t *Encode(void *buf, int64_t &len) override {
    FooBarContainer *fbc = new (buf) FooBarContainer;
    strcpy(fbc->location, "http://google.com/flatbuffers/");  // Unsafe eek!
    fbc->location_len = (int)strlen(fbc->location);
    fbc->fruit = Bananas;
    fbc->initialized = true;
    for (int i = 0; i < kVectorLength; i++) {
      // We add + i to not make these identical copies for a more realistic
      // compression test.
      auto &foobar = fbc->list[i];
      foobar.rating = 3.1415432432445543543 + i;
      foobar.postfix = '!' + i;
      strcpy(foobar.name, "Hello, World!");
      foobar.name_len = (int)strlen(foobar.name);
      auto &bar = foobar.sibling;
      bar.ratio = 3.14159f + i;
      bar.size = 10000 + i;
      bar.time = 123456 + i;
      auto &foo = bar.parent;
      foo.id = 0xABADCAFEABADCAFE + i;
      foo.count = 10000 + i;
      foo.length = 1000000 + i;
      foo.prefix = '@' + i;
    }

    len = sizeof(FooBarContainer);
    return reinterpret_cast<uint8_t *>(fbc);
  };

  int64_t Use(void *decoded) override {
    auto foobarcontainer = reinterpret_cast<FooBarContainer *>(decoded);
    sum = 0;
    Add(foobarcontainer->initialized);
    Add(foobarcontainer->location_len);
    Add(foobarcontainer->fruit);
    for (unsigned int i = 0; i < kVectorLength; i++) {
      auto foobar = &foobarcontainer->list[i];
      Add(foobar->name_len);
      Add(foobar->postfix);
      Add(static_cast<int64_t>(foobar->rating));
      auto bar = &foobar->sibling;
      Add(static_cast<int64_t>(bar->ratio));
      Add(bar->size);
      Add(bar->time);
      auto &foo = bar->parent;
      Add(foo.count);
      Add(foo.id);
      Add(foo.length);
      Add(foo.prefix);
    }
    return sum;
  }

  void *Decode(void *buf, int64_t) override { return buf; }
  void Dealloc(void *) override{};
};

}  // namespace

std::unique_ptr<Bench> NewRawBench() {
  return std::unique_ptr<RawBench>(new RawBench());
}
