#include "key_field_test.h"

#include <iostream>

#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "key_field/key_field_sample_generated.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

using namespace keyfield::sample;

void FixedSizedScalarKeyInStructTest() {
  flatbuffers::FlatBufferBuilder fbb;
  std::vector<Baz> bazs;
  uint8_t test_array1[4] = { 8, 2, 3, 0 };
  uint8_t test_array2[4] = { 1, 2, 3, 4 };
  uint8_t test_array3[4] = { 2, 2, 3, 4 };
  uint8_t test_array4[4] = { 3, 2, 3, 4 };
  bazs.push_back(Baz(flatbuffers::make_span(test_array1), 4));
  bazs.push_back(Baz(flatbuffers::make_span(test_array2), 1));
  bazs.push_back(Baz(flatbuffers::make_span(test_array3), 2));
  bazs.push_back(Baz(flatbuffers::make_span(test_array4), 3));
  auto baz_vec = fbb.CreateVectorOfSortedStructs(&bazs);

  auto test_string = fbb.CreateString("TEST");

  float test_float_array1[3] = { 1.5, 2.5, 0 };
  float test_float_array2[3] = { 7.5, 2.5, 0 };
  float test_float_array3[3] = { 1.5, 2.5, -1 };
  float test_float_array4[3] = { -1.5, 2.5, 0 };
  std::vector<Bar> bars;
  bars.push_back(Bar(flatbuffers::make_span(test_float_array1), 3));
  bars.push_back(Bar(flatbuffers::make_span(test_float_array2), 4));
  bars.push_back(Bar(flatbuffers::make_span(test_float_array3), 2));
  bars.push_back(Bar(flatbuffers::make_span(test_float_array4), 1));
  auto bar_vec = fbb.CreateVectorOfSortedStructs(&bars);

  auto t = CreateFooTable(fbb, 1, 2, test_string, baz_vec, bar_vec);

  fbb.Finish(t);

  uint8_t *buf = fbb.GetBufferPointer();
  auto foo_table = GetFooTable(buf);

  auto sorted_baz_vec = foo_table->d();
  TEST_EQ(sorted_baz_vec->Get(0)->b(), 1);
  TEST_EQ(sorted_baz_vec->Get(3)->b(), 4);

  uint8_t test_array[4];
  auto *key_array = &flatbuffers::CastToArray(test_array);
  key_array->CopyFromSpan(flatbuffers::make_span(test_array1));

  TEST_NOTNULL(sorted_baz_vec->LookupByKey(key_array));
  TEST_EQ(sorted_baz_vec->LookupByKey(key_array)->b(), 4);
  uint8_t array_int[4] = { 7, 2, 3, 0 };
  key_array->CopyFromSpan(flatbuffers::make_span(array_int));
  TEST_EQ(sorted_baz_vec->LookupByKey(key_array),
          static_cast<const Baz *>(nullptr));

  auto sorted_bar_vec = foo_table->e();
  TEST_EQ(sorted_bar_vec->Get(0)->b(), 1);
  TEST_EQ(sorted_bar_vec->Get(3)->b(), 4);

  float test_float_array[3];
  auto *key_float_array = &flatbuffers::CastToArray(test_float_array);
  key_float_array->CopyFromSpan(flatbuffers::make_span(test_float_array1));
  TEST_NOTNULL(sorted_bar_vec->LookupByKey(key_float_array));
  TEST_EQ(sorted_bar_vec->LookupByKey(key_float_array)->b(), 3);
  float array_float[3] = { -1, -2, -3 };
  key_float_array->CopyFromSpan(flatbuffers::make_span(array_float));
  TEST_EQ(sorted_bar_vec->LookupByKey(key_float_array),
          static_cast<const Bar *>(nullptr));
}

void StructKeyInStructTest() {
  flatbuffers::FlatBufferBuilder fbb;
  std::vector<Apple> apples;
  float test_float_array1[3] = { 1.5, 2.5, 0 };
  float test_float_array2[3] = { 7.5, 2.5, 0 };
  float test_float_array3[3] = { 1.5, 2.5, -1 };
  apples.push_back(
      Apple(2, Color(flatbuffers::make_span(test_float_array1), 3)));
  apples.push_back(
      Apple(3, Color(flatbuffers::make_span(test_float_array2), 3)));
  apples.push_back(
      Apple(1, Color(flatbuffers::make_span(test_float_array3), 1)));

  auto apples_vec = fbb.CreateVectorOfSortedStructs(&apples);
  auto test_string = fbb.CreateString("TEST");

  FooTableBuilder foo_builder(fbb);
  foo_builder.add_a(1);
  foo_builder.add_c(test_string);

  foo_builder.add_f(apples_vec);

  auto orc = foo_builder.Finish();
  fbb.Finish(orc);

  uint8_t *buf = fbb.GetBufferPointer();
  auto foo_table = GetFooTable(buf);

  auto sorted_apple_vec = foo_table->f();
  TEST_EQ(sorted_apple_vec->Get(0)->tag(), 1);
  TEST_EQ(sorted_apple_vec->Get(1)->tag(), 2);
  TEST_EQ(sorted_apple_vec->Get(2)->tag(), 3);
  TEST_EQ(sorted_apple_vec
              ->LookupByKey(Color(flatbuffers::make_span(test_float_array1), 3))
              ->tag(),
          2);
  TEST_EQ(sorted_apple_vec->LookupByKey(
              Color(flatbuffers::make_span(test_float_array1), 0)),
          static_cast<const Apple *>(nullptr));
}

void NestedStructKeyInStructTest() {
  flatbuffers::FlatBufferBuilder fbb;
  std::vector<Fruit> fruits;
  float test_float_array1[3] = { 1.5, 2.5, 0 };
  float test_float_array2[3] = { 1.5, 2.5, 0 };
  float test_float_array3[3] = { 1.5, 2.5, -1 };

  fruits.push_back(
      Fruit(Apple(2, Color(flatbuffers::make_span(test_float_array1), 2)), 2));
  fruits.push_back(
      Fruit(Apple(2, Color(flatbuffers::make_span(test_float_array2), 1)), 1));
  fruits.push_back(
      Fruit(Apple(2, Color(flatbuffers::make_span(test_float_array3), 3)), 3));

  auto test_string = fbb.CreateString("TEST");
  auto fruits_vec = fbb.CreateVectorOfSortedStructs(&fruits);

  FooTableBuilder foo_builder(fbb);
  foo_builder.add_a(1);
  foo_builder.add_c(test_string);
  foo_builder.add_g(fruits_vec);

  auto orc = foo_builder.Finish();
  fbb.Finish(orc);
  uint8_t *buf = fbb.GetBufferPointer();
  auto foo_table = GetFooTable(buf);

  auto sorted_fruit_vec = foo_table->g();
  TEST_EQ(sorted_fruit_vec->Get(0)->b(), 3);
  TEST_EQ(sorted_fruit_vec->Get(1)->b(), 1);
  TEST_EQ(sorted_fruit_vec->Get(2)->b(), 2);
  TEST_EQ(sorted_fruit_vec
              ->LookupByKey(
                  Apple(2, Color(flatbuffers::make_span(test_float_array2), 1)))
              ->b(),
          1);
  TEST_EQ(sorted_fruit_vec->LookupByKey(
              Apple(1, Color(flatbuffers::make_span(test_float_array2), 1))),
          static_cast<const Fruit *>(nullptr));
}

void FixedSizedStructArrayKeyInStructTest() {
  flatbuffers::FlatBufferBuilder fbb;
  std::vector<Grain> grains;
  uint8_t test_char_array1[3] = { 'u', 's', 'a' };
  uint8_t test_char_array2[3] = { 'c', 'h', 'n' };
  uint8_t test_char_array3[3] = { 'c', 'h', 'l' };
  uint8_t test_char_array4[3] = { 'f', 'r', 'a' };
  uint8_t test_char_array5[3] = { 'i', 'n', 'd' };
  uint8_t test_char_array6[3] = { 'i', 't', 'a' };

  Rice test_rice_array1[3] = {
    Rice(flatbuffers::make_span(test_char_array1), 2),
    Rice(flatbuffers::make_span(test_char_array2), 1),
    Rice(flatbuffers::make_span(test_char_array3), 2)
  };
  Rice test_rice_array2[3] = {
    Rice(flatbuffers::make_span(test_char_array4), 2),
    Rice(flatbuffers::make_span(test_char_array5), 1),
    Rice(flatbuffers::make_span(test_char_array6), 2)
  };
  Rice test_rice_array3[3] = {
    Rice(flatbuffers::make_span(test_char_array4), 2),
    Rice(flatbuffers::make_span(test_char_array6), 1),
    Rice(flatbuffers::make_span(test_char_array1), 2)
  };

  grains.push_back(Grain(flatbuffers::make_span(test_rice_array1), 3));
  grains.push_back(Grain(flatbuffers::make_span(test_rice_array2), 1));
  grains.push_back(Grain(flatbuffers::make_span(test_rice_array3), 2));

  auto test_string = fbb.CreateString("TEST");
  auto grains_vec = fbb.CreateVectorOfSortedStructs(&grains);
  FooTableBuilder foo_builder(fbb);
  foo_builder.add_a(1);
  foo_builder.add_c(test_string);
  foo_builder.add_h(grains_vec);

  auto orc = foo_builder.Finish();
  fbb.Finish(orc);
  uint8_t *buf = fbb.GetBufferPointer();
  auto foo_table = GetFooTable(buf);

  auto sorted_grain_vec = foo_table->h();
  TEST_EQ(sorted_grain_vec->Get(0)->tag(), 1);
  TEST_EQ(sorted_grain_vec->Get(1)->tag(), 2);
  TEST_EQ(sorted_grain_vec->Get(2)->tag(), 3);
  TEST_EQ(
      sorted_grain_vec->LookupByKey(&flatbuffers::CastToArray(test_rice_array1))
          ->tag(),
      3);
  Rice test_rice_array[3] = { Rice(flatbuffers::make_span(test_char_array3), 2),
                              Rice(flatbuffers::make_span(test_char_array2), 1),
                              Rice(flatbuffers::make_span(test_char_array1),
                                   2) };
  TEST_EQ(
      sorted_grain_vec->LookupByKey(&flatbuffers::CastToArray(test_rice_array)),
      static_cast<const Grain *>(nullptr));
  TEST_EQ(
      sorted_grain_vec->LookupByKey(&flatbuffers::CastToArray(test_rice_array1))
          ->tag(),
      3);
}

}  // namespace tests
}  // namespace flatbuffers
