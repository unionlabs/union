#include "evolution_test.h"

#include "evolution_test/evolution_v1_generated.h"
#include "evolution_test/evolution_v2_generated.h"
#include "flatbuffers/idl.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

void EvolutionTest(const std::string &tests_data_path) {
  // VS10 does not support typed enums, exclude from tests
#if !defined(_MSC_VER) || _MSC_VER >= 1700
  const int NUM_VERSIONS = 2;
  std::string schemas[NUM_VERSIONS];
  std::string jsonfiles[NUM_VERSIONS];
  std::vector<uint8_t> binaries[NUM_VERSIONS];

  flatbuffers::IDLOptions idl_opts;
  idl_opts.lang_to_generate |= flatbuffers::IDLOptions::kBinary;
  flatbuffers::Parser parser(idl_opts);

  // Load all the schema versions and their associated data.
  for (int i = 0; i < NUM_VERSIONS; ++i) {
    std::string schema = tests_data_path + "evolution_test/evolution_v" +
                         flatbuffers::NumToString(i + 1) + ".fbs";
    TEST_ASSERT(flatbuffers::LoadFile(schema.c_str(), false, &schemas[i]));
    std::string json = tests_data_path + "evolution_test/evolution_v" +
                       flatbuffers::NumToString(i + 1) + ".json";
    TEST_ASSERT(flatbuffers::LoadFile(json.c_str(), false, &jsonfiles[i]));

    TEST_ASSERT(parser.Parse(schemas[i].c_str()));
    TEST_ASSERT(parser.Parse(jsonfiles[i].c_str()));

    auto bufLen = parser.builder_.GetSize();
    auto buf = parser.builder_.GetBufferPointer();
    binaries[i].reserve(bufLen);
    std::copy(buf, buf + bufLen, std::back_inserter(binaries[i]));
  }

  // Assert that all the verifiers for the different schema versions properly
  // verify any version data.
  for (int i = 0; i < NUM_VERSIONS; ++i) {
    flatbuffers::Verifier verifier(&binaries[i].front(), binaries[i].size());
    TEST_ASSERT(Evolution::V1::VerifyRootBuffer(verifier));
    TEST_ASSERT(Evolution::V2::VerifyRootBuffer(verifier));
  }

  // Test backwards compatibility by reading old data with an evolved schema.
  auto root_v1_viewed_from_v2 = Evolution::V2::GetRoot(&binaries[0].front());
  // field 'k' is new in version 2, so it should be null.
  TEST_ASSERT(nullptr == root_v1_viewed_from_v2->k());
  // field 'l' is new in version 2 with a default of 56.
  TEST_EQ(root_v1_viewed_from_v2->l(), 56);
  // field 'c' of 'TableA' is new in version 2, so it should be null.
  TEST_ASSERT(nullptr == root_v1_viewed_from_v2->e()->c());
  // 'TableC' was added to field 'c' union in version 2, so it should be null.
  TEST_ASSERT(nullptr == root_v1_viewed_from_v2->c_as_TableC());
  // The field 'c' union should be of type 'TableB' regardless of schema version
  TEST_ASSERT(root_v1_viewed_from_v2->c_type() == Evolution::V2::Union::TableB);
  // The field 'f' was renamed to 'ff' in version 2, it should still be
  // readable.
  TEST_EQ(root_v1_viewed_from_v2->ff()->a(), 16);

  // Test forwards compatibility by reading new data with an old schema.
  auto root_v2_viewed_from_v1 = Evolution::V1::GetRoot(&binaries[1].front());
  // The field 'c' union in version 2 is a new table (index = 3) and should
  // still be accessible, but not interpretable.
  TEST_EQ(static_cast<uint8_t>(root_v2_viewed_from_v1->c_type()), 3);
  TEST_NOTNULL(root_v2_viewed_from_v1->c());
  // The field 'd' enum in verison 2 has new members and should still be
  // accessible, but not interpretable.
  TEST_EQ(static_cast<int8_t>(root_v2_viewed_from_v1->d()), 3);
  // The field 'a' in version 2 is deprecated and should return the default
  // value (0) instead of the value stored in the in the buffer (42).
  TEST_EQ(root_v2_viewed_from_v1->a(), 0);
  // The field 'ff' was originally named 'f' in version 1, it should still be
  // readable.
  TEST_EQ(root_v2_viewed_from_v1->f()->a(), 35);
#endif
}

void ConformTest() {
  const char ref[] = "table T { A:int; } enum E:byte { A }";

  auto test_conform = [](const char *ref, const char *test,
                         const char *expected_err) {
    flatbuffers::Parser parser1;
    TEST_EQ(parser1.Parse(ref), true);
    flatbuffers::Parser parser2;
    TEST_EQ(parser2.Parse(test), true);
    auto err = parser2.ConformTo(parser1);
    if (*expected_err == '\0') {
      TEST_EQ_STR(err.c_str(), expected_err);
    } else {
      TEST_NOTNULL(strstr(err.c_str(), expected_err));
    }
  };

  test_conform(ref, "table T { A:byte; }", "types differ for field: T.A");
  test_conform(ref, "table T { B:int; A:int; }",
               "offsets differ for field: T.A");
  test_conform(ref, "table T { A:int = 1; }", "defaults differ for field: T.A");
  test_conform(ref, "table T { B:float; }",
               "field renamed to different type: T.B (renamed from T.A)");
  test_conform(ref, "enum E:byte { B, A }", "values differ for enum: A");
  test_conform(ref, "table T { }", "field deleted: T.A");
  test_conform(ref, "table T { B:int; }", "");  // renaming a field is allowed

  const char ref2[] = "enum E:byte { A } table T2 { f:E; } ";
  test_conform(ref2, "enum E:int32 { A } table T2 { df:byte; f:E; }",
               "field renamed to different type: T2.df (renamed from T2.f)");

  // Check enum underlying type changes.
  test_conform("enum E:int32 {A}", "enum E: byte {A}", "underlying type differ for enum: E");
  
  // Check union underlying type changes.
  const char ref3[] = "table A {} table B {} union C {A, B}";
  test_conform(ref3, "table A {} table B {} union C:int32 {A, B}", "underlying type differ for union: C");

  // Check conformity for Offset64-related changes.
  {
    const char ref[] = "table T { a:[uint8]; b:string; }";

    // Adding a 'vector64' changes the type.
    test_conform(ref, "table T { a:[uint8] (vector64); b:string; }",
                 "types differ for field: T.a");

    // Adding a 'offset64' to the vector changes the type.
    test_conform(ref, "table T { a:[uint8] (offset64); b:string; }",
                 "offset types differ for field: T.a");

    // Adding a 'offset64' to the string also changes the type.
    test_conform(ref, "table T { a:[uint8]; b:string (offset64); }",
                 "offset types differ for field: T.b");

    // Now try the opposite direction of removing an attribute from an existing
    // field.

    // Removing a 'vector64' changes the type.
    test_conform("table T { a:[uint8] (vector64); b:string; }", ref,
                 "types differ for field: T.a");

    // Removing a 'offset64' to the string also changes the type.
    test_conform("table T { a:[uint8] (offset64); b:string; }", ref,
                 "offset types differ for field: T.a");

    // Remove a 'offset64' to the string also changes the type.
    test_conform("table T { a:[uint8]; b:string (offset64); }", ref,
                 "offset types differ for field: T.b");
  }
}

void UnionDeprecationTest(const std::string &tests_data_path) {
  const int NUM_VERSIONS = 2;
  std::string schemas[NUM_VERSIONS];
  std::string jsonfiles[NUM_VERSIONS];
  std::vector<uint8_t> binaries[NUM_VERSIONS];

  flatbuffers::IDLOptions idl_opts;
  idl_opts.lang_to_generate |= flatbuffers::IDLOptions::kBinary;
  flatbuffers::Parser parser(idl_opts);

  // Load all the schema versions and their associated data.
  for (int i = 0; i < NUM_VERSIONS; ++i) {
    std::string schema = tests_data_path + "evolution_test/evolution_v" +
                         flatbuffers::NumToString(i + 1) + ".fbs";
    TEST_ASSERT(flatbuffers::LoadFile(schema.c_str(), false, &schemas[i]));
    std::string json = tests_data_path + "evolution_test/evolution_v" +
                       flatbuffers::NumToString(i + 1) + ".json";
    TEST_ASSERT(flatbuffers::LoadFile(json.c_str(), false, &jsonfiles[i]));

    TEST_ASSERT(parser.Parse(schemas[i].c_str()));
    TEST_ASSERT(parser.Parse(jsonfiles[i].c_str()));

    auto bufLen = parser.builder_.GetSize();
    auto buf = parser.builder_.GetBufferPointer();
    binaries[i].reserve(bufLen);
    std::copy(buf, buf + bufLen, std::back_inserter(binaries[i]));
  }

  auto v2 = parser.LookupStruct("Evolution.V2.Root");
  TEST_NOTNULL(v2);
  auto j_type_field = v2->fields.Lookup("j_type");
  TEST_NOTNULL(j_type_field);
  TEST_ASSERT(j_type_field->deprecated);
}

}  // namespace tests
}  // namespace flatbuffers
