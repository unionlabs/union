/*
 * Copyright 2014 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// This is a sandbox for modeling C++17 code generator.
// C++17 code generator: "flatc --cpp-std c++17".
// Warning:
// This is an experimental feature and could change at any time.

#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/flex_flat_util.h"
#include "flatbuffers/flexbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/minireflect.h"
#include "flatbuffers/registry.h"
#include "flatbuffers/util.h"
#include "stringify_util.h"
#include "test_assert.h"

// Embed generated code into an isolated namespace.
namespace cpp17 {
#include "generated_cpp17/monster_test_generated.h"
#include "generated_cpp17/optional_scalars_generated.h"
#include "generated_cpp17/union_vector_generated.h"
}  // namespace cpp17

namespace cpp11 {
#include "../monster_test_generated.h"
#include "../optional_scalars_generated.h"
}  // namespace cpp11

using ::cpp17::MyGame::Example::Monster;
using ::cpp17::MyGame::Example::Vec3;

/*******************************************************************************
** Build some FB objects.
*******************************************************************************/
namespace {
const Monster *BuildMonster(flatbuffers::FlatBufferBuilder &fbb) {
  using ::cpp17::MyGame::Example::Color;
  using ::cpp17::MyGame::Example::MonsterBuilder;
  using ::cpp17::MyGame::Example::Test;
  auto name = fbb.CreateString("my_monster");
  auto inventory = fbb.CreateVector(std::vector<uint8_t>{ 4, 5, 6, 7 });
  MonsterBuilder builder(fbb);
  auto vec3 = Vec3{ /*x=*/1.1f,
                    /*y=*/2.2f,
                    /*z=*/3.3f,
                    /*test1=*/6.6,
                    /*test2=*/Color::Green,
                    /*test3=*/
                    Test(
                        /*a=*/11,
                        /*b=*/90) };
  builder.add_pos(&vec3);
  builder.add_name(name);
  builder.add_mana(1);
  builder.add_hp(2);
  builder.add_testbool(true);
  builder.add_testhashs32_fnv1(4);
  builder.add_testhashu32_fnv1(5);
  builder.add_testhashs64_fnv1(6);
  builder.add_testhashu64_fnv1(7);
  builder.add_testhashs32_fnv1a(8);
  builder.add_testhashu32_fnv1a(9);
  builder.add_testhashs64_fnv1a(10);
  builder.add_testhashu64_fnv1a(11);
  builder.add_testf(12.1f);
  builder.add_testf2(13.1f);
  builder.add_testf3(14.1f);
  builder.add_single_weak_reference(15);
  builder.add_co_owning_reference(16);
  builder.add_non_owning_reference(17);
  builder.add_inventory(inventory);
  fbb.Finish(builder.Finish());
  const Monster *monster =
      flatbuffers::GetRoot<Monster>(fbb.GetBufferPointer());
  return monster;
}

/*******************************************************************************
** Test Case: Static Field Reflection Traits for Table & Structs.
*******************************************************************************/
// This test tests & demonstrates the power of the static reflection. Using it,
// we can given any Flatbuffer type to a generic function and it will be able to
// produce is full recursive string representation of it.
//
// This test covers all types: primitive types, structs, tables, Vectors, etc.
//
void StringifyAnyFlatbuffersTypeTest() {
  flatbuffers::FlatBufferBuilder fbb;
  // We are using a Monster here, but we could have used any type, because the
  // code that follows is totally generic!
  const auto *monster = BuildMonster(fbb);

  std::string expected = R"(MyGame.Example.Monster{
        pos = MyGame.Example.Vec3{
          x = 1.1
          y = 2.2
          z = 3.3
          test1 = 6.6
          test2 = 2
          test3 = MyGame.Example.Test{
            a = 11
            b = 90
          }
        }
        mana = 1
        hp = 2
        name = "my_monster"
        inventory = [
          4,
          5,
          6,
          7
        ]
        color = 8
        test_type = 0
        testbool = 1
        testhashs32_fnv1 = 4
        testhashu32_fnv1 = 5
        testhashs64_fnv1 = 6
        testhashu64_fnv1 = 7
        testhashs32_fnv1a = 8
        testhashu32_fnv1a = 9
        testhashs64_fnv1a = 10
        testhashu64_fnv1a = 11
        testf = 12.1
        testf2 = 13.1
        testf3 = 14.1
        single_weak_reference = 15
        co_owning_reference = 16
        non_owning_reference = 17
        any_unique_type = 0
        any_ambiguous_type = 0
        signed_enum = -1
        long_enum_non_enum_default = 0
        long_enum_normal_default = 2
        nan_default = nan
        inf_default = inf
        positive_inf_default = inf
        infinity_default = inf
        positive_infinity_default = inf
        negative_inf_default = -inf
        negative_infinity_default = -inf
        double_inf_default = inf
      })";

  // Call a generic function that has no specific knowledge of the flatbuffer we
  // are passing in; it should use only static reflection to produce a string
  // representations of the field names and values recursively. We give it an
  // initial indentation so that the result can be compared with our raw string
  // above, which we wanted to indent so that it will look nicer in this code.
  //
  // A note about JSON: as can be seen from the string above, this produces a
  // JSON-like notation, but we are not using any of Flatbuffers' JSON infra to
  // produce this! It is produced entirely using compile-time reflection, and
  // thus does not require any runtime access to the *.fbs definition files!
  std::optional<std::string> result =
      cpp17::StringifyFlatbufferValue(*monster, /*indent=*/"      ");

  TEST_ASSERT(result.has_value());
  TEST_EQ_STR(expected.c_str(), result->c_str());
}

/*******************************************************************************
** Test Traits::FieldType
*******************************************************************************/
using pos_type = Monster::Traits::FieldType<0>;
static_assert(std::is_same_v<pos_type, const Vec3 *>);

using mana_type = Monster::Traits::FieldType<1>;
static_assert(std::is_same_v<mana_type, int16_t>);

using name_type = Monster::Traits::FieldType<3>;
static_assert(std::is_same_v<name_type, const flatbuffers::String *>);

/*******************************************************************************
** Generic Create Function Test.
*******************************************************************************/
void CreateTableByTypeTest() {
  flatbuffers::FlatBufferBuilder builder;

  // We will create an object of this type using only the type.
  using type_to_create_t = cpp17::MyGame::Example::Stat;

  [&builder] {
    auto id_str = builder.CreateString("my_id");
    auto table = type_to_create_t::Traits::Create(builder, id_str, 42, 7);
    // Be sure that the correct return type was inferred.
    static_assert(
        std::is_same_v<decltype(table), flatbuffers::Offset<type_to_create_t>>);
    builder.Finish(table);
  }();

  // Access it.
  auto stat =
      flatbuffers::GetRoot<type_to_create_t>(builder.GetBufferPointer());
  TEST_EQ_STR(stat->id()->c_str(), "my_id");
  TEST_EQ(stat->val(), 42);
  TEST_EQ(stat->count(), 7);
}

void OptionalScalarsTest() {
  static_assert(
      std::is_same<flatbuffers::Optional<float>, std::optional<float>>::value);
  static_assert(std::is_same<flatbuffers::nullopt_t, std::nullopt_t>::value);

  // test C++ nullable
  flatbuffers::FlatBufferBuilder fbb;
  FinishScalarStuffBuffer(fbb, cpp17::optional_scalars::CreateScalarStuff(
                                   fbb, 1, static_cast<int8_t>(2)));
  auto opts =
      cpp17::optional_scalars::GetMutableScalarStuff(fbb.GetBufferPointer());
  TEST_ASSERT(!opts->maybe_bool());
  TEST_ASSERT(!opts->maybe_f32().has_value());
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 2);
  TEST_ASSERT(opts->mutate_maybe_i8(3));
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 3);
  TEST_ASSERT(!opts->mutate_maybe_i16(-10));

  cpp17::optional_scalars::ScalarStuffT obj;
  opts->UnPackTo(&obj);
  TEST_ASSERT(!obj.maybe_bool);
  TEST_ASSERT(!obj.maybe_f32.has_value());
  TEST_ASSERT(obj.maybe_i8.has_value() && obj.maybe_i8.value() == 3);
  TEST_ASSERT(obj.maybe_i8 && *obj.maybe_i8 == 3);
  obj.maybe_i32 = -1;

  fbb.Clear();
  FinishScalarStuffBuffer(
      fbb, cpp17::optional_scalars::ScalarStuff::Pack(fbb, &obj));
  opts = cpp17::optional_scalars::GetMutableScalarStuff(fbb.GetBufferPointer());
  TEST_ASSERT(opts->maybe_i8().has_value());
  TEST_EQ(opts->maybe_i8().value(), 3);
  TEST_ASSERT(opts->maybe_i32().has_value());
  TEST_EQ(opts->maybe_i32().value(), -1);

  TEST_EQ(std::optional<int32_t>(opts->maybe_i32()).value(), -1);
  TEST_EQ(std::optional<int64_t>(opts->maybe_i32()).value(), -1);
  TEST_ASSERT(opts->maybe_i32() == std::optional<int64_t>(-1));
}

int FlatBufferCpp17Tests() {
  CreateTableByTypeTest();
  OptionalScalarsTest();
  StringifyAnyFlatbuffersTypeTest();
  return 0;
}
}  // namespace

int main(int /*argc*/, const char * /*argv*/[]) {
  InitTestEngine();

  FlatBufferCpp17Tests();

  if (!testing_fails) {
    TEST_OUTPUT_LINE("C++17: ALL TESTS PASSED");
  } else {
    TEST_OUTPUT_LINE("C++17: %d FAILED TESTS", testing_fails);
  }
  return CloseTestEngine();
}
