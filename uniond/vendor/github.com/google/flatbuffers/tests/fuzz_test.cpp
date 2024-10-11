#include "fuzz_test.h"

#include <algorithm>

#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {
namespace {

// Include simple random number generator to ensure results will be the
// same cross platform.
// http://en.wikipedia.org/wiki/Park%E2%80%93Miller_random_number_generator
uint32_t lcg_seed = 48271;
uint32_t lcg_rand() {
  return lcg_seed =
             (static_cast<uint64_t>(lcg_seed) * 279470273UL) % 4294967291UL;
}
void lcg_reset() { lcg_seed = 48271; }

template<typename T>
static void CompareTableFieldValue(flatbuffers::Table *table,
                                   flatbuffers::voffset_t voffset, T val) {
  T read = table->GetField(voffset, static_cast<T>(0));
  TEST_EQ(read, val);
}

}  // namespace

// Low level stress/fuzz test: serialize/deserialize a variety of
// different kinds of data in different combinations
void FuzzTest1() {
  // Values we're testing against: chosen to ensure no bits get chopped
  // off anywhere, and also be different from eachother.
  const uint8_t bool_val = true;
  const int8_t char_val = -127;  // 0x81
  const uint8_t uchar_val = 0xFF;
  const int16_t short_val = -32222;  // 0x8222;
  const uint16_t ushort_val = 0xFEEE;
  const int32_t int_val = 0x83333333;
  const uint32_t uint_val = 0xFDDDDDDD;
  const int64_t long_val = 0x8444444444444444LL;
  const uint64_t ulong_val = 0xFCCCCCCCCCCCCCCCULL;
  const float float_val = 3.14159f;
  const double double_val = 3.14159265359;

  const int test_values_max = 11;
  const flatbuffers::voffset_t fields_per_object = 4;
  const int num_fuzz_objects = 10000;  // The higher, the more thorough :)

  flatbuffers::FlatBufferBuilder builder;

  lcg_reset();  // Keep it deterministic.

  flatbuffers::uoffset_t objects[num_fuzz_objects];

  // Generate num_fuzz_objects random objects each consisting of
  // fields_per_object fields, each of a random type.
  for (int i = 0; i < num_fuzz_objects; i++) {
    auto start = builder.StartTable();
    for (flatbuffers::voffset_t f = 0; f < fields_per_object; f++) {
      int choice = lcg_rand() % test_values_max;
      auto off = flatbuffers::FieldIndexToOffset(f);
      switch (choice) {
        case 0: builder.AddElement<uint8_t>(off, bool_val, 0); break;
        case 1: builder.AddElement<int8_t>(off, char_val, 0); break;
        case 2: builder.AddElement<uint8_t>(off, uchar_val, 0); break;
        case 3: builder.AddElement<int16_t>(off, short_val, 0); break;
        case 4: builder.AddElement<uint16_t>(off, ushort_val, 0); break;
        case 5: builder.AddElement<int32_t>(off, int_val, 0); break;
        case 6: builder.AddElement<uint32_t>(off, uint_val, 0); break;
        case 7: builder.AddElement<int64_t>(off, long_val, 0); break;
        case 8: builder.AddElement<uint64_t>(off, ulong_val, 0); break;
        case 9: builder.AddElement<float>(off, float_val, 0); break;
        case 10: builder.AddElement<double>(off, double_val, 0); break;
      }
    }
    objects[i] = builder.EndTable(start);
  }
  builder.PreAlign<flatbuffers::largest_scalar_t>(0);  // Align whole buffer.

  lcg_reset();  // Reset.

  uint8_t *eob = builder.GetCurrentBufferPointer() + builder.GetSize();

  // Test that all objects we generated are readable and return the
  // expected values. We generate random objects in the same order
  // so this is deterministic.
  for (int i = 0; i < num_fuzz_objects; i++) {
    auto table = reinterpret_cast<flatbuffers::Table *>(eob - objects[i]);
    for (flatbuffers::voffset_t f = 0; f < fields_per_object; f++) {
      int choice = lcg_rand() % test_values_max;
      flatbuffers::voffset_t off = flatbuffers::FieldIndexToOffset(f);
      switch (choice) {
        case 0: CompareTableFieldValue(table, off, bool_val); break;
        case 1: CompareTableFieldValue(table, off, char_val); break;
        case 2: CompareTableFieldValue(table, off, uchar_val); break;
        case 3: CompareTableFieldValue(table, off, short_val); break;
        case 4: CompareTableFieldValue(table, off, ushort_val); break;
        case 5: CompareTableFieldValue(table, off, int_val); break;
        case 6: CompareTableFieldValue(table, off, uint_val); break;
        case 7: CompareTableFieldValue(table, off, long_val); break;
        case 8: CompareTableFieldValue(table, off, ulong_val); break;
        case 9: CompareTableFieldValue(table, off, float_val); break;
        case 10: CompareTableFieldValue(table, off, double_val); break;
      }
    }
  }
}

// High level stress/fuzz test: generate a big schema and
// matching json data in random combinations, then parse both,
// generate json back from the binary, and compare with the original.
void FuzzTest2() {
  lcg_reset();  // Keep it deterministic.

  const int num_definitions = 30;
  const int num_struct_definitions = 5;  // Subset of num_definitions.
  const int fields_per_definition = 15;
  const int instances_per_definition = 5;
  const int deprecation_rate = 10;  // 1 in deprecation_rate fields will
                                    // be deprecated.

  std::string schema = "namespace test;\n\n";

  struct RndDef {
    std::string instances[instances_per_definition];

    // Since we're generating schema and corresponding data in tandem,
    // this convenience function adds strings to both at once.
    static void Add(RndDef (&definitions_l)[num_definitions],
                    std::string &schema_l, const int instances_per_definition_l,
                    const char *schema_add, const char *instance_add,
                    int definition) {
      schema_l += schema_add;
      for (int i = 0; i < instances_per_definition_l; i++)
        definitions_l[definition].instances[i] += instance_add;
    }
  };

  // clang-format off
  #define AddToSchemaAndInstances(schema_add, instance_add) \
    RndDef::Add(definitions, schema, instances_per_definition, \
                schema_add, instance_add, definition)

  #define Dummy() \
    RndDef::Add(definitions, schema, instances_per_definition, \
                "byte", "1", definition)
  // clang-format on

  RndDef definitions[num_definitions];

  // We are going to generate num_definitions, the first
  // num_struct_definitions will be structs, the rest tables. For each
  // generate random fields, some of which may be struct/table types
  // referring to previously generated structs/tables.
  // Simultanenously, we generate instances_per_definition JSON data
  // definitions, which will have identical structure to the schema
  // being generated. We generate multiple instances such that when creating
  // hierarchy, we get some variety by picking one randomly.
  for (int definition = 0; definition < num_definitions; definition++) {
    std::string definition_name = "D" + flatbuffers::NumToString(definition);

    bool is_struct = definition < num_struct_definitions;

    AddToSchemaAndInstances(
        ((is_struct ? "struct " : "table ") + definition_name + " {\n").c_str(),
        "{\n");

    for (int field = 0; field < fields_per_definition; field++) {
      const bool is_last_field = field == fields_per_definition - 1;

      // Deprecate 1 in deprecation_rate fields. Only table fields can be
      // deprecated.
      // Don't deprecate the last field to avoid dangling commas in JSON.
      const bool deprecated =
          !is_struct && !is_last_field && (lcg_rand() % deprecation_rate == 0);

      std::string field_name = "f" + flatbuffers::NumToString(field);
      AddToSchemaAndInstances(("  " + field_name + ":").c_str(),
                              deprecated ? "" : (field_name + ": ").c_str());
      // Pick random type:
      auto base_type = static_cast<flatbuffers::BaseType>(
          lcg_rand() % (flatbuffers::BASE_TYPE_UNION + 1));
      switch (base_type) {
        case flatbuffers::BASE_TYPE_STRING:
          if (is_struct) {
            Dummy();  // No strings in structs.
          } else {
            AddToSchemaAndInstances("string", deprecated ? "" : "\"hi\"");
          }
          break;
        case flatbuffers::BASE_TYPE_VECTOR:
          if (is_struct) {
            Dummy();  // No vectors in structs.
          } else {
            AddToSchemaAndInstances("[ubyte]",
                                    deprecated ? "" : "[\n0,\n1,\n255\n]");
          }
          break;
        case flatbuffers::BASE_TYPE_NONE:
        case flatbuffers::BASE_TYPE_UTYPE:
        case flatbuffers::BASE_TYPE_STRUCT:
        case flatbuffers::BASE_TYPE_UNION:
          if (definition) {
            // Pick a random previous definition and random data instance of
            // that definition.
            int defref = lcg_rand() % definition;
            int instance = lcg_rand() % instances_per_definition;
            AddToSchemaAndInstances(
                ("D" + flatbuffers::NumToString(defref)).c_str(),
                deprecated ? ""
                           : definitions[defref].instances[instance].c_str());
          } else {
            // If this is the first definition, we have no definition we can
            // refer to.
            Dummy();
          }
          break;
        case flatbuffers::BASE_TYPE_BOOL:
          AddToSchemaAndInstances(
              "bool", deprecated ? "" : (lcg_rand() % 2 ? "true" : "false"));
          break;
        case flatbuffers::BASE_TYPE_ARRAY:
          if (!is_struct) {
            AddToSchemaAndInstances(
                "ubyte",
                deprecated ? "" : "255");  // No fixed-length arrays in tables.
          } else {
            AddToSchemaAndInstances("[int:3]", deprecated ? "" : "[\n,\n,\n]");
          }
          break;
        default:
          // All the scalar types.
          schema += flatbuffers::TypeName(base_type);

          if (!deprecated) {
            // We want each instance to use its own random value.
            for (int inst = 0; inst < instances_per_definition; inst++)
              definitions[definition].instances[inst] +=
                  flatbuffers::IsFloat(base_type)
                      ? flatbuffers::NumToString<double>(lcg_rand() % 128)
                            .c_str()
                      : flatbuffers::NumToString<int>(lcg_rand() % 128).c_str();
          }
      }
      AddToSchemaAndInstances(deprecated ? "(deprecated);\n" : ";\n",
                              deprecated      ? ""
                              : is_last_field ? "\n"
                                              : ",\n");
    }
    AddToSchemaAndInstances("}\n\n", "}");
  }

  schema += "root_type D" + flatbuffers::NumToString(num_definitions - 1);
  schema += ";\n";

  flatbuffers::Parser parser;

  // Will not compare against the original if we don't write defaults
  parser.builder_.ForceDefaults(true);

  // Parse the schema, parse the generated data, then generate text back
  // from the binary and compare against the original.
  TEST_EQ(parser.Parse(schema.c_str()), true);

  const std::string &json =
      definitions[num_definitions - 1].instances[0] + "\n";

  TEST_EQ(parser.Parse(json.c_str()), true);

  std::string jsongen;
  parser.opts.indent_step = 0;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_NULL(result);

  if (jsongen != json) {
    // These strings are larger than a megabyte, so we show the bytes around
    // the first bytes that are different rather than the whole string.
    size_t len = std::min(json.length(), jsongen.length());
    for (size_t i = 0; i < len; i++) {
      if (json[i] != jsongen[i]) {
        i -= std::min(static_cast<size_t>(10), i);  // show some context;
        size_t end = std::min(len, i + 20);
        for (; i < end; i++)
          TEST_OUTPUT_LINE("at %d: found \"%c\", expected \"%c\"\n",
                           static_cast<int>(i), jsongen[i], json[i]);
        break;
      }
    }
    TEST_NOTNULL(nullptr);  //-V501 (this comment suppresses CWE-570 warning)
  }

  // clang-format off
  #ifdef FLATBUFFERS_TEST_VERBOSE
    TEST_OUTPUT_LINE("%dk schema tested with %dk of json\n",
                     static_cast<int>(schema.length() / 1024),
                     static_cast<int>(json.length() / 1024));
  #endif
  // clang-format on
}

}  // namespace tests
}  // namespace flatbuffers
