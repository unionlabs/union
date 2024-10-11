#include "optional_scalars_test.h"

#include <string>
#include <vector>

#include "flatbuffers/idl.h"
#include "optional_scalars_generated.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

void OptionalScalarsTest() {
  // Simple schemas and a "has optional scalar" sentinal.
  std::vector<std::string> schemas;
  schemas.push_back("table Monster { mana : int; }");
  schemas.push_back("table Monster { mana : int = 42; }");
  schemas.push_back("table Monster { mana : int =  null; }");
  schemas.push_back("table Monster { mana : long; }");
  schemas.push_back("table Monster { mana : long = 42; }");
  schemas.push_back("table Monster { mana : long = null; }");
  schemas.push_back("table Monster { mana : float; }");
  schemas.push_back("table Monster { mana : float = 42; }");
  schemas.push_back("table Monster { mana : float = null; }");
  schemas.push_back("table Monster { mana : double; }");
  schemas.push_back("table Monster { mana : double = 42; }");
  schemas.push_back("table Monster { mana : double = null; }");
  schemas.push_back("table Monster { mana : bool; }");
  schemas.push_back("table Monster { mana : bool = 42; }");
  schemas.push_back("table Monster { mana : bool = null; }");
  schemas.push_back(
      "enum Enum: int {A=0, B=1} "
      "table Monster { mana : Enum; }");
  schemas.push_back(
      "enum Enum: int {A=0, B=1} "
      "table Monster { mana : Enum = B; }");
  schemas.push_back(
      "enum Enum: int {A=0, B=1} "
      "table Monster { mana : Enum = null; }");

  // Check the FieldDef is correctly set.
  for (auto schema = schemas.begin(); schema < schemas.end(); schema++) {
    const bool has_null = schema->find("null") != std::string::npos;
    flatbuffers::Parser parser;
    TEST_ASSERT(parser.Parse(schema->c_str()));
    const auto *mana = parser.structs_.Lookup("Monster")->fields.Lookup("mana");
    TEST_EQ(mana->IsOptional(), has_null);
  }

  // Test if nullable scalars are allowed for each language.
  for (unsigned lang = 1; lang < flatbuffers::IDLOptions::kMAX; lang <<= 1) {
    flatbuffers::IDLOptions opts;
    opts.lang_to_generate = lang;
    if (false == flatbuffers::Parser::SupportsOptionalScalars(opts)) {
      continue;
    }
    for (auto schema = schemas.begin(); schema < schemas.end(); schema++) {
      flatbuffers::Parser parser(opts);
      auto done = parser.Parse(schema->c_str());
      TEST_EQ_STR(parser.error_.c_str(), "");
      TEST_ASSERT(done);
    }
  }

  // test C++ nullable
  flatbuffers::FlatBufferBuilder fbb;
  FinishScalarStuffBuffer(
      fbb, optional_scalars::CreateScalarStuff(fbb, 1, static_cast<int8_t>(2)));
  auto opts = optional_scalars::GetMutableScalarStuff(fbb.GetBufferPointer());
  TEST_ASSERT(!opts->maybe_bool());
  TEST_ASSERT(!opts->maybe_f32().has_value());
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 2);
  TEST_ASSERT(opts->mutate_maybe_i8(3));
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 3);
  TEST_ASSERT(!opts->mutate_maybe_i16(-10));

  optional_scalars::ScalarStuffT obj;
  TEST_ASSERT(!obj.maybe_bool);
  TEST_ASSERT(!obj.maybe_f32.has_value());
  opts->UnPackTo(&obj);
  TEST_ASSERT(!obj.maybe_bool);
  TEST_ASSERT(!obj.maybe_f32.has_value());
  TEST_ASSERT(obj.maybe_i8.has_value() && obj.maybe_i8.value() == 3);
  TEST_ASSERT(obj.maybe_i8 && *obj.maybe_i8 == 3);
  obj.maybe_i32 = -1;
  obj.maybe_enum = optional_scalars::OptionalByte_Two;

  fbb.Clear();
  FinishScalarStuffBuffer(fbb, optional_scalars::ScalarStuff::Pack(fbb, &obj));
  opts = optional_scalars::GetMutableScalarStuff(fbb.GetBufferPointer());
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 3);
  TEST_ASSERT(opts->maybe_i32().has_value());
  TEST_EQ(opts->maybe_i32().value(), -1);
  TEST_EQ(opts->maybe_enum().value(), optional_scalars::OptionalByte_Two);
  TEST_ASSERT(opts->maybe_i32() == flatbuffers::Optional<int64_t>(-1));
}

}  // namespace tests
}  // namespace flatbuffers
