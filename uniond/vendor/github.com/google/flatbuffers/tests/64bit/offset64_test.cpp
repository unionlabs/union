#include "offset64_test.h"

#include <stdint.h>

#include <cstdint>
#include <fstream>
#include <limits>
#include <ostream>

#include "flatbuffers/base.h"
#include "flatbuffers/buffer.h"
#include "flatbuffers/flatbuffer_builder.h"
#include "flatbuffers/flatbuffers.h"
#include "tests/64bit/evolution/v1_generated.h"
#include "tests/64bit/evolution/v2_generated.h"
#include "tests/64bit/test_64bit_generated.h"
#include "tests/test_assert.h"

namespace flatbuffers {
namespace tests {

void Offset64Test() {
  FlatBufferBuilder64 builder;

  const size_t far_vector_size = 1LL << 2;
  // Make a large number if wanting to test a real large buffer.
  const size_t big_vector_size = 1LL << 2;

  {
    // First create the vectors that will be copied to the buffer.
    std::vector<uint8_t> far_data;
    far_data.resize(far_vector_size);
    far_data[0] = 4;
    far_data[far_vector_size - 1] = 2;

    std::vector<uint8_t> big_data;
    big_data.resize(big_vector_size);
    big_data[0] = 8;
    big_data[big_vector_size - 1] = 3;

    // Then serialize all the fields that have 64-bit offsets, as these must be
    // serialized before any 32-bit fields are added to the buffer.
    const Offset64<Vector<uint8_t>> far_vector_offset =
        builder.CreateVector64<Vector>(far_data);

    const Offset64<String> far_string_offset =
        builder.CreateString<Offset64>("some far string");

    const Offset64<Vector64<uint8_t>> big_vector_offset =
        builder.CreateVector64(big_data);

    // Now that we are done with the 64-bit fields, we can create and add the
    // normal fields.
    const Offset<String> near_string_offset =
        builder.CreateString("some near string");

    // Finish by building the root table by passing in all the offsets.
    const Offset<RootTable> root_table_offset =
        CreateRootTable(builder, far_vector_offset, 0, far_string_offset,
                        big_vector_offset, near_string_offset);

    // Finish the buffer.
    builder.Finish(root_table_offset);

    Verifier::Options options;
    // Allow the verifier to verify 64-bit buffers.
    options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
    options.assert = true;

    Verifier verifier(builder.GetBufferPointer(), builder.GetSize(), options);

    TEST_EQ(VerifyRootTableBuffer(verifier), true);
  }

  {
    const RootTable *root_table = GetRootTable(builder.GetBufferPointer());

    // Expect the far vector to be properly sized.
    TEST_EQ(root_table->far_vector()->size(), far_vector_size);
    TEST_EQ(root_table->far_vector()->Get(0), 4);
    TEST_EQ(root_table->far_vector()->Get(far_vector_size - 1), 2);

    TEST_EQ_STR(root_table->far_string()->c_str(), "some far string");

    // Expect the big vector to be properly sized.
    TEST_EQ(root_table->big_vector()->size(), big_vector_size);
    TEST_EQ(root_table->big_vector()->Get(0), 8);
    TEST_EQ(root_table->big_vector()->Get(big_vector_size - 1), 3);

    TEST_EQ_STR(root_table->near_string()->c_str(), "some near string");
  }
}

void Offset64SerializedFirst() {
  FlatBufferBuilder64 fbb;

  // First create the vectors that will be copied to the buffer.
  std::vector<uint8_t> data;
  data.resize(64);

  // Then serialize all the fields that have 64-bit offsets, as these must be
  // serialized before any 32-bit fields are added to the buffer.
  fbb.CreateVector64(data);

  // TODO(derekbailey): figure out how to test assertions.
  // Uncommenting this line should fail the test with an assertion.
  // fbb.CreateString("some near string");

  fbb.CreateVector64(data);
}

void Offset64NestedFlatBuffer() {
  FlatBufferBuilder64 fbb;

  // First serialize a nested buffer.
  const Offset<String> near_string_offset =
      fbb.CreateString("nested: some near string");

  // Finish by building the root table by passing in all the offsets.
  const Offset<RootTable> root_table_offset =
      CreateRootTable(fbb, 0, 0, 0, 0, near_string_offset, 0);

  // Finish the buffer.
  fbb.Finish(root_table_offset);

  // Ensure the buffer is valid.
  const RootTable *root_table = GetRootTable(fbb.GetBufferPointer());
  TEST_EQ_STR(root_table->near_string()->c_str(), "nested: some near string");

  // Copy the data out of the builder.
  std::vector<uint8_t> nested_data{ fbb.GetBufferPointer(),
                                    fbb.GetBufferPointer() + fbb.GetSize() };

  {
    // Clear so we can reuse the builder.
    fbb.Clear();

    const Offset64<Vector64<uint8_t>> nested_flatbuffer_offset =
        fbb.CreateVector64<Vector64>(nested_data);

    // Now that we are done with the 64-bit fields, we can create and add the
    // normal fields.
    const Offset<String> near_string_offset =
        fbb.CreateString("some near string");

    // Finish by building the root table by passing in all the offsets.
    const Offset<RootTable> root_table_offset = CreateRootTable(
        fbb, 0, 0, 0, 0, near_string_offset, nested_flatbuffer_offset);

    // Finish the buffer.
    fbb.Finish(root_table_offset);

    Verifier::Options options;
    // Allow the verifier to verify 64-bit buffers.
    options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
    options.assert = true;

    Verifier verifier(fbb.GetBufferPointer(), fbb.GetSize(), options);

    TEST_EQ(VerifyRootTableBuffer(verifier), true);
  }

  {
    const RootTable *root_table = GetRootTable(fbb.GetBufferPointer());

    // Test that the parent buffer field is ok.
    TEST_EQ_STR(root_table->near_string()->c_str(), "some near string");

    // Expect nested buffer to be properly sized.
    TEST_EQ(root_table->nested_root()->size(), nested_data.size());

    // Expect the direct accessors to the nested buffer work.
    TEST_EQ_STR(root_table->nested_root_nested_root()->near_string()->c_str(),
                "nested: some near string");
  }
}

void Offset64CreateDirect() {
  FlatBufferBuilder64 fbb;

  // Create a vector of some data
  std::vector<uint8_t> data{ 0, 1, 2 };

  // Call the "Direct" creation method to ensure that things are added to the
  // buffer in the correct order, Offset64 first followed by any Offsets.
  const Offset<RootTable> root_table_offset = CreateRootTableDirect(
      fbb, &data, 0, "some far string", &data, "some near string");

  // Finish the buffer.
  fbb.Finish(root_table_offset);

  Verifier::Options options;
  // Allow the verifier to verify 64-bit buffers.
  options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
  options.assert = true;

  Verifier verifier(fbb.GetBufferPointer(), fbb.GetSize(), options);

  TEST_EQ(VerifyRootTableBuffer(verifier), true);

  // Verify the data.
  const RootTable *root_table = GetRootTable(fbb.GetBufferPointer());
  TEST_EQ(root_table->far_vector()->size(), data.size());
  TEST_EQ(root_table->big_vector()->size(), data.size());
  TEST_EQ_STR(root_table->far_string()->c_str(), "some far string");
  TEST_EQ_STR(root_table->near_string()->c_str(), "some near string");
}

void Offset64Evolution() {
  // Some common data for the tests.
  const std::vector<uint8_t> data = { 1, 2, 3, 4 };
  const std::vector<uint8_t> big_data = { 6, 7, 8, 9, 10 };

  // Built V1 read V2
  {
    // Use the 32-bit builder since V1 doesn't have any 64-bit offsets.
    FlatBufferBuilder builder;

    builder.Finish(v1::CreateRootTableDirect(builder, 1234, &data));

    // Use each version to get a view at the root table.
    auto v1_root = v1::GetRootTable(builder.GetBufferPointer());
    auto v2_root = v2::GetRootTable(builder.GetBufferPointer());

    // Test field equivalents for fields common to V1 and V2.
    TEST_EQ(v1_root->a(), v2_root->a());

    TEST_EQ(v1_root->b(), v2_root->b());
    TEST_EQ(v1_root->b()->Get(2), 3);
    TEST_EQ(v2_root->b()->Get(2), 3);

    // This field is added in V2, so it should be null since V1 couldn't have
    // written it.
    TEST_ASSERT(v2_root->big_vector() == nullptr);
  }

  // Built V2 read V1
  {
    // Use the 64-bit builder since V2 has 64-bit offsets.
    FlatBufferBuilder64 builder;

    builder.Finish(v2::CreateRootTableDirect(builder, 1234, &data, &big_data));

    // Use each version to get a view at the root table.
    auto v1_root = v1::GetRootTable(builder.GetBufferPointer());
    auto v2_root = v2::GetRootTable(builder.GetBufferPointer());

    // Test field equivalents for fields common to V1 and V2.
    TEST_EQ(v1_root->a(), v2_root->a());

    TEST_EQ(v1_root->b(), v2_root->b());
    TEST_EQ(v1_root->b()->Get(2), 3);
    TEST_EQ(v2_root->b()->Get(2), 3);

    // Test that V2 can read the big vector, which V1 doesn't even have
    // accessors for (i.e. v1_root->big_vector() doesn't exist).
    TEST_ASSERT(v2_root->big_vector() != nullptr);
    TEST_EQ(v2_root->big_vector()->size(), big_data.size());
    TEST_EQ(v2_root->big_vector()->Get(2), 8);
  }

  // Built V2 read V1, bigger than max 32-bit buffer sized.
  // This checks that even a large buffer can still be read by V1.
  {
    // Use the 64-bit builder since V2 has 64-bit offsets.
    FlatBufferBuilder64 builder;

    std::vector<uint8_t> giant_data;
    giant_data.resize(1LL << 3);
    giant_data[2] = 42;

    builder.Finish(
        v2::CreateRootTableDirect(builder, 1234, &data, &giant_data));

    // Use each version to get a view at the root table.
    auto v1_root = v1::GetRootTable(builder.GetBufferPointer());
    auto v2_root = v2::GetRootTable(builder.GetBufferPointer());

    // Test field equivalents for fields common to V1 and V2.
    TEST_EQ(v1_root->a(), v2_root->a());

    TEST_EQ(v1_root->b(), v2_root->b());
    TEST_EQ(v1_root->b()->Get(2), 3);
    TEST_EQ(v2_root->b()->Get(2), 3);

    // Test that V2 can read the big vector, which V1 doesn't even have
    // accessors for (i.e. v1_root->big_vector() doesn't exist).
    TEST_ASSERT(v2_root->big_vector() != nullptr);
    TEST_EQ(v2_root->big_vector()->size(), giant_data.size());
    TEST_EQ(v2_root->big_vector()->Get(2), 42);
  }
}

void Offset64VectorOfStructs() {
  FlatBufferBuilder64 builder;

  std::vector<LeafStruct> far_leaves;
  far_leaves.emplace_back(LeafStruct{ 123, 4.567 });
  far_leaves.emplace_back(LeafStruct{ 987, 6.543 });

  std::vector<LeafStruct> big_leaves;
  big_leaves.emplace_back(LeafStruct{ 72, 72.8 });
  big_leaves.emplace_back(LeafStruct{ 82, 82.8 });
  big_leaves.emplace_back(LeafStruct{ 92, 92.8 });

  // Add the two vectors of leaf structs.
  const Offset<RootTable> root_table_offset =
      CreateRootTableDirect(builder, nullptr, 0, nullptr, nullptr, nullptr,
                            nullptr, &far_leaves, &big_leaves);

  // Finish the buffer.
  builder.Finish(root_table_offset);

  Verifier::Options options;
  // Allow the verifier to verify 64-bit buffers.
  options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
  options.assert = true;

  Verifier verifier(builder.GetBufferPointer(), builder.GetSize(), options);

  TEST_EQ(VerifyRootTableBuffer(verifier), true);

  // Verify the data.
  const RootTable *root_table = GetRootTable(builder.GetBufferPointer());
  TEST_EQ(root_table->far_struct_vector()->size(), far_leaves.size());
  TEST_EQ(root_table->far_struct_vector()->Get(0)->a(), 123);
  TEST_EQ(root_table->far_struct_vector()->Get(0)->b(), 4.567);
  TEST_EQ(root_table->far_struct_vector()->Get(1)->a(), 987);
  TEST_EQ(root_table->far_struct_vector()->Get(1)->b(), 6.543);

  TEST_EQ(root_table->big_struct_vector()->size(), big_leaves.size());
  TEST_EQ(root_table->big_struct_vector()->Get(0)->a(), 72);
  TEST_EQ(root_table->big_struct_vector()->Get(0)->b(), 72.8);
  TEST_EQ(root_table->big_struct_vector()->Get(1)->a(), 82);
  TEST_EQ(root_table->big_struct_vector()->Get(1)->b(), 82.8);
  TEST_EQ(root_table->big_struct_vector()->Get(2)->a(), 92);
  TEST_EQ(root_table->big_struct_vector()->Get(2)->b(), 92.8);
}

void Offset64SizePrefix() {
  FlatBufferBuilder64 builder;

  // First serialize a nested buffer.
  const Offset<String> near_string_offset =
      builder.CreateString("some near string");

  // Finish by building the root table by passing in all the offsets.
  const Offset<RootTable> root_table_offset =
      CreateRootTable(builder, 0, 0, 0, 0, near_string_offset, 0);

  // Finish the buffer.
  FinishSizePrefixedRootTableBuffer(builder, root_table_offset);

  TEST_EQ(GetPrefixedSize<uoffset64_t>(builder.GetBufferPointer()),
          builder.GetSize() - sizeof(uoffset64_t));

  Verifier::Options options;
  // Allow the verifier to verify 64-bit buffers.
  options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
  options.assert = true;

  Verifier verifier(builder.GetBufferPointer(), builder.GetSize(), options);

  TEST_EQ(VerifySizePrefixedRootTableBuffer(verifier), true);

  const RootTable *root_table =
      GetSizePrefixedRootTable(builder.GetBufferPointer());

  // Verify the fields.
  TEST_EQ_STR(root_table->near_string()->c_str(), "some near string");
}

void Offset64ManyVectors() {
  FlatBufferBuilder64 builder;

  // Setup some data to serialize.
  std::vector<int8_t> data;
  data.resize(20);
  data.front() = 42;
  data.back() = 18;

  const size_t kNumVectors = 20;

  // First serialize all the 64-bit address vectors. We need to store all the
  // offsets to later add to a wrapper table. We cannot serialize one vector and
  // then add it to a table immediately, as it would violate the strict ordering
  // of putting all 64-bit things at the tail of the buffer.
  std::array<Offset64<Vector<int8_t>>, kNumVectors> offsets_64bit;
  for (size_t i = 0; i < kNumVectors; ++i) {
    offsets_64bit[i] = builder.CreateVector64<Vector>(data);
  }

  // Create some unrelated, 64-bit offset value for later testing.
  const Offset64<String> far_string_offset =
      builder.CreateString<Offset64>("some far string");

  // Now place all the offsets into their own wrapper tables. Again, we have to
  // store the offsets before we can add them to the root table vector.
  std::array<Offset<WrapperTable>, kNumVectors> offsets_wrapper;
  for (size_t i = 0; i < kNumVectors; ++i) {
    offsets_wrapper[i] = CreateWrapperTable(builder, offsets_64bit[i]);
  }

  // Now create the 32-bit vector that is stored in the root table.
  // TODO(derekbailey): the array type wasn't auto deduced, see if that could be
  // fixed.
  const Offset<Vector<Offset<WrapperTable>>> many_vectors_offset =
      builder.CreateVector<Offset<WrapperTable>>(offsets_wrapper);

  // Finish by building using the root table builder, to exercise a different
  // code path than the other tests.
  RootTableBuilder root_table_builder(builder);
  root_table_builder.add_many_vectors(many_vectors_offset);
  root_table_builder.add_far_string(far_string_offset);
  const Offset<RootTable> root_table_offset = root_table_builder.Finish();

  // Finish the buffer.
  FinishRootTableBuffer(builder, root_table_offset);

  Verifier::Options options;
  // Allow the verifier to verify 64-bit buffers.
  options.max_size = FLATBUFFERS_MAX_64_BUFFER_SIZE;
  options.assert = true;

  Verifier verifier(builder.GetBufferPointer(), builder.GetSize(), options);

  TEST_EQ(VerifyRootTableBuffer(verifier), true);

  const RootTable *root_table = GetRootTable(builder.GetBufferPointer());

  // Verify the fields.
  TEST_EQ_STR(root_table->far_string()->c_str(), "some far string");
  TEST_EQ(root_table->many_vectors()->size(), kNumVectors);

  // Spot check one of the vectors.
  TEST_EQ(root_table->many_vectors()->Get(12)->vector()->size(), 20);
  TEST_EQ(root_table->many_vectors()->Get(12)->vector()->Get(0), 42);
  TEST_EQ(root_table->many_vectors()->Get(12)->vector()->Get(19), 18);
}

void Offset64ForceAlign() {
  FlatBufferBuilder64 builder;

  // Setup some data to serialize that is less than the force_align size of 32
  // bytes.
  std::vector<uint8_t> data{ 1, 2, 3 };

  // Use the CreateDirect which calls the ForceVectorAlign
  const auto root_table_offset =
      CreateRootTableDirect(builder, nullptr, 0, nullptr, nullptr, nullptr,
                            nullptr, nullptr, nullptr, nullptr, &data);

  // Finish the buffer.
  FinishRootTableBuffer(builder, root_table_offset);
}

}  // namespace tests
}  // namespace flatbuffers
