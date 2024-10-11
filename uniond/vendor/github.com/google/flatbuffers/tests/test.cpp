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
#include <stdint.h>

#include <cmath>
#include <limits>
#include <memory>
#include <string>

#if defined(__ANDROID__)
#define INCLUDE_64_BIT_TESTS 0
#else
#define INCLUDE_64_BIT_TESTS 1
#endif

#include "alignment_test.h"
#include "evolution_test.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/minireflect.h"
#include "flatbuffers/reflection_generated.h"
#include "flatbuffers/registry.h"
#include "flatbuffers/util.h"
#include "fuzz_test.h"
#include "json_test.h"
#include "key_field_test.h"
#include "monster_test.h"
#include "monster_test_generated.h"
#include "native_inline_table_test_generated.h"
#include "optional_scalars_test.h"
#include "parser_test.h"
#include "proto_test.h"
#include "reflection_test.h"
#include "tests/union_vector/union_vector_generated.h"
#include "union_underlying_type_test_generated.h"
#if !defined(_MSC_VER) || _MSC_VER >= 1700
#  include "tests/arrays_test_generated.h"
#endif
#if INCLUDE_64_BIT_TESTS
#include "tests/64bit/offset64_test.h"
#endif
#include "flexbuffers_test.h"
#include "is_quiet_nan.h"
#include "monster_test_bfbs_generated.h"  // Generated using --bfbs-comments --bfbs-builtins --cpp --bfbs-gen-embed
#include "native_type_test_generated.h"
#include "test_assert.h"
#include "util_test.h"

void FlatBufferBuilderTest();

namespace flatbuffers {
namespace tests {
namespace {

// clang-format off
// Check that char* and uint8_t* are interoperable types.
// The reinterpret_cast<> between the pointers are used to simplify data loading.
static_assert(flatbuffers::is_same<uint8_t, char>::value ||
              flatbuffers::is_same<uint8_t, unsigned char>::value,
              "unexpected uint8_t type");

#if defined(FLATBUFFERS_HAS_NEW_STRTOD) && (FLATBUFFERS_HAS_NEW_STRTOD > 0)
  // Ensure IEEE-754 support if tests of floats with NaN/Inf will run.
  static_assert(std::numeric_limits<float>::is_iec559 &&
                std::numeric_limits<double>::is_iec559,
                "IEC-559 (IEEE-754) standard required");
#endif
// clang-format on

using namespace MyGame::Example;

void TriviallyCopyableTest() {
  // clang-format off
  #if __GNUG__ && __GNUC__ < 5 && \
      !(defined(__clang__) && __clang_major__ >= 16)
    TEST_EQ(__is_trivially_copyable(Vec3), true);
  #else
    #if __cplusplus >= 201103L
      TEST_EQ(std::is_trivially_copyable<Vec3>::value, true);
    #endif
  #endif
  // clang-format on
}

// Guard against -Wunused-function on platforms without file tests.
#ifndef FLATBUFFERS_NO_FILE_TESTS
void GenerateTableTextTest(const std::string &tests_data_path) {
  std::string schemafile;
  std::string jsonfile;
  bool ok =
      flatbuffers::LoadFile((tests_data_path + "monster_test.fbs").c_str(),
                            false, &schemafile) &&
      flatbuffers::LoadFile((tests_data_path + "monsterdata_test.json").c_str(),
                            false, &jsonfile);
  TEST_EQ(ok, true);
  auto include_test_path =
      flatbuffers::ConCatPathFileName(tests_data_path, "include_test");
  const char *include_directories[] = { tests_data_path.c_str(),
                                        include_test_path.c_str(), nullptr };
  flatbuffers::IDLOptions opt;
  opt.indent_step = -1;
  flatbuffers::Parser parser(opt);
  ok = parser.Parse(schemafile.c_str(), include_directories) &&
       parser.Parse(jsonfile.c_str(), include_directories);
  TEST_EQ(ok, true);
  // Test root table
  const Monster *monster = GetMonster(parser.builder_.GetBufferPointer());
  const auto abilities = monster->testarrayofsortedstruct();
  TEST_EQ(abilities->size(), 3);
  TEST_EQ(abilities->Get(0)->id(), 0);
  TEST_EQ(abilities->Get(0)->distance(), 45);
  TEST_EQ(abilities->Get(1)->id(), 1);
  TEST_EQ(abilities->Get(1)->distance(), 21);
  TEST_EQ(abilities->Get(2)->id(), 5);
  TEST_EQ(abilities->Get(2)->distance(), 12);

  std::string jsongen;
  auto result = GenTextFromTable(parser, monster, "MyGame.Example.Monster",
                                      &jsongen);
  TEST_NULL(result);
  // Test sub table
  const Vec3 *pos = monster->pos();
  jsongen.clear();
  result = GenTextFromTable(parser, pos, "MyGame.Example.Vec3", &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(
      jsongen.c_str(),
      "{x: 1.0,y: 2.0,z: 3.0,test1: 3.0,test2: \"Green\",test3: {a: 5,b: 6}}");
  const Test &test3 = pos->test3();
  jsongen.clear();
  result =
      GenTextFromTable(parser, &test3, "MyGame.Example.Test", &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(), "{a: 5,b: 6}");
  const Test *test4 = monster->test4()->Get(0);
  jsongen.clear();
  result =
      GenTextFromTable(parser, test4, "MyGame.Example.Test", &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(), "{a: 10,b: 20}");
}

void MultiFileNameClashTest(const std::string &tests_data_path) {
  const auto name_clash_path =
      flatbuffers::ConCatPathFileName(tests_data_path, "name_clash_test");
  const char *include_directories[] = { name_clash_path.c_str() };

  // Load valid 2 file Flatbuffer schema
  const auto valid_path =
      flatbuffers::ConCatPathFileName(name_clash_path, "valid_test1.fbs");
  std::string valid_schema;
  TEST_ASSERT(flatbuffers::LoadFile(valid_path.c_str(), false, &valid_schema));
  // Clashing table and union names in different namespaces must be parsable
  TEST_ASSERT(
      flatbuffers::Parser().Parse(valid_schema.c_str(), include_directories));

  flatbuffers::Parser p;
  TEST_ASSERT(p.Parse(valid_schema.c_str(), include_directories));

  // Load invalid 2 file Flatbuffer schema
  const auto invalid_path =
      flatbuffers::ConCatPathFileName(name_clash_path, "invalid_test1.fbs");
  std::string invalid_schema;
  TEST_ASSERT(
      flatbuffers::LoadFile(invalid_path.c_str(), false, &invalid_schema));
  // Clashing table and union names in same namespace must fail to parse
  TEST_EQ(
      flatbuffers::Parser().Parse(invalid_schema.c_str(), include_directories),
      false);
}

void InvalidNestedFlatbufferTest(const std::string &tests_data_path) {
  // First, load and parse FlatBuffer schema (.fbs)
  std::string schemafile;
  TEST_EQ(flatbuffers::LoadFile((tests_data_path + "monster_test.fbs").c_str(),
                                false, &schemafile),
          true);
  auto include_test_path =
      flatbuffers::ConCatPathFileName(tests_data_path, "include_test");
  const char *include_directories[] = { tests_data_path.c_str(),
                                        include_test_path.c_str(), nullptr };
  flatbuffers::Parser parser1;
  TEST_EQ(parser1.Parse(schemafile.c_str(), include_directories), true);

  // "color" inside nested flatbuffer contains invalid enum value
  TEST_EQ(parser1.Parse("{ name: \"Bender\", testnestedflatbuffer: { name: "
                        "\"Leela\", color: \"nonexistent\"}}"),
          false);
}

void UnionVectorTest(const std::string &tests_data_path) {
  // load FlatBuffer fbs schema and json.
  std::string schemafile, jsonfile;
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "union_vector/union_vector.fbs").c_str(),
              false, &schemafile),
          true);
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "union_vector/union_vector.json").c_str(),
              false, &jsonfile),
          true);

  // parse schema.
  flatbuffers::IDLOptions idl_opts;
  idl_opts.lang_to_generate |= flatbuffers::IDLOptions::kBinary;
  flatbuffers::Parser parser(idl_opts);
  TEST_EQ(parser.Parse(schemafile.c_str()), true);

  flatbuffers::FlatBufferBuilder fbb;

  // union types.
  std::vector<uint8_t> types;
  types.push_back(static_cast<uint8_t>(Character_Belle));
  types.push_back(static_cast<uint8_t>(Character_MuLan));
  types.push_back(static_cast<uint8_t>(Character_BookFan));
  types.push_back(static_cast<uint8_t>(Character_Other));
  types.push_back(static_cast<uint8_t>(Character_Unused));

  // union values.
  std::vector<flatbuffers::Offset<void>> characters;
  characters.push_back(fbb.CreateStruct(BookReader(/*books_read=*/7)).Union());
  characters.push_back(CreateAttacker(fbb, /*sword_attack_damage=*/5).Union());
  characters.push_back(fbb.CreateStruct(BookReader(/*books_read=*/2)).Union());
  characters.push_back(fbb.CreateString("Other").Union());
  characters.push_back(fbb.CreateString("Unused").Union());

  // create Movie.
  const auto movie_offset =
      CreateMovie(fbb, Character_Rapunzel,
                  fbb.CreateStruct(Rapunzel(/*hair_length=*/6)).Union(),
                  fbb.CreateVector(types), fbb.CreateVector(characters));
  FinishMovieBuffer(fbb, movie_offset);

  flatbuffers::Verifier verifier(fbb.GetBufferPointer(), fbb.GetSize());
  TEST_EQ(VerifyMovieBuffer(verifier), true);

  auto flat_movie = GetMovie(fbb.GetBufferPointer());

  auto TestMovie = [](const Movie *movie) {
    TEST_EQ(movie->main_character_type() == Character_Rapunzel, true);

    auto cts = movie->characters_type();
    TEST_EQ(movie->characters_type()->size(), 5);
    TEST_EQ(cts->GetEnum<Character>(0) == Character_Belle, true);
    TEST_EQ(cts->GetEnum<Character>(1) == Character_MuLan, true);
    TEST_EQ(cts->GetEnum<Character>(2) == Character_BookFan, true);
    TEST_EQ(cts->GetEnum<Character>(3) == Character_Other, true);
    TEST_EQ(cts->GetEnum<Character>(4) == Character_Unused, true);

    auto rapunzel = movie->main_character_as_Rapunzel();
    TEST_NOTNULL(rapunzel);
    TEST_EQ(rapunzel->hair_length(), 6);

    auto cs = movie->characters();
    TEST_EQ(cs->size(), 5);
    auto belle = cs->GetAs<BookReader>(0);
    TEST_EQ(belle->books_read(), 7);
    auto mu_lan = cs->GetAs<Attacker>(1);
    TEST_EQ(mu_lan->sword_attack_damage(), 5);
    auto book_fan = cs->GetAs<BookReader>(2);
    TEST_EQ(book_fan->books_read(), 2);
    auto other = cs->GetAsString(3);
    TEST_EQ_STR(other->c_str(), "Other");
    auto unused = cs->GetAsString(4);
    TEST_EQ_STR(unused->c_str(), "Unused");
  };

  TestMovie(flat_movie);

  // Also test the JSON we loaded above.
  TEST_EQ(parser.Parse(jsonfile.c_str()), true);
  auto jbuf = parser.builder_.GetBufferPointer();
  flatbuffers::Verifier jverifier(jbuf, parser.builder_.GetSize());
  TEST_EQ(VerifyMovieBuffer(jverifier), true);
  TestMovie(GetMovie(jbuf));

  auto movie_object = flat_movie->UnPack();
  TEST_EQ(movie_object->main_character.AsRapunzel()->hair_length(), 6);
  TEST_EQ(movie_object->characters[0].AsBelle()->books_read(), 7);
  TEST_EQ(movie_object->characters[1].AsMuLan()->sword_attack_damage, 5);
  TEST_EQ(movie_object->characters[2].AsBookFan()->books_read(), 2);
  TEST_EQ_STR(movie_object->characters[3].AsOther()->c_str(), "Other");
  TEST_EQ_STR(movie_object->characters[4].AsUnused()->c_str(), "Unused");

  fbb.Clear();
  fbb.Finish(Movie::Pack(fbb, movie_object));

  delete movie_object;

  auto repacked_movie = GetMovie(fbb.GetBufferPointer());

  TestMovie(repacked_movie);

  // Generate text using mini-reflection.
  auto s =
      flatbuffers::FlatBufferToString(fbb.GetBufferPointer(), MovieTypeTable());
  TEST_EQ_STR(
      s.c_str(),
      "{ main_character_type: Rapunzel, main_character: { hair_length: 6 }, "
      "characters_type: [ Belle, MuLan, BookFan, Other, Unused ], "
      "characters: [ { books_read: 7 }, { sword_attack_damage: 5 }, "
      "{ books_read: 2 }, \"Other\", \"Unused\" ] }");

  flatbuffers::ToStringVisitor visitor("\n", true, "  ");
  IterateFlatBuffer(fbb.GetBufferPointer(), MovieTypeTable(), &visitor);
  TEST_EQ_STR(visitor.s.c_str(),
              "{\n"
              "  \"main_character_type\": \"Rapunzel\",\n"
              "  \"main_character\": {\n"
              "    \"hair_length\": 6\n"
              "  },\n"
              "  \"characters_type\": [\n"
              "    \"Belle\",\n"
              "    \"MuLan\",\n"
              "    \"BookFan\",\n"
              "    \"Other\",\n"
              "    \"Unused\"\n"
              "  ],\n"
              "  \"characters\": [\n"
              "    {\n"
              "      \"books_read\": 7\n"
              "    },\n"
              "    {\n"
              "      \"sword_attack_damage\": 5\n"
              "    },\n"
              "    {\n"
              "      \"books_read\": 2\n"
              "    },\n"
              "    \"Other\",\n"
              "    \"Unused\"\n"
              "  ]\n"
              "}");

  // Generate text using parsed schema.
  std::string jsongen;
  auto result = GenText(parser, fbb.GetBufferPointer(), &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(),
              "{\n"
              "  main_character_type: \"Rapunzel\",\n"
              "  main_character: {\n"
              "    hair_length: 6\n"
              "  },\n"
              "  characters_type: [\n"
              "    \"Belle\",\n"
              "    \"MuLan\",\n"
              "    \"BookFan\",\n"
              "    \"Other\",\n"
              "    \"Unused\"\n"
              "  ],\n"
              "  characters: [\n"
              "    {\n"
              "      books_read: 7\n"
              "    },\n"
              "    {\n"
              "      sword_attack_damage: 5\n"
              "    },\n"
              "    {\n"
              "      books_read: 2\n"
              "    },\n"
              "    \"Other\",\n"
              "    \"Unused\"\n"
              "  ]\n"
              "}\n");

  // Simple test with reflection.
  parser.Serialize();
  auto schema = reflection::GetSchema(parser.builder_.GetBufferPointer());
  auto ok = flatbuffers::Verify(*schema, *schema->root_table(),
                                fbb.GetBufferPointer(), fbb.GetSize());
  TEST_EQ(ok, true);

  flatbuffers::Parser parser2(idl_opts);
  TEST_EQ(parser2.Parse("struct Bool { b:bool; }"
                        "union Any { Bool }"
                        "table Root { a:Any; }"
                        "root_type Root;"),
          true);
  TEST_EQ(parser2.Parse("{a_type:Bool,a:{b:true}}"), true);
}
#endif

void EndianSwapTest() {
  TEST_EQ(flatbuffers::EndianSwap(static_cast<int16_t>(0x1234)), 0x3412);
  TEST_EQ(flatbuffers::EndianSwap(static_cast<int32_t>(0x12345678)),
          0x78563412);
  TEST_EQ(flatbuffers::EndianSwap(static_cast<int64_t>(0x1234567890ABCDEF)),
          0xEFCDAB9078563412);
  TEST_EQ(flatbuffers::EndianSwap(flatbuffers::EndianSwap(3.14f)), 3.14f);
}

void UninitializedVectorTest() {
  flatbuffers::FlatBufferBuilder builder;

  Test *buf = nullptr;
  auto vector_offset =
      builder.CreateUninitializedVectorOfStructs<Test>(2, &buf);
  TEST_NOTNULL(buf);
  buf[0] = Test(10, 20);
  buf[1] = Test(30, 40);

  auto required_name = builder.CreateString("myMonster");
  auto monster_builder = MonsterBuilder(builder);
  monster_builder.add_name(
      required_name);  // required field mandated for monster.
  monster_builder.add_test4(vector_offset);
  builder.Finish(monster_builder.Finish());

  auto p = builder.GetBufferPointer();
  auto uvt = flatbuffers::GetRoot<Monster>(p);
  TEST_NOTNULL(uvt);
  auto vec = uvt->test4();
  TEST_NOTNULL(vec);
  auto test_0 = vec->Get(0);
  auto test_1 = vec->Get(1);
  TEST_EQ(test_0->a(), 10);
  TEST_EQ(test_0->b(), 20);
  TEST_EQ(test_1->a(), 30);
  TEST_EQ(test_1->b(), 40);
}

void EqualOperatorTest() {
  MonsterT a;
  MonsterT b;
  // We have to reset the fields that are NaN to zero to allow the equality
  // to evaluate to true.
  TEST_EQ(std::isnan(a.nan_default), true);
  TEST_EQ(std::isnan(b.nan_default), true);
  a.nan_default = 0;
  b.nan_default = 0;
  TEST_EQ(b == a, true);
  TEST_EQ(b != a, false);

  b.mana = 33;
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);
  b.mana = 150;
  TEST_EQ(b == a, true);
  TEST_EQ(b != a, false);

  b.inventory.push_back(3);
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);
  b.inventory.clear();
  TEST_EQ(b == a, true);
  TEST_EQ(b != a, false);

  a.enemy.reset(new MonsterT());
  a.enemy->nan_default = 0;
  TEST_EQ(b != a, true);
  a.enemy->mana = 33;
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);

  b.enemy.reset(new MonsterT());
  b.enemy->nan_default = 0;
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);
  b.enemy->mana = 33;
  TEST_EQ(b == a, true);
  TEST_EQ(b != a, false);

  a.enemy.reset(nullptr);
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);
  b.enemy->mana = 150;
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);
  a.enemy.reset(new MonsterT());
  a.enemy->nan_default = 0;
  TEST_EQ(b == a, true);
  TEST_EQ(b != a, false);

  b.enemy.reset(nullptr);

  b.test.type = Any_Monster;
  TEST_EQ(b == a, false);
  TEST_EQ(b != a, true);

  // Test that vector of tables are compared by value and not by reference.
  {
    // Two tables are equal by default.
    MonsterT a, b;
    a.nan_default = 0;
    b.nan_default = 0;
    TEST_EQ(a == b, true);

    // Adding only a table to one of the monster vectors should make it not
    // equal (due to size mistmatch).
    a.testarrayoftables.push_back(
        flatbuffers::unique_ptr<MonsterT>(new MonsterT));
    a.testarrayoftables.back()->nan_default = 0;
    TEST_EQ(a == b, false);

    // Adding an equalivant table to the other monster vector should make it
    // equal again.
    b.testarrayoftables.push_back(
        flatbuffers::unique_ptr<MonsterT>(new MonsterT));
    b.testarrayoftables.back()->nan_default = 0;
    TEST_EQ(a == b, true);

    // Create two new monsters that are different.
    auto c = flatbuffers::unique_ptr<MonsterT>(new MonsterT);
    auto d = flatbuffers::unique_ptr<MonsterT>(new MonsterT);
    c->nan_default = 0;
    d->nan_default = 0;
    c->hp = 1;
    d->hp = 2;
    TEST_EQ(c == d, false);

    // Adding them to the original monsters should also make them different.
    a.testarrayoftables.push_back(std::move(c));
    b.testarrayoftables.push_back(std::move(d));
    TEST_EQ(a == b, false);

    // Remove the mismatching monsters to get back to equality
    a.testarrayoftables.pop_back();
    b.testarrayoftables.pop_back();
    TEST_EQ(a == b, true);

    // Check that nullptr are OK.
    a.testarrayoftables.push_back(nullptr);
    b.testarrayoftables.push_back(
        flatbuffers::unique_ptr<MonsterT>(new MonsterT));
    TEST_EQ(a == b, false);
  }
}

void CreateSharedStringTest() {
  flatbuffers::FlatBufferBuilder builder;
  const auto one1 = builder.CreateSharedString("one");
  const auto two = builder.CreateSharedString("two");
  const auto one2 = builder.CreateSharedString("one");
  TEST_EQ(one1.o, one2.o);
  const auto onetwo = builder.CreateSharedString("onetwo");
  TEST_EQ(onetwo.o != one1.o, true);
  TEST_EQ(onetwo.o != two.o, true);

  // Support for embedded nulls
  const char chars_b[] = { 'a', '\0', 'b' };
  const char chars_c[] = { 'a', '\0', 'c' };
  const auto null_b1 = builder.CreateSharedString(chars_b, sizeof(chars_b));
  const auto null_c = builder.CreateSharedString(chars_c, sizeof(chars_c));
  const auto null_b2 = builder.CreateSharedString(chars_b, sizeof(chars_b));
  TEST_EQ(null_b1.o != null_c.o, true);  // Issue#5058 repro
  TEST_EQ(null_b1.o, null_b2.o);

  // Put the strings into an array for round trip verification.
  std::array<flatbuffers::Offset<flatbuffers::String>, 7> array = {
    one1, two, one2, onetwo, null_b1, null_c, null_b2
  };
  const auto vector_offset =
      builder.CreateVector<flatbuffers::Offset<flatbuffers::String>>(array);
  MonsterBuilder monster_builder(builder);
  monster_builder.add_name(two);
  monster_builder.add_testarrayofstring(vector_offset);
  builder.Finish(monster_builder.Finish());

  // Read the Monster back.
  const auto *monster =
      flatbuffers::GetRoot<Monster>(builder.GetBufferPointer());
  TEST_EQ_STR(monster->name()->c_str(), "two");
  const auto *testarrayofstring = monster->testarrayofstring();
  TEST_EQ(testarrayofstring->size(), flatbuffers::uoffset_t(7));
  const auto &a = *testarrayofstring;
  TEST_EQ_STR(a[0]->c_str(), "one");
  TEST_EQ_STR(a[1]->c_str(), "two");
  TEST_EQ_STR(a[2]->c_str(), "one");
  TEST_EQ_STR(a[3]->c_str(), "onetwo");
  TEST_EQ(a[4]->str(), (std::string(chars_b, sizeof(chars_b))));
  TEST_EQ(a[5]->str(), (std::string(chars_c, sizeof(chars_c))));
  TEST_EQ(a[6]->str(), (std::string(chars_b, sizeof(chars_b))));

  // Make sure String::operator< works, too, since it is related to
  // StringOffsetCompare.
  TEST_EQ((*a[0]) < (*a[1]), true);
  TEST_EQ((*a[1]) < (*a[0]), false);
  TEST_EQ((*a[1]) < (*a[2]), false);
  TEST_EQ((*a[2]) < (*a[1]), true);
  TEST_EQ((*a[4]) < (*a[3]), true);
  TEST_EQ((*a[5]) < (*a[4]), false);
  TEST_EQ((*a[5]) < (*a[4]), false);
  TEST_EQ((*a[6]) < (*a[5]), true);
}

#if !defined(FLATBUFFERS_USE_STD_SPAN) && !defined(FLATBUFFERS_SPAN_MINIMAL)
void FlatbuffersSpanTest() {
  // Compile-time checking of non-const [] to const [] conversions.
  using flatbuffers::internal::is_span_convertible;
  (void)is_span_convertible<int, 1, int, 1>::type(123);
  (void)is_span_convertible<const int, 1, int, 1>::type(123);
  (void)is_span_convertible<const int64_t, 1, int64_t, 1>::type(123);
  (void)is_span_convertible<const uint64_t, 1, uint64_t, 1>::type(123);
  (void)is_span_convertible<const int, 1, const int, 1>::type(123);
  (void)is_span_convertible<const int64_t, 1, const int64_t, 1>::type(123);
  (void)is_span_convertible<const uint64_t, 1, const uint64_t, 1>::type(123);

  using flatbuffers::span;
  span<char, 0> c1;
  TEST_EQ(c1.size(), 0);
  span<char, flatbuffers::dynamic_extent> c2;
  TEST_EQ(c2.size(), 0);
  span<char> c3;
  TEST_EQ(c3.size(), 0);
  TEST_ASSERT(c1.empty() && c2.empty() && c3.empty());

  int i_data7[7] = { 0, 1, 2, 3, 4, 5, 6 };
  span<int, 7> i1(&i_data7[0], 7);
  span<int> i2(i1);  // make dynamic from static
  TEST_EQ(i1.size(), 7);
  TEST_EQ(i1.empty(), false);
  TEST_EQ(i1.size(), i2.size());
  TEST_EQ(i1.data(), i_data7);
  TEST_EQ(i1[2], 2);
  // Make const span from a non-const one.
  span<const int, 7> i3(i1);
  // Construct from a C-array.
  span<int, 7> i4(i_data7);
  span<const int, 7> i5(i_data7);
  span<int> i6(i_data7);
  span<const int> i7(i_data7);
  TEST_EQ(i7.size(), 7);
  // Check construction from a const array.
  const int i_cdata5[5] = { 4, 3, 2, 1, 0 };
  span<const int, 5> i8(i_cdata5);
  span<const int> i9(i_cdata5);
  TEST_EQ(i9.size(), 5);
  // Construction from a (ptr, size) pair.
  span<int, 7> i10(i_data7, 7);
  span<int> i11(i_data7, 7);
  TEST_EQ(i11.size(), 7);
  span<const int, 5> i12(i_cdata5, 5);
  span<const int> i13(i_cdata5, 5);
  TEST_EQ(i13.size(), 5);
  // Construction from std::array.
  std::array<int, 6> i_arr6 = { { 0, 1, 2, 3, 4, 5 } };
  span<int, 6> i14(i_arr6);
  span<const int, 6> i15(i_arr6);
  span<int> i16(i_arr6);
  span<const int> i17(i_arr6);
  TEST_EQ(i17.size(), 6);
  const std::array<int, 8> i_carr8 = { { 0, 1, 2, 3, 4, 5, 6, 7 } };
  span<const int, 8> i18(i_carr8);
  span<const int> i19(i_carr8);
  TEST_EQ(i18.size(), 8);
  TEST_EQ(i19.size(), 8);
  TEST_EQ(i19[7], 7);
  // Check compatibility with flatbuffers::Array.
  int fbs_int3_underlaying[3] = { 0 };
  int fbs_int3_data[3] = { 1, 2, 3 };
  auto &fbs_int3 = flatbuffers::CastToArray(fbs_int3_underlaying);
  fbs_int3.CopyFromSpan(fbs_int3_data);
  TEST_EQ(fbs_int3.Get(1), 2);
  const int fbs_cint3_data[3] = { 2, 3, 4 };
  fbs_int3.CopyFromSpan(fbs_cint3_data);
  TEST_EQ(fbs_int3.Get(1), 3);
  // Check with Array<Enum, N>
  enum class Dummy : uint16_t { Zero = 0, One, Two };
  Dummy fbs_dummy3_underlaying[3] = {};
  Dummy fbs_dummy3_data[3] = { Dummy::One, Dummy::Two, Dummy::Two };
  auto &fbs_dummy3 = flatbuffers::CastToArray(fbs_dummy3_underlaying);
  fbs_dummy3.CopyFromSpan(fbs_dummy3_data);
  TEST_EQ(fbs_dummy3.Get(1), Dummy::Two);
}
#else
void FlatbuffersSpanTest() {}
#endif

// VS10 does not support typed enums, exclude from tests
#if !defined(_MSC_VER) || _MSC_VER >= 1700
void FixedLengthArrayTest() {
  // Generate an ArrayTable containing one ArrayStruct.
  flatbuffers::FlatBufferBuilder fbb;
  MyGame::Example::NestedStruct nStruct0(MyGame::Example::TestEnum::B);
  TEST_NOTNULL(nStruct0.mutable_a());
  nStruct0.mutable_a()->Mutate(0, 1);
  nStruct0.mutable_a()->Mutate(1, 2);
  TEST_NOTNULL(nStruct0.mutable_c());
  nStruct0.mutable_c()->Mutate(0, MyGame::Example::TestEnum::C);
  nStruct0.mutable_c()->Mutate(1, MyGame::Example::TestEnum::A);
  TEST_NOTNULL(nStruct0.mutable_d());
  nStruct0.mutable_d()->Mutate(0, flatbuffers::numeric_limits<int64_t>::max());
  nStruct0.mutable_d()->Mutate(1, flatbuffers::numeric_limits<int64_t>::min());
  MyGame::Example::NestedStruct nStruct1(MyGame::Example::TestEnum::C);
  TEST_NOTNULL(nStruct1.mutable_a());
  nStruct1.mutable_a()->Mutate(0, 3);
  nStruct1.mutable_a()->Mutate(1, 4);
  TEST_NOTNULL(nStruct1.mutable_c());
  nStruct1.mutable_c()->Mutate(0, MyGame::Example::TestEnum::C);
  nStruct1.mutable_c()->Mutate(1, MyGame::Example::TestEnum::A);
  TEST_NOTNULL(nStruct1.mutable_d());
  nStruct1.mutable_d()->Mutate(0, flatbuffers::numeric_limits<int64_t>::min());
  nStruct1.mutable_d()->Mutate(1, flatbuffers::numeric_limits<int64_t>::max());
  MyGame::Example::ArrayStruct aStruct(2, 12, 1);
  TEST_NOTNULL(aStruct.b());
  TEST_NOTNULL(aStruct.mutable_b());
  TEST_NOTNULL(aStruct.mutable_d());
  TEST_NOTNULL(aStruct.mutable_f());
  for (int i = 0; i < aStruct.b()->size(); i++)
    aStruct.mutable_b()->Mutate(i, i + 1);
  aStruct.mutable_d()->Mutate(0, nStruct0);
  aStruct.mutable_d()->Mutate(1, nStruct1);
  auto aTable = MyGame::Example::CreateArrayTable(fbb, &aStruct);
  MyGame::Example::FinishArrayTableBuffer(fbb, aTable);
  // Verify correctness of the ArrayTable.
  flatbuffers::Verifier verifier(fbb.GetBufferPointer(), fbb.GetSize());
  TEST_ASSERT(MyGame::Example::VerifyArrayTableBuffer(verifier));
  // Do test.
  auto p = MyGame::Example::GetMutableArrayTable(fbb.GetBufferPointer());
  auto mArStruct = p->mutable_a();
  TEST_NOTNULL(mArStruct);
  TEST_NOTNULL(mArStruct->b());
  TEST_NOTNULL(mArStruct->d());
  TEST_NOTNULL(mArStruct->f());
  TEST_NOTNULL(mArStruct->mutable_b());
  TEST_NOTNULL(mArStruct->mutable_d());
  TEST_NOTNULL(mArStruct->mutable_f());
  TEST_EQ(mArStruct->a(), 2);
  TEST_EQ(mArStruct->b()->size(), 15);
  mArStruct->mutable_b()->Mutate(14, -14);
  TEST_EQ(mArStruct->b()->Get(14), -14);
  TEST_EQ(mArStruct->c(), 12);
  TEST_NOTNULL(mArStruct->d()->Get(0));
  TEST_NOTNULL(mArStruct->d()->Get(0)->a());
  TEST_EQ(mArStruct->d()->Get(0)->a()->Get(0), 1);
  TEST_EQ(mArStruct->d()->Get(0)->a()->Get(1), 2);
  TEST_NOTNULL(mArStruct->d()->Get(1));
  TEST_NOTNULL(mArStruct->d()->Get(1)->a());
  TEST_EQ(mArStruct->d()->Get(1)->a()->Get(0), 3);
  TEST_EQ(mArStruct->d()->Get(1)->a()->Get(1), 4);
  TEST_NOTNULL(mArStruct->mutable_d()->GetMutablePointer(1));
  TEST_NOTNULL(mArStruct->mutable_d()->GetMutablePointer(1)->mutable_a());
  mArStruct->mutable_d()->GetMutablePointer(1)->mutable_a()->Mutate(1, 5);
  TEST_EQ(5, mArStruct->d()->Get(1)->a()->Get(1));
  TEST_EQ(MyGame::Example::TestEnum::B, mArStruct->d()->Get(0)->b());
  TEST_NOTNULL(mArStruct->d()->Get(0)->c());
  TEST_EQ(MyGame::Example::TestEnum::C, mArStruct->d()->Get(0)->c()->Get(0));
  TEST_EQ(MyGame::Example::TestEnum::A, mArStruct->d()->Get(0)->c()->Get(1));
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::max(),
          mArStruct->d()->Get(0)->d()->Get(0));
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::min(),
          mArStruct->d()->Get(0)->d()->Get(1));
  TEST_EQ(MyGame::Example::TestEnum::C, mArStruct->d()->Get(1)->b());
  TEST_NOTNULL(mArStruct->d()->Get(1)->c());
  TEST_EQ(MyGame::Example::TestEnum::C, mArStruct->d()->Get(1)->c()->Get(0));
  TEST_EQ(MyGame::Example::TestEnum::A, mArStruct->d()->Get(1)->c()->Get(1));
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::min(),
          mArStruct->d()->Get(1)->d()->Get(0));
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::max(),
          mArStruct->d()->Get(1)->d()->Get(1));
  for (int i = 0; i < mArStruct->b()->size() - 1; i++)
    TEST_EQ(mArStruct->b()->Get(i), i + 1);
  // Check alignment
  TEST_EQ(0, reinterpret_cast<uintptr_t>(mArStruct->d()) % 8);
  TEST_EQ(0, reinterpret_cast<uintptr_t>(mArStruct->f()) % 8);

  // Check if default constructor set all memory zero
  const size_t arr_size = sizeof(MyGame::Example::ArrayStruct);
  char non_zero_memory[arr_size];
  // set memory chunk of size ArrayStruct to 1's
  std::memset(static_cast<void *>(non_zero_memory), 1, arr_size);
  // after placement-new it should be all 0's
#  if defined(_MSC_VER) && defined(_DEBUG)
#    undef new
#  endif
  MyGame::Example::ArrayStruct *ap =
      new (non_zero_memory) MyGame::Example::ArrayStruct;
#  if defined(_MSC_VER) && defined(_DEBUG)
#    define new DEBUG_NEW
#  endif
  (void)ap;
  for (size_t i = 0; i < arr_size; ++i) { TEST_EQ(non_zero_memory[i], 0); }
}
#else
void FixedLengthArrayTest() {}
#endif  // !defined(_MSC_VER) || _MSC_VER >= 1700

#if !defined(FLATBUFFERS_SPAN_MINIMAL) && \
    (!defined(_MSC_VER) || _MSC_VER >= 1700)
void FixedLengthArrayConstructorTest() {
  const int32_t nested_a[2] = { 1, 2 };
  MyGame::Example::TestEnum nested_c[2] = { MyGame::Example::TestEnum::A,
                                            MyGame::Example::TestEnum::B };
  const int64_t int64_2[2] = { -2, -1 };

  std::array<MyGame::Example::NestedStruct, 2> init_d = {
    { MyGame::Example::NestedStruct(nested_a, MyGame::Example::TestEnum::B,
                                    nested_c, int64_2),
      MyGame::Example::NestedStruct(nested_a, MyGame::Example::TestEnum::A,
                                    nested_c,
                                    std::array<int64_t, 2>{ { 12, 13 } }) }
  };

  MyGame::Example::ArrayStruct arr_struct(
      8.125,
      std::array<int32_t, 0xF>{
          { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 } },
      -17, init_d, 10, int64_2);
  TEST_EQ(arr_struct.a(), 8.125);
  TEST_EQ(arr_struct.b()->Get(2), 3);
  TEST_EQ(arr_struct.c(), -17);

  TEST_NOTNULL(arr_struct.d());
  const auto &arr_d_0 = *arr_struct.d()->Get(0);
  TEST_EQ(arr_d_0.a()->Get(0), 1);
  TEST_EQ(arr_d_0.a()->Get(1), 2);
  TEST_EQ(arr_d_0.b(), MyGame::Example::TestEnum::B);
  TEST_EQ(arr_d_0.c()->Get(0), MyGame::Example::TestEnum::A);
  TEST_EQ(arr_d_0.c()->Get(1), MyGame::Example::TestEnum::B);
  TEST_EQ(arr_d_0.d()->Get(0), -2);
  TEST_EQ(arr_d_0.d()->Get(1), -1);
  const auto &arr_d_1 = *arr_struct.d()->Get(1);
  TEST_EQ(arr_d_1.a()->Get(0), 1);
  TEST_EQ(arr_d_1.a()->Get(1), 2);
  TEST_EQ(arr_d_1.b(), MyGame::Example::TestEnum::A);
  TEST_EQ(arr_d_1.c()->Get(0), MyGame::Example::TestEnum::A);
  TEST_EQ(arr_d_1.c()->Get(1), MyGame::Example::TestEnum::B);
  TEST_EQ(arr_d_1.d()->Get(0), 12);
  TEST_EQ(arr_d_1.d()->Get(1), 13);

  TEST_EQ(arr_struct.e(), 10);
  TEST_EQ(arr_struct.f()->Get(0), -2);
  TEST_EQ(arr_struct.f()->Get(1), -1);
}
#else
void FixedLengthArrayConstructorTest() {}
#endif

void FixedLengthArrayOperatorEqualTest() {
  const int32_t nested_a[2] = { 1, 2 };
  MyGame::Example::TestEnum nested_c[2] = { MyGame::Example::TestEnum::A,
                                            MyGame::Example::TestEnum::B };

  MyGame::Example::TestEnum nested_cc[2] = { MyGame::Example::TestEnum::A,
                                             MyGame::Example::TestEnum::C };
  const int64_t int64_2[2] = { -2, -1 };

  std::array<MyGame::Example::NestedStruct, 2> init_d = {
    { MyGame::Example::NestedStruct(nested_a, MyGame::Example::TestEnum::B,
                                    nested_c, int64_2),
      MyGame::Example::NestedStruct(nested_a, MyGame::Example::TestEnum::B,
                                    nested_c,
                                    std::array<int64_t, 2>{ { -2, -1 } }) }
  };

  auto different = MyGame::Example::NestedStruct(
      nested_a, MyGame::Example::TestEnum::B, nested_cc,
      std::array<int64_t, 2>{ { -2, -1 } });

  TEST_ASSERT(init_d[0] == init_d[1]);
  TEST_ASSERT(init_d[0] != different);

  std::array<MyGame::Example::ArrayStruct, 3> arr_struct = {
    MyGame::Example::ArrayStruct(
        8.125,
        std::array<int32_t, 0xF>{
            { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 } },
        -17, init_d, 10, int64_2),

    MyGame::Example::ArrayStruct(
        8.125,
        std::array<int32_t, 0xF>{
            { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 } },
        -17, init_d, 10, int64_2),

    MyGame::Example::ArrayStruct(
        8.125,
        std::array<int32_t, 0xF>{
            { 1000, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 } },
        -17, init_d, 10, int64_2)
  };

  TEST_ASSERT(arr_struct[0] == arr_struct[1]);
  TEST_ASSERT(arr_struct[1] != arr_struct[2]);
}

void NativeTypeTest() {
  const int N = 3;

  Geometry::ApplicationDataT src_data;
  src_data.vectors.reserve(N);
  src_data.vectors_alt.reserve(N);

  for (int i = 0; i < N; ++i) {
    src_data.vectors.push_back(
        Native::Vector3D(10 * i + 0.1f, 10 * i + 0.2f, 10 * i + 0.3f));
    src_data.vectors_alt.push_back(
        Native::Vector3D(20 * i + 0.1f, 20 * i + 0.2f, 20 * i + 0.3f));
  }

  flatbuffers::FlatBufferBuilder fbb;
  fbb.Finish(Geometry::ApplicationData::Pack(fbb, &src_data));

  auto dstDataT = Geometry::UnPackApplicationData(fbb.GetBufferPointer());

  for (int i = 0; i < N; ++i) {
    const Native::Vector3D &v = dstDataT->vectors[i];
    TEST_EQ(v.x, 10 * i + 0.1f);
    TEST_EQ(v.y, 10 * i + 0.2f);
    TEST_EQ(v.z, 10 * i + 0.3f);

    const Native::Vector3D &v2 = dstDataT->vectors_alt[i];
    TEST_EQ(v2.x, 20 * i + 0.1f);
    TEST_EQ(v2.y, 20 * i + 0.2f);
    TEST_EQ(v2.z, 20 * i + 0.3f);
  }
}

// Guard against -Wunused-function on platforms without file tests.
#ifndef FLATBUFFERS_NO_FILE_TESTS
// VS10 does not support typed enums, exclude from tests
#  if !defined(_MSC_VER) || _MSC_VER >= 1700
void FixedLengthArrayJsonTest(const std::string &tests_data_path, bool binary) {
  // load FlatBuffer schema (.fbs) and JSON from disk
  std::string schemafile;
  std::string jsonfile;
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "arrays_test." + (binary ? "bfbs" : "fbs"))
                  .c_str(),
              binary, &schemafile),
          true);
  TEST_EQ(
      flatbuffers::LoadFile((tests_data_path + "arrays_test.golden").c_str(),
                            false, &jsonfile),
      true);

  // parse schema first, so we can use it to parse the data after
  flatbuffers::Parser parserOrg, parserGen;
  if (binary) {
    flatbuffers::Verifier verifier(
        reinterpret_cast<const uint8_t *>(schemafile.c_str()),
        schemafile.size());
    TEST_EQ(reflection::VerifySchemaBuffer(verifier), true);
    TEST_EQ(parserOrg.Deserialize(
                reinterpret_cast<const uint8_t *>(schemafile.c_str()),
                schemafile.size()),
            true);
    TEST_EQ(parserGen.Deserialize(
                reinterpret_cast<const uint8_t *>(schemafile.c_str()),
                schemafile.size()),
            true);
  } else {
    TEST_EQ(parserOrg.Parse(schemafile.c_str()), true);
    TEST_EQ(parserGen.Parse(schemafile.c_str()), true);
  }
  TEST_EQ(parserOrg.Parse(jsonfile.c_str()), true);

  // First, verify it, just in case:
  flatbuffers::Verifier verifierOrg(parserOrg.builder_.GetBufferPointer(),
                                    parserOrg.builder_.GetSize());
  TEST_EQ(VerifyArrayTableBuffer(verifierOrg), true);

  // Export to JSON
  std::string jsonGen;
  TEST_NULL(
      GenText(parserOrg, parserOrg.builder_.GetBufferPointer(), &jsonGen));

  // Import from JSON
  TEST_EQ(parserGen.Parse(jsonGen.c_str()), true);

  // Verify buffer from generated JSON
  flatbuffers::Verifier verifierGen(parserGen.builder_.GetBufferPointer(),
                                    parserGen.builder_.GetSize());
  TEST_EQ(VerifyArrayTableBuffer(verifierGen), true);

  // Compare generated buffer to original
  TEST_EQ(parserOrg.builder_.GetSize(), parserGen.builder_.GetSize());
  TEST_EQ(std::memcmp(parserOrg.builder_.GetBufferPointer(),
                      parserGen.builder_.GetBufferPointer(),
                      parserOrg.builder_.GetSize()),
          0);
}

void FixedLengthArraySpanTest(const std::string &tests_data_path) {
  // load FlatBuffer schema (.fbs) and JSON from disk
  std::string schemafile;
  std::string jsonfile;
  TEST_EQ(flatbuffers::LoadFile((tests_data_path + "arrays_test.fbs").c_str(),
                                false, &schemafile),
          true);
  TEST_EQ(
      flatbuffers::LoadFile((tests_data_path + "arrays_test.golden").c_str(),
                            false, &jsonfile),
      true);

  // parse schema first, so we can use it to parse the data after
  flatbuffers::Parser parser;
  TEST_EQ(parser.Parse(schemafile.c_str()), true);
  TEST_EQ(parser.Parse(jsonfile.c_str()), true);
  auto &fbb = parser.builder_;
  auto verifier = flatbuffers::Verifier(fbb.GetBufferPointer(), fbb.GetSize());
  TEST_EQ(true, VerifyArrayTableBuffer(verifier));

  auto p = MyGame::Example::GetMutableArrayTable(fbb.GetBufferPointer());
  TEST_NOTNULL(p);
  auto table_struct = p->mutable_a();
  TEST_NOTNULL(table_struct);
  TEST_EQ(2, table_struct->d()->size());
  TEST_NOTNULL(table_struct->d());
  TEST_NOTNULL(table_struct->mutable_d());
  // test array of structs
  auto const_d = flatbuffers::make_span(*table_struct->d());
  auto mutable_d = flatbuffers::make_span(*table_struct->mutable_d());
  TEST_EQ(2, const_d.size());
  TEST_EQ(2, mutable_d.size());
  TEST_ASSERT(const_d[0] == mutable_d[0]);
  TEST_ASSERT(const_d[1] == mutable_d[1]);
  mutable_d[0] = const_d[0];  // mutate
  // test scalars
  auto &const_nested = const_d[0];
  auto &mutable_nested = mutable_d[0];
  static_assert(sizeof(MyGame::Example::TestEnum) == sizeof(uint8_t),
                "TestEnum's underlaying type must by byte");
  TEST_NOTNULL(const_nested.d());
  TEST_NOTNULL(mutable_nested.d());
  {
    flatbuffers::span<const MyGame::Example::TestEnum, 2> const_d_c =
        flatbuffers::make_span(*const_nested.c());
    auto mutable_d_c = flatbuffers::make_span(*mutable_nested.mutable_c());
    TEST_EQ(2, const_d_c.size());
    TEST_EQ(2, mutable_d_c.size());
    TEST_EQ(MyGame::Example::TestEnum::C, const_d_c[0]);
    TEST_EQ(MyGame::Example::TestEnum::B, const_d_c[1]);
    TEST_ASSERT(mutable_d_c.end() == std::copy(const_d_c.begin(),
                                               const_d_c.end(),
                                               mutable_d_c.begin()));
    TEST_ASSERT(
        std::equal(const_d_c.begin(), const_d_c.end(), mutable_d_c.begin()));
  }
  // test little endian array of int32
#    if FLATBUFFERS_LITTLEENDIAN
  {
    flatbuffers::span<const int32_t, 2> const_d_a =
        flatbuffers::make_span(*const_nested.a());
    auto mutable_d_a = flatbuffers::make_span(*mutable_nested.mutable_a());
    TEST_EQ(2, const_d_a.size());
    TEST_EQ(2, mutable_d_a.size());
    TEST_EQ(-1, const_d_a[0]);
    TEST_EQ(2, const_d_a[1]);
    TEST_ASSERT(mutable_d_a.end() == std::copy(const_d_a.begin(),
                                               const_d_a.end(),
                                               mutable_d_a.begin()));
    TEST_ASSERT(
        std::equal(const_d_a.begin(), const_d_a.end(), mutable_d_a.begin()));
  }
#    endif
}
#  else
void FixedLengthArrayJsonTest(bool /*binary*/) {}
void FixedLengthArraySpanTest() {}
#  endif

void TestEmbeddedBinarySchema(const std::string &tests_data_path) {
  // load JSON from disk
  std::string jsonfile;
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "monsterdata_test.golden").c_str(), false,
              &jsonfile),
          true);

  // parse schema first, so we can use it to parse the data after
  flatbuffers::Parser parserOrg, parserGen;
  flatbuffers::Verifier verifier(MyGame::Example::MonsterBinarySchema::data(),
                                 MyGame::Example::MonsterBinarySchema::size());
  TEST_EQ(reflection::VerifySchemaBuffer(verifier), true);
  TEST_EQ(parserOrg.Deserialize(MyGame::Example::MonsterBinarySchema::data(),
                                MyGame::Example::MonsterBinarySchema::size()),
          true);
  TEST_EQ(parserGen.Deserialize(MyGame::Example::MonsterBinarySchema::data(),
                                MyGame::Example::MonsterBinarySchema::size()),
          true);
  TEST_EQ(parserOrg.Parse(jsonfile.c_str()), true);

  // First, verify it, just in case:
  flatbuffers::Verifier verifierOrg(parserOrg.builder_.GetBufferPointer(),
                                    parserOrg.builder_.GetSize());
  TEST_EQ(VerifyMonsterBuffer(verifierOrg), true);

  // Export to JSON
  std::string jsonGen;
  TEST_NULL(
      GenText(parserOrg, parserOrg.builder_.GetBufferPointer(), &jsonGen));

  // Import from JSON
  TEST_EQ(parserGen.Parse(jsonGen.c_str()), true);

  // Verify buffer from generated JSON
  flatbuffers::Verifier verifierGen(parserGen.builder_.GetBufferPointer(),
                                    parserGen.builder_.GetSize());
  TEST_EQ(VerifyMonsterBuffer(verifierGen), true);

  // Compare generated buffer to original
  TEST_EQ(parserOrg.builder_.GetSize(), parserGen.builder_.GetSize());
  TEST_EQ(std::memcmp(parserOrg.builder_.GetBufferPointer(),
                      parserGen.builder_.GetBufferPointer(),
                      parserOrg.builder_.GetSize()),
          0);
}
#endif

template<typename T> void EmbeddedSchemaAccessByType() {
  // Get the binary schema from the Type itself.
  // Verify the schema is OK.
  flatbuffers::Verifier verifierEmbeddedSchema(
      T::TableType::BinarySchema::data(), T::TableType::BinarySchema::size());
  TEST_EQ(reflection::VerifySchemaBuffer(verifierEmbeddedSchema), true);

  // Reflect it.
  auto schema = reflection::GetSchema(T::TableType::BinarySchema::data());

  // This should equal the expected root table.
  TEST_EQ_STR(schema->root_table()->name()->c_str(), "MyGame.Example.Monster");
}

void EmbeddedSchemaAccess() {
  // Get the binary schema for the monster.
  // Verify the schema is OK.
  flatbuffers::Verifier verifierEmbeddedSchema(Monster::BinarySchema::data(),
                                               Monster::BinarySchema::size());
  TEST_EQ(reflection::VerifySchemaBuffer(verifierEmbeddedSchema), true);

  // Reflect it.
  auto schema = reflection::GetSchema(Monster::BinarySchema::data());

  // This should equal the expected root table.
  TEST_EQ_STR(schema->root_table()->name()->c_str(), "MyGame.Example.Monster");

  // Repeat above, but do so through a template parameter:
  EmbeddedSchemaAccessByType<StatT>();
}

void NestedVerifierTest() {
  // Create a nested monster.
  flatbuffers::FlatBufferBuilder nested_builder;
  FinishMonsterBuffer(
      nested_builder,
      CreateMonster(nested_builder, nullptr, 0, 0,
                    nested_builder.CreateString("NestedMonster")));

  // Verify the nested monster
  flatbuffers::Verifier verifier(nested_builder.GetBufferPointer(),
                                 nested_builder.GetSize());
  TEST_EQ(true, VerifyMonsterBuffer(verifier));

  {
    // Create the outer monster.
    flatbuffers::FlatBufferBuilder builder;

    // Add the nested monster as a vector of bytes.
    auto nested_monster_bytes = builder.CreateVector(
        nested_builder.GetBufferPointer(), nested_builder.GetSize());

    auto name = builder.CreateString("OuterMonster");

    MonsterBuilder mon_builder(builder);
    mon_builder.add_name(name);
    mon_builder.add_testnestedflatbuffer(nested_monster_bytes);
    FinishMonsterBuffer(builder, mon_builder.Finish());

    // Verify the root monster, which includes verifing the nested monster
    flatbuffers::Verifier verifier(builder.GetBufferPointer(),
                                   builder.GetSize());
    TEST_EQ(true, VerifyMonsterBuffer(verifier));
  }

  {
    // Create the outer monster.
    flatbuffers::FlatBufferBuilder builder;

    // Purposely invalidate the nested flatbuffer setting its length to 1, an
    // invalid length.
    uint8_t invalid_nested_buffer[1];
    auto nested_monster_bytes = builder.CreateVector(invalid_nested_buffer, 1);

    auto name = builder.CreateString("OuterMonster");

    MonsterBuilder mon_builder(builder);
    mon_builder.add_name(name);
    mon_builder.add_testnestedflatbuffer(nested_monster_bytes);
    FinishMonsterBuffer(builder, mon_builder.Finish());

    // Verify the root monster fails, since the included nested monster fails.
    flatbuffers::Verifier verifier(builder.GetBufferPointer(),
                                   builder.GetSize());
    TEST_EQ(false, VerifyMonsterBuffer(verifier));

    // Verify the root monster succeeds, since we've disabled checking nested
    // flatbuffers
    flatbuffers::Verifier::Options options;
    options.check_nested_flatbuffers = false;
    flatbuffers::Verifier no_check_nested(builder.GetBufferPointer(),
                                          builder.GetSize(), options);
    TEST_EQ(true, VerifyMonsterBuffer(no_check_nested));
  }

  {
    // Create the outer monster.
    flatbuffers::FlatBufferBuilder builder;

    // Purposely invalidate the nested flatbuffer setting its length to 0, an
    // invalid length.
    uint8_t *invalid_nested_buffer = nullptr;
    auto nested_monster_bytes = builder.CreateVector(invalid_nested_buffer, 0);

    auto name = builder.CreateString("OuterMonster");

    MonsterBuilder mon_builder(builder);
    mon_builder.add_name(name);
    mon_builder.add_testnestedflatbuffer(nested_monster_bytes);
    FinishMonsterBuffer(builder, mon_builder.Finish());

    // Verify the root monster fails, since the included nested monster fails.
    flatbuffers::Verifier verifier(builder.GetBufferPointer(),
                                   builder.GetSize());
    TEST_EQ(false, VerifyMonsterBuffer(verifier));
  }
}

template<class T, class Container>
void TestIterators(const std::vector<T> &expected, const Container &tested) {
  TEST_ASSERT(tested.rbegin().base() == tested.end());
  TEST_ASSERT(tested.crbegin().base() == tested.cend());
  TEST_ASSERT(tested.rend().base() == tested.begin());
  TEST_ASSERT(tested.crend().base() == tested.cbegin());

  size_t k = 0;
  for (auto it = tested.begin(); it != tested.end(); ++it, ++k) {
    const auto &e = expected.at(k);
    TEST_EQ(*it, e);
  }
  TEST_EQ(k, expected.size());

  k = expected.size();
  for (auto it = tested.rbegin(); it != tested.rend(); ++it, --k) {
    const auto &e = expected.at(k - 1);
    TEST_EQ(*it, e);
  }
  TEST_EQ(k, 0);
}

void FlatbuffersIteratorsTest() {
  {
    flatbuffers::FlatBufferBuilder fbb;
    const std::vector<unsigned char> inv_data = { 1, 2, 3 };
    {
      auto mon_name = fbb.CreateString("MyMonster");  // key, mandatory
      auto inv_vec = fbb.CreateVector(inv_data);
      auto empty_i64_vec =
          fbb.CreateVector(static_cast<const int64_t *>(nullptr), 0);
      MonsterBuilder mb(fbb);
      mb.add_name(mon_name);
      mb.add_inventory(inv_vec);
      mb.add_vector_of_longs(empty_i64_vec);
      FinishMonsterBuffer(fbb, mb.Finish());
    }
    const auto &mon = *flatbuffers::GetRoot<Monster>(fbb.GetBufferPointer());

    TEST_EQ_STR("MyMonster", mon.name()->c_str());
    TEST_ASSERT(mon.inventory());
    TEST_ASSERT(mon.vector_of_longs());
    TestIterators(inv_data, *mon.inventory());
    TestIterators(std::vector<int64_t>(), *mon.vector_of_longs());
  }

  {
    flatbuffers::FlatBufferBuilder fbb;
    MyGame::Example::ArrayStruct aStruct;
    MyGame::Example::FinishArrayTableBuffer(
        fbb, MyGame::Example::CreateArrayTable(fbb, &aStruct));
    const auto &array_table =
        *flatbuffers::GetRoot<ArrayTable>(fbb.GetBufferPointer());
    TEST_ASSERT(array_table.a());
    auto &int_15 = *array_table.a()->b();
    TestIterators(std::vector<int>(15, 0), int_15);
  }
}

void PrivateAnnotationsLeaks() {
  // Simple schemas and a "has optional scalar" sentinal.
  std::vector<std::string> schemas;
  std::vector<std::string> failure_schemas;

  // (private) (table/struct)
  schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC (private) { mana: int; }");

  // (public) (table/struct)
  schemas.push_back(
      "table Monster { mana: int; }"
      "struct ABC { mana: int; }");

  // (private) (union) containing (private) (table/struct)
  schemas.push_back(
      "table Monster (private) { mana: int; } "
      "struct ABC (private) { mana: int; } "
      "union Any (private) { Monster, ABC } ");

  // (public) (union) containing (public) (table/struct)
  schemas.push_back(
      "table Monster { mana: int; }"
      "struct ABC { mana: int; }"
      "union Any { Monster, ABC }");

  // (private) (table/struct/enum)
  schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC (private) { mana: int; }"
      "enum Race:byte (private) { None = -1, Human = 0, }");

  // (public) (table/struct/enum)
  schemas.push_back(
      "table Monster { mana: int; }"
      "struct ABC { mana: int; }"
      "enum Race:byte { None = -1, Human = 0, }");

  // (private) (union) containing (private) (table/struct)
  schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC (private) { mana: int; }"
      "enum Race:byte (private) { None = -1, Human = 0, }"
      "union Any (private) { Monster, ABC }");

  // (public) (union) containing (public) (table/struct)
  schemas.push_back(
      "table Monster { mana: int; }"
      "struct ABC { mana: int; }"
      "enum Race:byte { None = -1, Human = 0, }"
      "union Any { Monster, ABC }");

  // (private) (table), (public struct)
  schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC { mana: int; }");

  // (private) (table), (public) (struct/enum)
  schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC { mana: int; }"
      "enum Race:byte { None = -1, Human = 0, }");

  // (public) (struct) containing (public) (enum)
  schemas.push_back(
      "enum Race:byte { None = -1, Human = 0, }"
      "table Monster { mana: int; }"
      "struct ABC { mana: int; type: Race; }");

  // (public) (union) containing (private) (table) & (public) (struct)
  failure_schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC { mana: int; }"
      "union Any { Monster, ABC }");

  // (public) (union) containing (private) (table/struct)
  failure_schemas.push_back(
      "table Monster (private) { mana: int; }"
      "struct ABC (private) { mana: int; }"
      "enum Race:byte { None = -1, Human = 0, }"
      "union Any { Monster, ABC }");

  // (public) (table) containing (private) (struct)
  failure_schemas.push_back(
      "table Monster { mana: int; ab: ABC; }"
      "struct ABC (private) { mana: int; }");

  // (public) (struct) containing (private) (enum)
  failure_schemas.push_back(
      "enum Race:byte (private) { None = -1, Human = 0, }"
      "table Monster { mana: int; }"
      "struct ABC { mana: int; type: Race; }");

  flatbuffers::IDLOptions opts;
  opts.lang_to_generate = flatbuffers::IDLOptions::Language::kSwift;
  opts.no_leak_private_annotations = true;

  for (auto schema = schemas.begin(); schema < schemas.end(); schema++) {
    flatbuffers::Parser parser(opts);
    TEST_ASSERT(parser.Parse(schema->c_str()));
  }

  for (auto schema = failure_schemas.begin(); schema < failure_schemas.end();
       schema++) {
    flatbuffers::Parser parser(opts);
    TEST_EQ(false, parser.Parse(schema->c_str()));
  }

  opts.no_leak_private_annotations = false;

  for (auto schema = schemas.begin(); schema < schemas.end(); schema++) {
    flatbuffers::Parser parser(opts);
    TEST_ASSERT(parser.Parse(schema->c_str()));
  }

  for (auto schema = failure_schemas.begin(); schema < failure_schemas.end();
       schema++) {
    flatbuffers::Parser parser(opts);
    TEST_ASSERT(parser.Parse(schema->c_str()));
  }
}

void VectorSpanTest() {
  flatbuffers::FlatBufferBuilder builder;

  auto mloc = CreateMonster(
      builder, nullptr, 0, 0, builder.CreateString("Monster"),
      builder.CreateVector<uint8_t>({ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 }));

  FinishMonsterBuffer(builder, mloc);

  auto monster = GetMonster(builder.GetBufferPointer());
  auto mutable_monster = GetMutableMonster(builder.GetBufferPointer());

  {  // using references
    TEST_NOTNULL(monster->inventory());

    flatbuffers::span<const uint8_t> const_inventory =
        flatbuffers::make_span(*monster->inventory());
    TEST_EQ(const_inventory.size(), 10);
    TEST_EQ(const_inventory[0], 0);
    TEST_EQ(const_inventory[9], 9);

    flatbuffers::span<uint8_t> mutable_inventory =
        flatbuffers::make_span(*mutable_monster->mutable_inventory());
    TEST_EQ(mutable_inventory.size(), 10);
    TEST_EQ(mutable_inventory[0], 0);
    TEST_EQ(mutable_inventory[9], 9);

    mutable_inventory[0] = 42;
    TEST_EQ(mutable_inventory[0], 42);

    mutable_inventory[0] = 0;
    TEST_EQ(mutable_inventory[0], 0);
  }

  {  // using pointers
    TEST_EQ(flatbuffers::VectorLength(monster->inventory()), 10);

    flatbuffers::span<const uint8_t> const_inventory =
        flatbuffers::make_span(monster->inventory());
    TEST_EQ(const_inventory.size(), 10);
    TEST_EQ(const_inventory[0], 0);
    TEST_EQ(const_inventory[9], 9);

    flatbuffers::span<uint8_t> mutable_inventory =
        flatbuffers::make_span(mutable_monster->mutable_inventory());
    TEST_EQ(mutable_inventory.size(), 10);
    TEST_EQ(mutable_inventory[0], 0);
    TEST_EQ(mutable_inventory[9], 9);

    mutable_inventory[0] = 42;
    TEST_EQ(mutable_inventory[0], 42);

    mutable_inventory[0] = 0;
    TEST_EQ(mutable_inventory[0], 0);
  }

  {
    TEST_ASSERT(nullptr == monster->testnestedflatbuffer());

    TEST_EQ(flatbuffers::VectorLength(monster->testnestedflatbuffer()), 0);

    flatbuffers::span<const uint8_t> const_nested =
        flatbuffers::make_span(monster->testnestedflatbuffer());
    TEST_ASSERT(const_nested.empty());

    flatbuffers::span<uint8_t> mutable_nested =
        flatbuffers::make_span(mutable_monster->mutable_testnestedflatbuffer());
    TEST_ASSERT(mutable_nested.empty());
  }
}

void NativeInlineTableVectorTest() {
  TestNativeInlineTableT test;
  for (int i = 0; i < 10; ++i) {
    NativeInlineTableT t;
    t.a = i;
    test.t.push_back(t);
  }

  flatbuffers::FlatBufferBuilder fbb;
  auto offset = TestNativeInlineTable::Pack(fbb, &test);
  fbb.Finish(offset);

  auto *root =
      flatbuffers::GetRoot<TestNativeInlineTable>(fbb.GetBufferPointer());
  TestNativeInlineTableT unpacked;
  root->UnPackTo(&unpacked);

  for (int i = 0; i < 10; ++i) { TEST_ASSERT(unpacked.t[i] == test.t[i]); }

  TEST_ASSERT(unpacked.t == test.t);
}

// Guard against -Wunused-function on platforms without file tests.
#ifndef FLATBUFFERS_NO_FILE_TESTS
void DoNotRequireEofTest(const std::string &tests_data_path) {
  std::string schemafile;
  bool ok = flatbuffers::LoadFile(
      (tests_data_path + "monster_test.fbs").c_str(), false, &schemafile);
  TEST_EQ(ok, true);
  auto include_test_path =
      flatbuffers::ConCatPathFileName(tests_data_path, "include_test");
  const char *include_directories[] = { tests_data_path.c_str(),
                                        include_test_path.c_str(), nullptr };
  flatbuffers::IDLOptions opt;
  opt.require_json_eof = false;
  flatbuffers::Parser parser(opt);
  ok = parser.Parse(schemafile.c_str(), include_directories);
  TEST_EQ(ok, true);

  const char *str = R"(Some text at the beginning. {
      "name": "Blob",
      "hp": 5
    }{
      "name": "Imp",
      "hp": 10
    }
    Some extra text at the end too.
  )";
  const char *tableStart = std::strchr(str, '{');
  ok = parser.ParseJson(tableStart);
  TEST_EQ(ok, true);

  const Monster *monster = GetMonster(parser.builder_.GetBufferPointer());
  TEST_EQ_STR(monster->name()->c_str(), "Blob");
  TEST_EQ(monster->hp(), 5);

  tableStart += parser.BytesConsumed();

  ok = parser.ParseJson(tableStart);
  TEST_EQ(ok, true);

  monster = GetMonster(parser.builder_.GetBufferPointer());
  TEST_EQ_STR(monster->name()->c_str(), "Imp");
  TEST_EQ(monster->hp(), 10);
}
#endif

void UnionUnderlyingTypeTest() {
    using namespace UnionUnderlyingType;
    TEST_ASSERT(sizeof(ABC) == sizeof(uint32_t));
    TEST_ASSERT(static_cast<int32_t>(ABC::A) == 555);
    TEST_ASSERT(static_cast<int32_t>(ABC::B) == 666);
    TEST_ASSERT(static_cast<int32_t>(ABC::C) == 777);

    DT buffer;
    AT a;
    a.a = 42;
    BT b;
    b.b = "foo";
    CT c;
    c.c = true;
    buffer.test_union = ABCUnion();
    buffer.test_union.Set(a);
    buffer.test_vector_of_union.resize(3);
    buffer.test_vector_of_union[0].Set(a);
    buffer.test_vector_of_union[1].Set(b);
    buffer.test_vector_of_union[2].Set(c);

    flatbuffers::FlatBufferBuilder fbb;
    auto offset = D::Pack(fbb, &buffer);
    fbb.Finish(offset);

    auto *root =
    flatbuffers::GetRoot<D>(fbb.GetBufferPointer());
    DT unpacked;
    root->UnPackTo(&unpacked);

    TEST_ASSERT(unpacked.test_union == buffer.test_union);
    TEST_ASSERT(unpacked.test_vector_of_union == buffer.test_vector_of_union);

}

static void Offset64Tests() {
#if INCLUDE_64_BIT_TESTS
  Offset64Test();
  Offset64SerializedFirst();
  Offset64NestedFlatBuffer();
  Offset64CreateDirect();
  Offset64Evolution();
  Offset64VectorOfStructs();
  Offset64SizePrefix();
  Offset64ManyVectors();
  Offset64ForceAlign();
#endif
}

int FlatBufferTests(const std::string &tests_data_path) {
  // Run our various test suites:

  std::string rawbuf;
  auto flatbuf1 = CreateFlatBufferTest(rawbuf);
  auto flatbuf = std::move(flatbuf1);  // Test move assignment.

  AccessFlatBufferTest(reinterpret_cast<const uint8_t *>(rawbuf.c_str()),
                       rawbuf.length());
  AccessFlatBufferTest(flatbuf.data(), flatbuf.size());

  MutateFlatBuffersTest(flatbuf.data(), flatbuf.size());

  ObjectFlatBuffersTest(flatbuf.data());
  UnPackTo(flatbuf.data());

  MiniReflectFlatBuffersTest(flatbuf.data());
  MiniReflectFixedLengthArrayTest();

  SizePrefixedTest();

  AlignmentTest();

#ifndef FLATBUFFERS_NO_FILE_TESTS
  ParseAndGenerateTextTest(tests_data_path, false);
  ParseAndGenerateTextTest(tests_data_path, true);
  FixedLengthArrayJsonTest(tests_data_path, false);
  FixedLengthArrayJsonTest(tests_data_path, true);
  ReflectionTest(tests_data_path, flatbuf.data(), flatbuf.size());
  ParseProtoTest(tests_data_path);
  EvolutionTest(tests_data_path);
  UnionDeprecationTest(tests_data_path);
  UnionVectorTest(tests_data_path);
  GenerateTableTextTest(tests_data_path);
  TestEmbeddedBinarySchema(tests_data_path);
  JsonOptionalTest(tests_data_path, false);
  JsonOptionalTest(tests_data_path, true);
  MultiFileNameClashTest(tests_data_path);
  InvalidNestedFlatbufferTest(tests_data_path);
  JsonDefaultTest(tests_data_path);
  JsonEnumsTest(tests_data_path);
  TestMonsterExtraFloats(tests_data_path);
  ParseIncorrectMonsterJsonTest(tests_data_path);
  FixedLengthArraySpanTest(tests_data_path);
  DoNotRequireEofTest(tests_data_path);
  JsonUnionStructTest();
#else
  // Guard against -Wunused-parameter.
  (void)tests_data_path;
#endif

  UtilConvertCase();

  FuzzTest1();
  FuzzTest2();

  TriviallyCopyableTest();
  ErrorTest();
  ValueTest();
  EnumValueTest();
  NestedListTest();
  EnumStringsTest();
  EnumNamesTest();
  EnumOutOfRangeTest();
  IntegerOutOfRangeTest();
  IntegerBoundaryTest();
  UnicodeTest();
  UnicodeTestAllowNonUTF8();
  UnicodeTestGenerateTextFailsOnNonUTF8();
  UnicodeSurrogatesTest();
  UnicodeInvalidSurrogatesTest();
  InvalidUTF8Test();
  UnknownFieldsTest();
  ParseUnionTest();
  ValidSameNameDifferentNamespaceTest();
  ConformTest();
  ParseProtoBufAsciiTest();
  TypeAliasesTest();
  EndianSwapTest();
  CreateSharedStringTest();
  FlexBuffersTest();
  FlexBuffersReuseBugTest();
  FlexBuffersDeprecatedTest();
  UninitializedVectorTest();
  EqualOperatorTest();
  NumericUtilsTest();
  IsAsciiUtilsTest();
  ValidFloatTest();
  InvalidFloatTest();
  FixedLengthArrayTest();
  NativeTypeTest();
  OptionalScalarsTest();
  ParseFlexbuffersFromJsonWithNullTest();
  FlatbuffersSpanTest();
  FixedLengthArrayConstructorTest();
  FixedLengthArrayOperatorEqualTest();
  FieldIdentifierTest();
  StringVectorDefaultsTest();
  FlexBuffersFloatingPointTest();
  FlatbuffersIteratorsTest();
  WarningsAsErrorsTest();
  NestedVerifierTest();
  PrivateAnnotationsLeaks();
  JsonUnsortedArrayTest();
  VectorSpanTest();
  NativeInlineTableVectorTest();
  FixedSizedScalarKeyInStructTest();
  StructKeyInStructTest();
  NestedStructKeyInStructTest();
  FixedSizedStructArrayKeyInStructTest();
  EmbeddedSchemaAccess();
  Offset64Tests();
  UnionUnderlyingTypeTest();
  return 0;
}
}  // namespace
}  // namespace tests
}  // namespace flatbuffers

int main(int argc, const char *argv[]) {
  std::string tests_data_path = "tests/";

  for (int argi = 1; argi < argc; argi++) {
    std::string arg = argv[argi];
    if (arg == "--test_path") {
      if (++argi >= argc) {
        fprintf(stderr, "error: missing path following: %s\n", arg.c_str());
        exit(1);
      }
      // Override default path if provided one.
      tests_data_path = argv[argi];

    } else {
      fprintf(stderr, "error: Unknown argument: %s\n", arg.c_str());
      exit(1);
    }
  }

  InitTestEngine();

  std::string req_locale;
  if (flatbuffers::ReadEnvironmentVariable("FLATBUFFERS_TEST_LOCALE",
                                           &req_locale)) {
    TEST_OUTPUT_LINE("The environment variable FLATBUFFERS_TEST_LOCALE=%s",
                     req_locale.c_str());
    req_locale = flatbuffers::RemoveStringQuotes(req_locale);
    std::string the_locale;
    TEST_ASSERT_FUNC(
        flatbuffers::SetGlobalTestLocale(req_locale.c_str(), &the_locale));
    TEST_OUTPUT_LINE("The global C-locale changed: %s", the_locale.c_str());
  }

#ifdef FLATBUFFERS_TEST_PATH_PREFIX
  tests_data_path =
      FLATBUFFERS_STRING(FLATBUFFERS_TEST_PATH_PREFIX) + tests_data_path;
#endif

  flatbuffers::tests::FlatBufferTests(tests_data_path);
  FlatBufferBuilderTest();

  if (!testing_fails) {
    TEST_OUTPUT_LINE("ALL TESTS PASSED");
  } else {
    TEST_OUTPUT_LINE("%d FAILED TESTS", testing_fails);
  }
  return CloseTestEngine();
}
