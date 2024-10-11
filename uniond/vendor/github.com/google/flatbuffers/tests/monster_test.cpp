#include "monster_test.h"

#include <limits>
#include <vector>

#include "flatbuffers/base.h"
#include "flatbuffers/flatbuffer_builder.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/registry.h"
#include "flatbuffers/verifier.h"
#include "is_quiet_nan.h"
#include "monster_extra_generated.h"
#include "monster_test_generated.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

// Shortcuts for the infinity.
static const auto infinity_f = std::numeric_limits<float>::infinity();
static const auto infinity_d = std::numeric_limits<double>::infinity();

using namespace MyGame::Example;

// example of how to build up a serialized buffer algorithmically:
flatbuffers::DetachedBuffer CreateFlatBufferTest(std::string &buffer) {
  flatbuffers::FlatBufferBuilder builder;

  auto vec = Vec3(1, 2, 3, 0, Color_Red, Test(10, 20));

  auto name = builder.CreateString("MyMonster");

  // Use the initializer_list specialization of CreateVector.
  auto inventory =
      builder.CreateVector<uint8_t>({ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 });

  // Alternatively, create the vector first, and fill in data later:
  // unsigned char *inv_buf = nullptr;
  // auto inventory = builder.CreateUninitializedVector<unsigned char>(
  //                                                              10, &inv_buf);
  // memcpy(inv_buf, inv_data, 10);

  Test tests[] = { Test(10, 20), Test(30, 40) };
  auto testv = builder.CreateVectorOfStructs(tests, 2);

  // Create a vector of structures from a lambda.
  auto testv2 = builder.CreateVectorOfStructs<Test>(
      2, [&](size_t i, Test *s) -> void { *s = tests[i]; });

  // create monster with very few fields set:
  // (same functionality as CreateMonster below, but sets fields manually)
  flatbuffers::Offset<Monster> mlocs[3];
  auto fred = builder.CreateString("Fred");
  auto barney = builder.CreateString("Barney");
  auto wilma = builder.CreateString("Wilma");
  MonsterBuilder mb1(builder);
  mb1.add_name(fred);
  mlocs[0] = mb1.Finish();
  MonsterBuilder mb2(builder);
  mb2.add_name(barney);
  mb2.add_hp(1000);
  mlocs[1] = mb2.Finish();
  MonsterBuilder mb3(builder);
  mb3.add_name(wilma);
  mlocs[2] = mb3.Finish();

  // Create an array of strings. Also test string pooling, and lambdas.
  auto vecofstrings =
      builder.CreateVector<flatbuffers::Offset<flatbuffers::String>>(
          4,
          [](size_t i, flatbuffers::FlatBufferBuilder *b)
              -> flatbuffers::Offset<flatbuffers::String> {
            static const char *names[] = { "bob", "fred", "bob", "fred" };
            return b->CreateSharedString(names[i]);
          },
          &builder);

  // Creating vectors of strings in one convenient call.
  std::vector<std::string> names2;
  names2.push_back("jane");
  names2.push_back("mary");
  auto vecofstrings2 = builder.CreateVectorOfStrings(names2);

  // Creating vectors from types that are different from std::string
  std::vector<const char *> names3;
  names3.push_back("foo");
  names3.push_back("bar");
  builder.CreateVectorOfStrings(names3);  // Also an accepted type

#ifdef FLATBUFFERS_HAS_STRING_VIEW
  std::vector<flatbuffers::string_view> names4;
  names3.push_back("baz");
  names3.push_back("quux");
  builder.CreateVectorOfStrings(names4);  // Also an accepted type
#endif

  // Make sure the template deduces an initializer as std::vector<std::string>
  builder.CreateVectorOfStrings({ "hello", "world" });

  // Create many vectors of strings
  std::vector<std::string> manyNames;
  for (auto i = 0; i < 100; i++) { manyNames.push_back("john_doe"); }
  auto manyNamesVec = builder.CreateVectorOfStrings(manyNames);
  TEST_EQ(false, manyNamesVec.IsNull());
  auto manyNamesVec2 =
      builder.CreateVectorOfStrings(manyNames.cbegin(), manyNames.cend());
  TEST_EQ(false, manyNamesVec2.IsNull());

  // Create an array of sorted tables, can be used with binary search when read:
  auto vecoftables = builder.CreateVectorOfSortedTables(mlocs, 3);

  // Create an array of sorted structs,
  // can be used with binary search when read:
  std::vector<Ability> abilities;
  abilities.push_back(Ability(4, 40));
  abilities.push_back(Ability(3, 30));
  abilities.push_back(Ability(2, 20));
  abilities.push_back(Ability(0, 0));
  auto vecofstructs = builder.CreateVectorOfSortedStructs(&abilities);

  flatbuffers::Offset<Stat> mlocs_stats[1];
  auto miss = builder.CreateString("miss");
  StatBuilder mb_miss(builder);
  mb_miss.add_id(miss);
  mb_miss.add_val(0);
  mb_miss.add_count(0);  // key
  mlocs_stats[0] = mb_miss.Finish();
  auto vec_of_stats = builder.CreateVectorOfSortedTables(mlocs_stats, 1);

  // Create a nested FlatBuffer.
  // Nested FlatBuffers are stored in a ubyte vector, which can be convenient
  // since they can be memcpy'd around much easier than other FlatBuffer
  // values. They have little overhead compared to storing the table directly.
  // As a test, create a mostly empty Monster buffer:
  flatbuffers::FlatBufferBuilder nested_builder;
  auto nmloc = CreateMonster(nested_builder, nullptr, 0, 0,
                             nested_builder.CreateString("NestedMonster"));
  FinishMonsterBuffer(nested_builder, nmloc);
  // Now we can store the buffer in the parent. Note that by default, vectors
  // are only aligned to their elements or size field, so in this case if the
  // buffer contains 64-bit elements, they may not be correctly aligned. We fix
  // that with:
  builder.ForceVectorAlignment(nested_builder.GetSize(), sizeof(uint8_t),
                               nested_builder.GetBufferMinAlignment());
  // If for whatever reason you don't have the nested_builder available, you
  // can substitute flatbuffers::largest_scalar_t (64-bit) for the alignment, or
  // the largest force_align value in your schema if you're using it.
  auto nested_flatbuffer_vector = builder.CreateVector(
      nested_builder.GetBufferPointer(), nested_builder.GetSize());

  // Test a nested FlexBuffer:
  flexbuffers::Builder flexbuild;
  flexbuild.Int(1234);
  flexbuild.Finish();
  auto flex = builder.CreateVector(flexbuild.GetBuffer());
  // Test vector of enums.
  Color colors[] = { Color_Blue, Color_Green };
  // We use this special creation function because we have an array of
  // pre-C++11 (enum class) enums whose size likely is int, yet its declared
  // type in the schema is byte.
  auto vecofcolors = builder.CreateVectorScalarCast<uint8_t, Color>(colors, 2);

  // shortcut for creating monster with all fields set:
  auto mloc = CreateMonster(
      builder, &vec, 150, 80, name, inventory, Color_Blue, Any_Monster,
      mlocs[1].Union(),  // Store a union.
      testv, vecofstrings, vecoftables, 0, nested_flatbuffer_vector, 0, false,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 3.14159f, 3.0f, 0.0f, vecofstrings2,
      vecofstructs, flex, testv2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      AnyUniqueAliases_NONE, 0, AnyAmbiguousAliases_NONE, 0, vecofcolors,
      MyGame::Example::Race_None, 0, vec_of_stats);

  FinishMonsterBuffer(builder, mloc);

  // clang-format off
  #ifdef FLATBUFFERS_TEST_VERBOSE
  // print byte data for debugging:
  auto p = builder.GetBufferPointer();
  for (flatbuffers::uoffset_t i = 0; i < builder.GetSize(); i++)
    printf("%d ", p[i]);
  #endif
  // clang-format on

  // return the buffer for the caller to use.
  auto bufferpointer =
      reinterpret_cast<const char *>(builder.GetBufferPointer());
  buffer.assign(bufferpointer, bufferpointer + builder.GetSize());

  return builder.Release();
}

//  example of accessing a buffer loaded in memory:
void AccessFlatBufferTest(const uint8_t *flatbuf, size_t length, bool pooled) {
  // First, verify the buffers integrity (optional)
  flatbuffers::Verifier verifier(flatbuf, length);
  std::vector<uint8_t> flex_reuse_tracker;
  verifier.SetFlexReuseTracker(&flex_reuse_tracker);
  TEST_EQ(VerifyMonsterBuffer(verifier), true);

  // clang-format off
  #ifdef FLATBUFFERS_TRACK_VERIFIER_BUFFER_SIZE
    std::vector<uint8_t> test_buff;
    test_buff.resize(length * 2);
    std::memcpy(&test_buff[0], flatbuf, length);
    std::memcpy(&test_buff[length], flatbuf, length);

    flatbuffers::Verifier verifier1(&test_buff[0], length);
    TEST_EQ(VerifyMonsterBuffer(verifier1), true);
    TEST_EQ(verifier1.GetComputedSize(), length);

    flatbuffers::Verifier verifier2(&test_buff[length], length);
    TEST_EQ(VerifyMonsterBuffer(verifier2), true);
    TEST_EQ(verifier2.GetComputedSize(), length);
  #endif
  // clang-format on

  TEST_EQ(strcmp(MonsterIdentifier(), "MONS"), 0);
  TEST_EQ(MonsterBufferHasIdentifier(flatbuf), true);
  TEST_EQ(strcmp(MonsterExtension(), "mon"), 0);

  // Access the buffer from the root.
  auto monster = GetMonster(flatbuf);

  TEST_EQ(monster->hp(), 80);
  TEST_EQ(monster->mana(), 150);  // default
  TEST_EQ_STR(monster->name()->c_str(), "MyMonster");
  // Can't access the following field, it is deprecated in the schema,
  // which means accessors are not generated:
  // monster.friendly()

  auto pos = monster->pos();
  TEST_NOTNULL(pos);
  TEST_EQ(pos->z(), 3);
  TEST_EQ(pos->test3().a(), 10);
  TEST_EQ(pos->test3().b(), 20);

  auto inventory = monster->inventory();
  TEST_EQ(VectorLength(inventory), 10UL);  // Works even if inventory is null.
  TEST_NOTNULL(inventory);
  unsigned char inv_data[] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
  // Check compatibilty of iterators with STL.
  std::vector<unsigned char> inv_vec(inventory->begin(), inventory->end());
  size_t n = 0;
  for (auto it = inventory->begin(); it != inventory->end(); ++it, ++n) {
    auto indx = it - inventory->begin();
    TEST_EQ(*it, inv_vec.at(indx));  // Use bounds-check.
    TEST_EQ(*it, inv_data[indx]);
  }
  TEST_EQ(n, inv_vec.size());

  n = 0;
  for (auto it = inventory->cbegin(); it != inventory->cend(); ++it, ++n) {
    auto indx = it - inventory->cbegin();
    TEST_EQ(*it, inv_vec.at(indx));  // Use bounds-check.
    TEST_EQ(*it, inv_data[indx]);
  }
  TEST_EQ(n, inv_vec.size());

  n = 0;
  for (auto it = inventory->rbegin(); it != inventory->rend(); ++it, ++n) {
    auto indx = inventory->rend() - it - 1;
    TEST_EQ(*it, inv_vec.at(indx));  // Use bounds-check.
    TEST_EQ(*it, inv_data[indx]);
  }
  TEST_EQ(n, inv_vec.size());

  n = 0;
  for (auto it = inventory->crbegin(); it != inventory->crend(); ++it, ++n) {
    auto indx = inventory->crend() - it - 1;
    TEST_EQ(*it, inv_vec.at(indx));  // Use bounds-check.
    TEST_EQ(*it, inv_data[indx]);
  }
  TEST_EQ(n, inv_vec.size());

  TEST_EQ(monster->color(), Color_Blue);

  // Example of accessing a union:
  TEST_EQ(monster->test_type(), Any_Monster);  // First make sure which it is.
  auto monster2 = reinterpret_cast<const Monster *>(monster->test());
  TEST_NOTNULL(monster2);
  TEST_EQ_STR(monster2->name()->c_str(), "Fred");

  // Example of accessing a vector of strings:
  auto vecofstrings = monster->testarrayofstring();
  TEST_EQ(vecofstrings->size(), 4U);
  TEST_EQ_STR(vecofstrings->Get(0)->c_str(), "bob");
  TEST_EQ_STR(vecofstrings->Get(1)->c_str(), "fred");
  if (pooled) {
    // These should have pointer equality because of string pooling.
    TEST_EQ(vecofstrings->Get(0)->c_str(), vecofstrings->Get(2)->c_str());
    TEST_EQ(vecofstrings->Get(1)->c_str(), vecofstrings->Get(3)->c_str());
  }

  auto vecofstrings2 = monster->testarrayofstring2();
  if (vecofstrings2) {
    TEST_EQ(vecofstrings2->size(), 2U);
    TEST_EQ_STR(vecofstrings2->Get(0)->c_str(), "jane");
    TEST_EQ_STR(vecofstrings2->Get(1)->c_str(), "mary");
  }

  // Example of accessing a vector of tables:
  auto vecoftables = monster->testarrayoftables();
  TEST_EQ(vecoftables->size(), 3U);
  for (auto it = vecoftables->begin(); it != vecoftables->end(); ++it) {
    TEST_EQ(strlen(it->name()->c_str()) >= 4, true);
  }
  TEST_EQ_STR(vecoftables->Get(0)->name()->c_str(), "Barney");
  TEST_EQ(vecoftables->Get(0)->hp(), 1000);
  TEST_EQ_STR(vecoftables->Get(1)->name()->c_str(), "Fred");
  TEST_EQ_STR(vecoftables->Get(2)->name()->c_str(), "Wilma");
  TEST_NOTNULL(vecoftables->LookupByKey("Barney"));
  TEST_NOTNULL(vecoftables->LookupByKey("Fred"));
  TEST_NOTNULL(vecoftables->LookupByKey("Wilma"));

  // Verify the same objects are returned for char*-based and string-based
  // lookups.
  TEST_EQ(vecoftables->LookupByKey("Barney"),
          vecoftables->LookupByKey(std::string("Barney")));
  TEST_EQ(vecoftables->LookupByKey("Fred"),
          vecoftables->LookupByKey(std::string("Fred")));
  TEST_EQ(vecoftables->LookupByKey("Wilma"),
          vecoftables->LookupByKey(std::string("Wilma")));

#ifdef FLATBUFFERS_HAS_STRING_VIEW
  // Tests for LookupByKey with a key that is a truncated
  // version of a longer, invalid key.
  const std::string invalid_key = "Barney123";
  std::string_view valid_truncated_key = invalid_key;
  valid_truncated_key.remove_suffix(3);  // "Barney"
  TEST_NOTNULL(vecoftables->LookupByKey(valid_truncated_key));
  TEST_EQ(vecoftables->LookupByKey("Barney"),
          vecoftables->LookupByKey(valid_truncated_key));

  // Tests for LookupByKey with a key that is a truncated
  // version of a longer, valid key.
  const std::string valid_key = "Barney";
  std::string_view invalid_truncated_key = valid_key;
  invalid_truncated_key.remove_suffix(3);  // "Bar"
  TEST_NULL(vecoftables->LookupByKey(invalid_truncated_key));
#endif  // FLATBUFFERS_HAS_STRING_VIEW

  // Test accessing a vector of sorted structs
  auto vecofstructs = monster->testarrayofsortedstruct();
  if (vecofstructs) {  // not filled in monster_test.bfbs
    for (flatbuffers::uoffset_t i = 0; i < vecofstructs->size() - 1; i++) {
      auto left = vecofstructs->Get(i);
      auto right = vecofstructs->Get(i + 1);
      TEST_EQ(true, (left->KeyCompareLessThan(right)));
    }
    TEST_NOTNULL(vecofstructs->LookupByKey(0));  // test default value
    TEST_NOTNULL(vecofstructs->LookupByKey(3));
    TEST_EQ(static_cast<const Ability *>(nullptr),
            vecofstructs->LookupByKey(5));
  }

  if (auto vec_of_stat = monster->scalar_key_sorted_tables()) {
    auto stat_0 = vec_of_stat->LookupByKey(static_cast<uint16_t>(0u));
    TEST_NOTNULL(stat_0);
    TEST_NOTNULL(stat_0->id());
    TEST_EQ(0, stat_0->count());
    TEST_EQ_STR("miss", stat_0->id()->c_str());
  }

  // Test nested FlatBuffers if available:
  auto nested_buffer = monster->testnestedflatbuffer();
  if (nested_buffer) {
    // nested_buffer is a vector of bytes you can memcpy. However, if you
    // actually want to access the nested data, this is a convenient
    // accessor that directly gives you the root table:
    auto nested_monster = monster->testnestedflatbuffer_nested_root();
    TEST_EQ_STR(nested_monster->name()->c_str(), "NestedMonster");
  }

  // Test flexbuffer if available:
  auto flex = monster->flex();
  // flex is a vector of bytes you can memcpy etc.
  TEST_EQ(flex->size(), 4);  // Encoded FlexBuffer bytes.
  // However, if you actually want to access the nested data, this is a
  // convenient accessor that directly gives you the root value:
  TEST_EQ(monster->flex_flexbuffer_root().AsInt16(), 1234);

  // Test vector of enums:
  auto colors = monster->vector_of_enums();
  if (colors) {
    TEST_EQ(colors->size(), 2);
    TEST_EQ(colors->Get(0), Color_Blue);
    TEST_EQ(colors->Get(1), Color_Green);
  }

  // Since Flatbuffers uses explicit mechanisms to override the default
  // compiler alignment, double check that the compiler indeed obeys them:
  // (Test consists of a short and byte):
  TEST_EQ(flatbuffers::AlignOf<Test>(), 2UL);
  TEST_EQ(sizeof(Test), 4UL);

  const flatbuffers::Vector<const Test *> *tests_array[] = {
    monster->test4(),
    monster->test5(),
  };
  for (size_t i = 0; i < sizeof(tests_array) / sizeof(tests_array[0]); ++i) {
    auto tests = tests_array[i];
    TEST_NOTNULL(tests);
    auto test_0 = tests->Get(0);
    auto test_1 = tests->Get(1);
    TEST_EQ(test_0->a(), 10);
    TEST_EQ(test_0->b(), 20);
    TEST_EQ(test_1->a(), 30);
    TEST_EQ(test_1->b(), 40);
    for (auto it = tests->begin(); it != tests->end(); ++it) {
      TEST_EQ(it->a() == 10 || it->a() == 30, true);  // Just testing iterators.
    }
  }

  // Checking for presence of fields:
  TEST_EQ(flatbuffers::IsFieldPresent(monster, Monster::VT_HP), true);
  TEST_EQ(flatbuffers::IsFieldPresent(monster, Monster::VT_MANA), false);

  // Obtaining a buffer from a root:
  TEST_EQ(GetBufferStartFromRootPointer(monster), flatbuf);
}

// Change a FlatBuffer in-place, after it has been constructed.
void MutateFlatBuffersTest(uint8_t *flatbuf, std::size_t length) {
  // Get non-const pointer to root.
  auto monster = GetMutableMonster(flatbuf);

  // Each of these tests mutates, then tests, then set back to the original,
  // so we can test that the buffer in the end still passes our original test.
  auto hp_ok = monster->mutate_hp(10);
  TEST_EQ(hp_ok, true);  // Field was present.
  TEST_EQ(monster->hp(), 10);
  // Mutate to default value
  auto hp_ok_default = monster->mutate_hp(100);
  TEST_EQ(hp_ok_default, true);  // Field was present.
  TEST_EQ(monster->hp(), 100);
  // Test that mutate to default above keeps field valid for further mutations
  auto hp_ok_2 = monster->mutate_hp(20);
  TEST_EQ(hp_ok_2, true);
  TEST_EQ(monster->hp(), 20);
  monster->mutate_hp(80);

  // Monster originally at 150 mana (default value)
  auto mana_default_ok = monster->mutate_mana(150);  // Mutate to default value.
  TEST_EQ(mana_default_ok,
          true);  // Mutation should succeed, because default value.
  TEST_EQ(monster->mana(), 150);
  auto mana_ok = monster->mutate_mana(10);
  TEST_EQ(mana_ok, false);  // Field was NOT present, because default value.
  TEST_EQ(monster->mana(), 150);

  // Mutate structs.
  auto pos = monster->mutable_pos();
  auto &test3 = pos->mutable_test3();  // Struct inside a struct.
  test3.mutate_a(50);                  // Struct fields never fail.
  TEST_EQ(test3.a(), 50);
  test3.mutate_a(10);

  // Mutate vectors.
  auto inventory = monster->mutable_inventory();
  inventory->Mutate(9, 100);
  TEST_EQ(inventory->Get(9), 100);
  inventory->Mutate(9, 9);

  auto tables = monster->mutable_testarrayoftables();
  auto first = tables->GetMutableObject(0);
  TEST_EQ(first->hp(), 1000);
  first->mutate_hp(0);
  TEST_EQ(first->hp(), 0);
  first->mutate_hp(1000);

  // Test for each loop over mutable entries
  for (auto item : *tables) {
    TEST_EQ(item->hp(), 1000);
    item->mutate_hp(0);
    TEST_EQ(item->hp(), 0);
    item->mutate_hp(1000);
    break;  // one iteration is enough, just testing compilation
  }

  // Mutate via LookupByKey
  TEST_NOTNULL(tables->MutableLookupByKey("Barney"));
  TEST_EQ(static_cast<Monster *>(nullptr),
          tables->MutableLookupByKey("DoesntExist"));
  TEST_EQ(tables->MutableLookupByKey("Barney")->hp(), 1000);
  TEST_EQ(tables->MutableLookupByKey("Barney")->mutate_hp(0), true);
  TEST_EQ(tables->LookupByKey("Barney")->hp(), 0);
  TEST_EQ(tables->MutableLookupByKey("Barney")->mutate_hp(1000), true);

  // Run the verifier and the regular test to make sure we didn't trample on
  // anything.
  AccessFlatBufferTest(flatbuf, length);
}

// Unpack a FlatBuffer into objects.
void ObjectFlatBuffersTest(uint8_t *flatbuf) {
  // Optional: we can specify resolver and rehasher functions to turn hashed
  // strings into object pointers and back, to implement remote references
  // and such.
  auto resolver = flatbuffers::resolver_function_t(
      [](void **pointer_adr, flatbuffers::hash_value_t hash) {
        (void)pointer_adr;
        (void)hash;
        // Don't actually do anything, leave variable null.
      });
  auto rehasher = flatbuffers::rehasher_function_t(
      [](void *pointer) -> flatbuffers::hash_value_t {
        (void)pointer;
        return 0;
      });

  // Turn a buffer into C++ objects.
  auto monster1 = UnPackMonster(flatbuf, &resolver);

  // Re-serialize the data.
  flatbuffers::FlatBufferBuilder fbb1;
  fbb1.Finish(CreateMonster(fbb1, monster1.get(), &rehasher),
              MonsterIdentifier());

  // Unpack again, and re-serialize again.
  auto monster2 = UnPackMonster(fbb1.GetBufferPointer(), &resolver);
  flatbuffers::FlatBufferBuilder fbb2;
  fbb2.Finish(CreateMonster(fbb2, monster2.get(), &rehasher),
              MonsterIdentifier());

  // Now we've gone full round-trip, the two buffers should match.
  const auto len1 = fbb1.GetSize();
  const auto len2 = fbb2.GetSize();
  TEST_EQ(len1, len2);
  TEST_EQ(memcmp(fbb1.GetBufferPointer(), fbb2.GetBufferPointer(), len1), 0);

  // Test it with the original buffer test to make sure all data survived.
  AccessFlatBufferTest(fbb2.GetBufferPointer(), len2, false);

  // Test accessing fields, similar to AccessFlatBufferTest above.
  CheckMonsterObject(monster2.get());

  // Test object copy.
  MonsterT monster3 = *monster2;
  flatbuffers::FlatBufferBuilder fbb3;
  fbb3.Finish(CreateMonster(fbb3, &monster3, &rehasher), MonsterIdentifier());
  const auto len3 = fbb3.GetSize();
  TEST_EQ(len2, len3);
  TEST_EQ(memcmp(fbb2.GetBufferPointer(), fbb3.GetBufferPointer(), len2), 0);
  // Delete monster1 and monster2, then test accessing fields in monster3.
  monster1.reset();
  monster2.reset();
  CheckMonsterObject(&monster3);
}

// Utility function to check a Monster object.
void CheckMonsterObject(MonsterT *monster2) {
  TEST_EQ(monster2->hp, 80);
  TEST_EQ(monster2->mana, 150);  // default
  TEST_EQ_STR(monster2->name.c_str(), "MyMonster");

  auto &pos = monster2->pos;
  TEST_NOTNULL(pos);
  TEST_EQ(pos->z(), 3);
  TEST_EQ(pos->test3().a(), 10);
  TEST_EQ(pos->test3().b(), 20);

  auto &inventory = monster2->inventory;
  TEST_EQ(inventory.size(), 10UL);
  unsigned char inv_data[] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
  for (auto it = inventory.begin(); it != inventory.end(); ++it)
    TEST_EQ(*it, inv_data[it - inventory.begin()]);

  TEST_EQ(monster2->color, Color_Blue);

  auto monster3 = monster2->test.AsMonster();
  TEST_NOTNULL(monster3);
  TEST_EQ_STR(monster3->name.c_str(), "Fred");

  auto &vecofstrings = monster2->testarrayofstring;
  TEST_EQ(vecofstrings.size(), 4U);
  TEST_EQ_STR(vecofstrings[0].c_str(), "bob");
  TEST_EQ_STR(vecofstrings[1].c_str(), "fred");

  auto &vecofstrings2 = monster2->testarrayofstring2;
  TEST_EQ(vecofstrings2.size(), 2U);
  TEST_EQ_STR(vecofstrings2[0].c_str(), "jane");
  TEST_EQ_STR(vecofstrings2[1].c_str(), "mary");

  auto &vecoftables = monster2->testarrayoftables;
  TEST_EQ(vecoftables.size(), 3U);
  TEST_EQ_STR(vecoftables[0]->name.c_str(), "Barney");
  TEST_EQ(vecoftables[0]->hp, 1000);
  TEST_EQ_STR(vecoftables[1]->name.c_str(), "Fred");
  TEST_EQ_STR(vecoftables[2]->name.c_str(), "Wilma");

  auto &tests = monster2->test4;
  TEST_EQ(tests[0].a(), 10);
  TEST_EQ(tests[0].b(), 20);
  TEST_EQ(tests[1].a(), 30);
  TEST_EQ(tests[1].b(), 40);
}

// Prefix a FlatBuffer with a size field.
void SizePrefixedTest() {
  // Create size prefixed buffer.
  flatbuffers::FlatBufferBuilder fbb;
  FinishSizePrefixedMonsterBuffer(
      fbb, CreateMonster(fbb, nullptr, 200, 300, fbb.CreateString("bob")));

  // Verify it.
  flatbuffers::Verifier verifier(fbb.GetBufferPointer(), fbb.GetSize());
  TEST_EQ(VerifySizePrefixedMonsterBuffer(verifier), true);

  // The prefixed size doesn't include itself, so substract the size of the
  // prefix
  TEST_EQ(GetPrefixedSize(fbb.GetBufferPointer()),
          fbb.GetSize() - sizeof(uoffset_t));

  // Getting the buffer length does include the prefix size, so it should be the
  // full lenght.
  TEST_EQ(GetSizePrefixedBufferLength(fbb.GetBufferPointer()), fbb.GetSize());

  // Access it.
  auto m = GetSizePrefixedMonster(fbb.GetBufferPointer());
  TEST_EQ(m->mana(), 200);
  TEST_EQ(m->hp(), 300);
  TEST_EQ_STR(m->name()->c_str(), "bob");

  {
    // Verify that passing a larger size is OK, but not a smaller
    flatbuffers::Verifier verifier_larger(fbb.GetBufferPointer(),
                                          fbb.GetSize() + 10);
    TEST_EQ(VerifySizePrefixedMonsterBuffer(verifier_larger), true);

    flatbuffers::Verifier verifier_smaller(fbb.GetBufferPointer(),
                                           fbb.GetSize() - 10);
    TEST_EQ(VerifySizePrefixedMonsterBuffer(verifier_smaller), false);
  }
}

void TestMonsterExtraFloats(const std::string &tests_data_path) {
#if defined(FLATBUFFERS_HAS_NEW_STRTOD) && (FLATBUFFERS_HAS_NEW_STRTOD > 0)
  TEST_EQ(is_quiet_nan(1.0), false);
  TEST_EQ(is_quiet_nan(infinity_d), false);
  TEST_EQ(is_quiet_nan(-infinity_f), false);
  TEST_EQ(is_quiet_nan(std::numeric_limits<float>::quiet_NaN()), true);
  TEST_EQ(is_quiet_nan(std::numeric_limits<double>::quiet_NaN()), true);

  using namespace flatbuffers;
  using namespace MyGame;
  // Load FlatBuffer schema (.fbs) from disk.
  std::string schemafile;
  TEST_EQ(LoadFile((tests_data_path + "monster_extra.fbs").c_str(), false,
                   &schemafile),
          true);
  // Parse schema first, so we can use it to parse the data after.
  Parser parser;
  auto include_test_path = ConCatPathFileName(tests_data_path, "include_test");
  const char *include_directories[] = { tests_data_path.c_str(),
                                        include_test_path.c_str(), nullptr };
  TEST_EQ(parser.Parse(schemafile.c_str(), include_directories), true);
  // Create empty extra and store to json.
  parser.opts.output_default_scalars_in_json = true;
  parser.opts.output_enum_identifiers = true;
  FlatBufferBuilder builder;
  const auto def_root = MonsterExtraBuilder(builder).Finish();
  FinishMonsterExtraBuffer(builder, def_root);
  const auto def_obj = builder.GetBufferPointer();
  const auto def_extra = GetMonsterExtra(def_obj);
  TEST_NOTNULL(def_extra);
  TEST_EQ(is_quiet_nan(def_extra->f0()), true);
  TEST_EQ(is_quiet_nan(def_extra->f1()), true);
  TEST_EQ(def_extra->f2(), +infinity_f);
  TEST_EQ(def_extra->f3(), -infinity_f);
  TEST_EQ(is_quiet_nan(def_extra->d0()), true);
  TEST_EQ(is_quiet_nan(def_extra->d1()), true);
  TEST_EQ(def_extra->d2(), +infinity_d);
  TEST_EQ(def_extra->d3(), -infinity_d);
  std::string jsongen;
  auto result = GenText(parser, def_obj, &jsongen);
  TEST_NULL(result);
  // Check expected default values.
  TEST_EQ(std::string::npos != jsongen.find("f0: nan"), true);
  TEST_EQ(std::string::npos != jsongen.find("f1: nan"), true);
  TEST_EQ(std::string::npos != jsongen.find("f2: inf"), true);
  TEST_EQ(std::string::npos != jsongen.find("f3: -inf"), true);
  TEST_EQ(std::string::npos != jsongen.find("d0: nan"), true);
  TEST_EQ(std::string::npos != jsongen.find("d1: nan"), true);
  TEST_EQ(std::string::npos != jsongen.find("d2: inf"), true);
  TEST_EQ(std::string::npos != jsongen.find("d3: -inf"), true);
  // Parse 'mosterdata_extra.json'.
  const auto extra_base = tests_data_path + "monsterdata_extra";
  jsongen = "";
  TEST_EQ(LoadFile((extra_base + ".json").c_str(), false, &jsongen), true);
  TEST_EQ(parser.Parse(jsongen.c_str()), true);
  const auto test_file = parser.builder_.GetBufferPointer();
  const auto test_size = parser.builder_.GetSize();
  Verifier verifier(test_file, test_size);
  TEST_ASSERT(VerifyMonsterExtraBuffer(verifier));
  const auto extra = GetMonsterExtra(test_file);
  TEST_NOTNULL(extra);
  TEST_EQ(is_quiet_nan(extra->f0()), true);
  TEST_EQ(is_quiet_nan(extra->f1()), true);
  TEST_EQ(extra->f2(), +infinity_f);
  TEST_EQ(extra->f3(), -infinity_f);
  TEST_EQ(is_quiet_nan(extra->d0()), true);
  TEST_EQ(extra->d1(), +infinity_d);
  TEST_EQ(extra->d2(), -infinity_d);
  TEST_EQ(is_quiet_nan(extra->d3()), true);
  TEST_NOTNULL(extra->fvec());
  TEST_EQ(extra->fvec()->size(), 4);
  TEST_EQ(extra->fvec()->Get(0), 1.0f);
  TEST_EQ(extra->fvec()->Get(1), -infinity_f);
  TEST_EQ(extra->fvec()->Get(2), +infinity_f);
  TEST_EQ(is_quiet_nan(extra->fvec()->Get(3)), true);
  TEST_NOTNULL(extra->dvec());
  TEST_EQ(extra->dvec()->size(), 4);
  TEST_EQ(extra->dvec()->Get(0), 2.0);
  TEST_EQ(extra->dvec()->Get(1), +infinity_d);
  TEST_EQ(extra->dvec()->Get(2), -infinity_d);
  TEST_EQ(is_quiet_nan(extra->dvec()->Get(3)), true);
#endif
}

void EnumNamesTest() {
  TEST_EQ_STR("Red", EnumNameColor(Color_Red));
  TEST_EQ_STR("Green", EnumNameColor(Color_Green));
  TEST_EQ_STR("Blue", EnumNameColor(Color_Blue));
  // Check that Color to string don't crash while decode a mixture of Colors.
  // 1) Example::Color enum is enum with unfixed underlying type.
  // 2) Valid enum range: [0; 2^(ceil(log2(Color_ANY))) - 1].
  // Consequence: A value is out of this range will lead to UB (since C++17).
  // For details see C++17 standard or explanation on the SO:
  // stackoverflow.com/questions/18195312/what-happens-if-you-static-cast-invalid-value-to-enum-class
  TEST_EQ_STR("", EnumNameColor(static_cast<Color>(0)));
  TEST_EQ_STR("", EnumNameColor(static_cast<Color>(Color_ANY - 1)));
  TEST_EQ_STR("", EnumNameColor(static_cast<Color>(Color_ANY + 1)));
}

void TypeAliasesTest() {
  flatbuffers::FlatBufferBuilder builder;

  builder.Finish(CreateTypeAliases(
      builder, flatbuffers::numeric_limits<int8_t>::min(),
      flatbuffers::numeric_limits<uint8_t>::max(),
      flatbuffers::numeric_limits<int16_t>::min(),
      flatbuffers::numeric_limits<uint16_t>::max(),
      flatbuffers::numeric_limits<int32_t>::min(),
      flatbuffers::numeric_limits<uint32_t>::max(),
      flatbuffers::numeric_limits<int64_t>::min(),
      flatbuffers::numeric_limits<uint64_t>::max(), 2.3f, 2.3));

  auto p = builder.GetBufferPointer();
  auto ta = flatbuffers::GetRoot<TypeAliases>(p);

  TEST_EQ(ta->i8(), flatbuffers::numeric_limits<int8_t>::min());
  TEST_EQ(ta->u8(), flatbuffers::numeric_limits<uint8_t>::max());
  TEST_EQ(ta->i16(), flatbuffers::numeric_limits<int16_t>::min());
  TEST_EQ(ta->u16(), flatbuffers::numeric_limits<uint16_t>::max());
  TEST_EQ(ta->i32(), flatbuffers::numeric_limits<int32_t>::min());
  TEST_EQ(ta->u32(), flatbuffers::numeric_limits<uint32_t>::max());
  TEST_EQ(ta->i64(), flatbuffers::numeric_limits<int64_t>::min());
  TEST_EQ(ta->u64(), flatbuffers::numeric_limits<uint64_t>::max());
  TEST_EQ(ta->f32(), 2.3f);
  TEST_EQ(ta->f64(), 2.3);
  using namespace flatbuffers;  // is_same
  static_assert(is_same<decltype(ta->i8()), int8_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->i16()), int16_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->i32()), int32_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->i64()), int64_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->u8()), uint8_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->u16()), uint16_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->u32()), uint32_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->u64()), uint64_t>::value, "invalid type");
  static_assert(is_same<decltype(ta->f32()), float>::value, "invalid type");
  static_assert(is_same<decltype(ta->f64()), double>::value, "invalid type");
}

// example of parsing text straight into a buffer, and generating
// text back from it:
void ParseAndGenerateTextTest(const std::string &tests_data_path, bool binary) {
  // load FlatBuffer schema (.fbs) and JSON from disk
  std::string schemafile;
  std::string jsonfile;
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "monster_test." + (binary ? "bfbs" : "fbs"))
                  .c_str(),
              binary, &schemafile),
          true);
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "monsterdata_test.golden").c_str(), false,
              &jsonfile),
          true);

  auto include_test_path =
      flatbuffers::ConCatPathFileName(tests_data_path, "include_test");
  const char *include_directories[] = { tests_data_path.c_str(),
                                        include_test_path.c_str(), nullptr };

  // parse schema first, so we can use it to parse the data after
  flatbuffers::Parser parser;
  if (binary) {
    flatbuffers::Verifier verifier(
        reinterpret_cast<const uint8_t *>(schemafile.c_str()),
        schemafile.size());
    TEST_EQ(reflection::VerifySchemaBuffer(verifier), true);
    // auto schema = reflection::GetSchema(schemafile.c_str());
    TEST_EQ(parser.Deserialize(
                reinterpret_cast<const uint8_t *>(schemafile.c_str()),
                schemafile.size()),
            true);
  } else {
    TEST_EQ(parser.Parse(schemafile.c_str(), include_directories), true);
  }
  TEST_EQ(parser.ParseJson(jsonfile.c_str()), true);

  // here, parser.builder_ contains a binary buffer that is the parsed data.

  // First, verify it, just in case:
  flatbuffers::Verifier verifier(parser.builder_.GetBufferPointer(),
                                 parser.builder_.GetSize());
  TEST_EQ(VerifyMonsterBuffer(verifier), true);

  AccessFlatBufferTest(parser.builder_.GetBufferPointer(),
                       parser.builder_.GetSize(), false);

  // to ensure it is correct, we now generate text back from the binary,
  // and compare the two:
  std::string jsongen;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(), jsonfile.c_str());

  // We can also do the above using the convenient Registry that knows about
  // a set of file_identifiers mapped to schemas.
  flatbuffers::Registry registry;
  // Make sure schemas can find their includes.
  registry.AddIncludeDirectory(tests_data_path.c_str());
  registry.AddIncludeDirectory(include_test_path.c_str());
  // Call this with many schemas if possible.
  registry.Register(MonsterIdentifier(),
                    (tests_data_path + "monster_test.fbs").c_str());
  // Now we got this set up, we can parse by just specifying the identifier,
  // the correct schema will be loaded on the fly:
  auto buf = registry.TextToFlatBuffer(jsonfile.c_str(), MonsterIdentifier());
  // If this fails, check registry.lasterror_.
  TEST_NOTNULL(buf.data());
  // Test the buffer, to be sure:
  AccessFlatBufferTest(buf.data(), buf.size(), false);
  // We can use the registry to turn this back into text, in this case it
  // will get the file_identifier from the binary:
  std::string text;
  auto ok = registry.FlatBufferToText(buf.data(), buf.size(), &text);
  // If this fails, check registry.lasterror_.
  TEST_EQ(ok, true);
  TEST_EQ_STR(text.c_str(), jsonfile.c_str());

  // Generate text for UTF-8 strings without escapes.
  std::string jsonfile_utf8;
  TEST_EQ(flatbuffers::LoadFile((tests_data_path + "unicode_test.json").c_str(),
                                false, &jsonfile_utf8),
          true);
  TEST_EQ(parser.Parse(jsonfile_utf8.c_str(), include_directories), true);
  // To ensure it is correct, generate utf-8 text back from the binary.
  std::string jsongen_utf8;
  // request natural printing for utf-8 strings
  parser.opts.natural_utf8 = true;
  parser.opts.strict_json = true;
  TEST_NULL(GenText(parser, parser.builder_.GetBufferPointer(), &jsongen_utf8));
  TEST_EQ_STR(jsongen_utf8.c_str(), jsonfile_utf8.c_str());
}

void UnPackTo(const uint8_t *flatbuf) {
  // Get a monster that has a name and no enemy
  auto orig_monster = GetMonster(flatbuf);
  TEST_EQ_STR(orig_monster->name()->c_str(), "MyMonster");
  TEST_ASSERT(orig_monster->enemy() == nullptr);

  // Create an enemy
  MonsterT *enemy = new MonsterT();
  enemy->name = "Enemy";

  // And create another monster owning the enemy,
  MonsterT mon;
  mon.name = "I'm monster 1";
  mon.enemy.reset(enemy);
  TEST_ASSERT(mon.enemy != nullptr);

  // Assert that all the Monster objects are correct.
  TEST_EQ_STR(mon.name.c_str(), "I'm monster 1");
  TEST_EQ_STR(enemy->name.c_str(), "Enemy");
  TEST_EQ_STR(mon.enemy->name.c_str(), "Enemy");

  // Now unpack monster ("MyMonster") into monster
  orig_monster->UnPackTo(&mon);

  // Monster name should be from monster
  TEST_EQ_STR(mon.name.c_str(), "MyMonster");

  // The monster shouldn't have any enemies, because monster didn't.
  TEST_ASSERT(mon.enemy == nullptr);
}

}  // namespace tests
}  // namespace flatbuffers
