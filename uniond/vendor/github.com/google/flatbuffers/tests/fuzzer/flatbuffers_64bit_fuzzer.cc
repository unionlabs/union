#include <cstdint>
#include <filesystem>
#include <type_traits>

#include "64bit/test_64bit_bfbs_generated.h"
#include "64bit/test_64bit_generated.h"
#include "flatbuffers/base.h"
#include "flatbuffers/flatbuffer_builder.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/reflection.h"
#include "flatbuffers/verifier.h"
#include "test_assert.h"
#include "test_init.h"

OneTimeTestInit OneTimeTestInit::one_time_init_;

static RootTableBinarySchema schema;

static constexpr uint8_t flags_sized_prefixed = 0b00000001;

static const uint64_t kFnvPrime = 0x00000100000001b3ULL;
static const uint64_t kOffsetBasis = 0xcbf29ce484222645ULL;

namespace flatbuffers {

template<typename T, typename = std::enable_if_t<std::is_integral_v<T>>>
uint64_t Hash(T value, uint64_t hash) {
  return (hash * kFnvPrime) ^ value;
}

uint64_t Hash(double value, uint64_t hash) {
  static_assert(sizeof(double) == sizeof(uint64_t));
  return (hash * kFnvPrime) ^ static_cast<uint64_t>(value);
}

uint64_t Hash(const flatbuffers::String *value, uint64_t hash) {
  if (value == nullptr) { return hash * kFnvPrime; }
  for (auto &c : value->str()) { hash = Hash(static_cast<uint8_t>(c), hash); }
  return hash;
}

uint64_t Hash(const LeafStruct *value, uint64_t hash) {
  if (value == nullptr) { return hash * kFnvPrime; }
  hash = Hash(value->a(), hash);
  hash = Hash(value->b(), hash);
  return hash;
}

template<typename T> uint64_t Hash(const Vector<T> *value, uint64_t hash) {
  if (value == nullptr) { return hash * kFnvPrime; }
  for (const T c : *value) { hash = Hash(c, hash); }
  return hash;
}

template<typename T> uint64_t Hash(const Vector64<T> *value, uint64_t hash) {
  if (value == nullptr) { return hash * kFnvPrime; }
  for (const T c : *value) { hash = Hash(c, hash); }
  return hash;
}

uint64_t Hash(const RootTable *value, uint64_t hash) {
  if (value == nullptr) { return hash * kFnvPrime; }
  // Hash all the fields so we can exercise all parts of the code.
  hash = Hash(value->far_vector(), hash);
  hash = Hash(value->a(), hash);
  hash = Hash(value->far_string(), hash);
  hash = Hash(value->big_vector(), hash);
  hash = Hash(value->near_string(), hash);
  hash = Hash(value->nested_root(), hash);
  hash = Hash(value->far_struct_vector(), hash);
  hash = Hash(value->big_struct_vector(), hash);
  return hash;
}

static int AccessBuffer(const uint8_t *data, size_t size,
                        bool is_size_prefixed) {
  const RootTable *root_table =
      is_size_prefixed ? GetSizePrefixedRootTable(data) : GetRootTable(data);
  TEST_NOTNULL(root_table);

  uint64_t hash = kOffsetBasis;
  hash = Hash(root_table, hash);
  hash = Hash(root_table->nested_root_nested_root(), hash);

  return 0;
}

extern "C" int LLVMFuzzerInitialize(int *, char ***argv) {
  Verifier verifier(schema.begin(), schema.size());
  TEST_EQ(true, reflection::VerifySchemaBuffer(verifier));

  return 0;
}

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  if (size < FLATBUFFERS_MIN_BUFFER_SIZE) { return 0; }

  // Take the first bit of data as a flag to control things.
  const uint8_t flags = data[0];
  data++;
  size--;

  Verifier::Options options;
  options.assert = true;
  options.check_alignment = true;
  options.check_nested_flatbuffers = true;

  Verifier verifier(data, size, options);

  const bool is_size_prefixed = flags & flags_sized_prefixed;

  // Filter out data that isn't valid.
  if ((is_size_prefixed && !VerifySizePrefixedRootTableBuffer(verifier)) ||
      !VerifyRootTableBuffer(verifier)) {
    return 0;
  }

  return AccessBuffer(data, size, is_size_prefixed);
}

}  // namespace flatbuffers