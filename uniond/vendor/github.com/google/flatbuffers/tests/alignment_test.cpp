#include "alignment_test.h"

#include "tests/alignment_test_generated.h"
#include "flatbuffers/flatbuffer_builder.h"
#include "flatbuffers/util.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

void AlignmentTest() {
  FlatBufferBuilder builder;

  BadAlignmentLarge large;
  Offset<OuterLarge> outer_large = CreateOuterLarge(builder, &large);

  BadAlignmentSmall *small;
  Offset<Vector<const BadAlignmentSmall *>> small_offset =
      builder.CreateUninitializedVectorOfStructs(9, &small);
  (void)small;  // We do not have to write data to trigger the test failure

  Offset<BadAlignmentRoot> root =
      CreateBadAlignmentRoot(builder, outer_large, small_offset);

  builder.Finish(root);

  Verifier verifier(builder.GetBufferPointer(), builder.GetSize());
  TEST_ASSERT(verifier.VerifyBuffer<BadAlignmentRoot>(nullptr));


  // ============= Test Small Structs Vector misalignment ========

  builder.Clear();

  // creating 5 structs with 2 bytes each
  // 10 bytes in total for Vector data is needed
  std::vector<JustSmallStruct> small_vector = {
    { 2, 1 }, { 3, 1 }, { 4, 1 }
  };
  // CreateVectorOfStructs is used in the generated CreateSmallStructsDirect()
  // method, but we test it directly
  Offset<Vector<const JustSmallStruct *>> small_structs_offset =
      builder.CreateVectorOfStructs<JustSmallStruct>(small_vector);
  Offset<SmallStructs> small_structs_root =
      CreateSmallStructs(builder, small_structs_offset);

  builder.Finish(small_structs_root);

  // Save the binary that we later can annotate with `flatc --annotate` command
  // NOTE: the conversion of the JSON data to --binary via `flatc --binary`
  //       command is not changed with that fix and was always producing the
  //       correct binary data.
  // SaveFile("alignment_test_{before,after}_fix.bin",
  //          reinterpret_cast<char *>(builder.GetBufferPointer()),
  //          builder.GetSize(), true);

  Verifier verifier_small_structs(builder.GetBufferPointer(),
                                  builder.GetSize());
  TEST_ASSERT(verifier_small_structs.VerifyBuffer<SmallStructs>(nullptr));

  // Reading SmallStructs vector values back and compare with original
  auto root_msg =
      flatbuffers::GetRoot<SmallStructs>(builder.GetBufferPointer());

  TEST_EQ(root_msg->small_structs()->size(), small_vector.size());
  for (flatbuffers::uoffset_t i = 0; i < root_msg->small_structs()->size();
       ++i) {
    TEST_EQ(small_vector[i].var_0(),
            root_msg->small_structs()->Get(i)->var_0());
    TEST_EQ(small_vector[i].var_1(),
            root_msg->small_structs()->Get(i)->var_1());
  }
}

}  // namespace tests
}  // namespace flatbuffers
