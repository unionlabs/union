#include "benchmarks/cpp/flatbuffers/fb_bench.h"

#include <cstdint>
#include <memory>

#include "benchmarks/cpp/bench.h"
#include "benchmarks/cpp/flatbuffers/bench_generated.h"
#include "flatbuffers/flatbuffers.h"

using namespace flatbuffers;
using namespace benchmarks_flatbuffers;

namespace {

struct FlatBufferBench : Bench {
  explicit FlatBufferBench(int64_t initial_size, Allocator *allocator)
      : fbb(initial_size, allocator, false) {}

  uint8_t *Encode(void *, int64_t &len) override {
    fbb.Clear();

    const int kVectorLength = 3;
    Offset<FooBar> vec[kVectorLength];

    for (int i = 0; i < kVectorLength; ++i) {
      Foo foo(0xABADCAFEABADCAFE + i, 10000 + i, '@' + i, 1000000 + i);
      Bar bar(foo, 123456 + i, 3.14159f + i, 10000 + i);
      auto name = fbb.CreateString("Hello, World!");
      auto foobar =
          CreateFooBar(fbb, &bar, name, 3.1415432432445543543 + i, '!' + i);
      vec[i] = foobar;
    }
    auto location = fbb.CreateString("http://google.com/flatbuffers/");
    auto foobarvec = fbb.CreateVector(vec, kVectorLength);
    auto foobarcontainer =
        CreateFooBarContainer(fbb, foobarvec, true, Enum_Bananas, location);
    fbb.Finish(foobarcontainer);

    len = fbb.GetSize();
    return fbb.GetBufferPointer();
  }

  int64_t Use(void *decoded) override {
    sum = 0;
    auto foobarcontainer = GetFooBarContainer(decoded);
    sum = 0;
    Add(foobarcontainer->initialized());
    Add(foobarcontainer->location()->Length());
    Add(foobarcontainer->fruit());
    for (unsigned int i = 0; i < foobarcontainer->list()->Length(); i++) {
      auto foobar = foobarcontainer->list()->Get(i);
      Add(foobar->name()->Length());
      Add(foobar->postfix());
      Add(static_cast<int64_t>(foobar->rating()));
      auto bar = foobar->sibling();
      Add(static_cast<int64_t>(bar->ratio()));
      Add(bar->size());
      Add(bar->time());
      auto &foo = bar->parent();
      Add(foo.count());
      Add(foo.id());
      Add(foo.length());
      Add(foo.prefix());
    }
    return sum;
  }

  void *Decode(void *buffer, int64_t) override { return buffer; }
  void Dealloc(void *) override {};

  FlatBufferBuilder fbb;
};

}  // namespace

std::unique_ptr<Bench> NewFlatBuffersBench(int64_t initial_size,
                                           Allocator *allocator) {
  return std::unique_ptr<FlatBufferBench>(
      new FlatBufferBench(initial_size, allocator));
}
