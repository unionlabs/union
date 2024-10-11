#include <benchmark/benchmark.h>
#include <gtest/gtest.h>

#include "benchmarks/cpp/bench.h"
#include "benchmarks/cpp/flatbuffers/fb_bench.h"
#include "benchmarks/cpp/raw/raw_bench.h"

static inline void Encode(benchmark::State &state,
                          std::unique_ptr<Bench> &bench, uint8_t *buffer) {
  int64_t length;
  for (auto _ : state) {
    bench->Encode(buffer, length);
    benchmark::DoNotOptimize(length);
  }
}

static inline void Decode(benchmark::State &state,
                          std::unique_ptr<Bench> &bench, uint8_t *buffer) {
  int64_t length;
  uint8_t *encoded = bench->Encode(buffer, length);

  for (auto _ : state) {
    void *decoded = bench->Decode(encoded, length);
    benchmark::DoNotOptimize(decoded);
  }
}

static inline void Use(benchmark::State &state, std::unique_ptr<Bench> &bench,
                       uint8_t *buffer, int64_t check_sum) {
  int64_t length;
  uint8_t *encoded = bench->Encode(buffer, length);
  void *decoded = bench->Decode(encoded, length);

  int64_t sum = 0;

  for (auto _ : state) { sum = bench->Use(decoded); }

  EXPECT_EQ(sum, check_sum);
}

static void BM_Flatbuffers_Encode(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  StaticAllocator allocator(&buffer[0]);
  std::unique_ptr<Bench> bench = NewFlatBuffersBench(kBufferLength, &allocator);
  Encode(state, bench, buffer);
}
BENCHMARK(BM_Flatbuffers_Encode);

static void BM_Flatbuffers_Decode(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  StaticAllocator allocator(&buffer[0]);
  std::unique_ptr<Bench> bench = NewFlatBuffersBench(kBufferLength, &allocator);
  Decode(state, bench, buffer);
}
BENCHMARK(BM_Flatbuffers_Decode);

static void BM_Flatbuffers_Use(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  StaticAllocator allocator(&buffer[0]);
  std::unique_ptr<Bench> bench = NewFlatBuffersBench(kBufferLength, &allocator);
  Use(state, bench, buffer, 218812692406581874);
}
BENCHMARK(BM_Flatbuffers_Use);

static void BM_Raw_Encode(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  std::unique_ptr<Bench> bench = NewRawBench();
  Encode(state, bench, buffer);
}
BENCHMARK(BM_Raw_Encode);

static void BM_Raw_Decode(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  std::unique_ptr<Bench> bench = NewRawBench();
  Decode(state, bench, buffer);
}
BENCHMARK(BM_Raw_Decode);

static void BM_Raw_Use(benchmark::State &state) {
  const int64_t kBufferLength = 1024;
  uint8_t buffer[kBufferLength];

  std::unique_ptr<Bench> bench = NewRawBench();
  Use(state, bench, buffer, 218812692406581874);
}
BENCHMARK(BM_Raw_Use);
