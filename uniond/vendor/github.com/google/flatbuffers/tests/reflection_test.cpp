#include "reflection_test.h"

#include "tests/arrays_test_generated.h"
#include "flatbuffers/minireflect.h"
#include "flatbuffers/reflection.h"
#include "flatbuffers/reflection_generated.h"
#include "flatbuffers/verifier.h"
#include "monster_test.h"
#include "monster_test_generated.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

using namespace MyGame::Example;

void ReflectionTest(const std::string &tests_data_path, uint8_t *flatbuf,
                    size_t length) {
  // Load a binary schema.
  std::string bfbsfile;
  TEST_EQ(flatbuffers::LoadFile((tests_data_path + "monster_test.bfbs").c_str(),
                                true, &bfbsfile),
          true);

  // Verify it, just in case:
  flatbuffers::Verifier verifier(
      reinterpret_cast<const uint8_t *>(bfbsfile.c_str()), bfbsfile.length());
  TEST_EQ(reflection::VerifySchemaBuffer(verifier), true);

  // Make sure the schema is what we expect it to be.
  auto &schema = *reflection::GetSchema(bfbsfile.c_str());
  auto root_table = schema.root_table();

  // Check the declaration files.
  TEST_EQ_STR(root_table->name()->c_str(), "MyGame.Example.Monster");
  TEST_EQ_STR(root_table->declaration_file()->c_str(), "//monster_test.fbs");
  TEST_EQ_STR(
      schema.objects()->LookupByKey("TableA")->declaration_file()->c_str(),
      "//include_test/include_test1.fbs");
  TEST_EQ_STR(schema.objects()
                  ->LookupByKey("MyGame.OtherNameSpace.Unused")
                  ->declaration_file()
                  ->c_str(),
              "//include_test/sub/include_test2.fbs");
  TEST_EQ_STR(schema.enums()
                  ->LookupByKey("MyGame.OtherNameSpace.FromInclude")
                  ->declaration_file()
                  ->c_str(),
              "//include_test/sub/include_test2.fbs");

  // Check scheam filenames and their includes.
  TEST_EQ(schema.fbs_files()->size(), 3);

  const auto fbs0 = schema.fbs_files()->Get(0);
  TEST_EQ_STR(fbs0->filename()->c_str(), "//include_test/include_test1.fbs");
  const auto fbs0_includes = fbs0->included_filenames();
  TEST_EQ(fbs0_includes->size(), 2);

  // TODO(caspern): Should we force or disallow inclusion of self?
  TEST_EQ_STR(fbs0_includes->Get(0)->c_str(),
              "//include_test/include_test1.fbs");
  TEST_EQ_STR(fbs0_includes->Get(1)->c_str(),
              "//include_test/sub/include_test2.fbs");

  const auto fbs1 = schema.fbs_files()->Get(1);
  TEST_EQ_STR(fbs1->filename()->c_str(),
              "//include_test/sub/include_test2.fbs");
  const auto fbs1_includes = fbs1->included_filenames();
  TEST_EQ(fbs1_includes->size(), 2);
  TEST_EQ_STR(fbs1_includes->Get(0)->c_str(),
              "//include_test/include_test1.fbs");
  TEST_EQ_STR(fbs1_includes->Get(1)->c_str(),
              "//include_test/sub/include_test2.fbs");

  const auto fbs2 = schema.fbs_files()->Get(2);
  TEST_EQ_STR(fbs2->filename()->c_str(), "//monster_test.fbs");
  const auto fbs2_includes = fbs2->included_filenames();
  TEST_EQ(fbs2_includes->size(), 1);
  TEST_EQ_STR(fbs2_includes->Get(0)->c_str(),
              "//include_test/include_test1.fbs");

  // Check Root table fields
  auto fields = root_table->fields();
  auto hp_field_ptr = fields->LookupByKey("hp");
  TEST_NOTNULL(hp_field_ptr);
  auto &hp_field = *hp_field_ptr;
  TEST_EQ_STR(hp_field.name()->c_str(), "hp");
  TEST_EQ(hp_field.id(), 2);
  TEST_EQ(hp_field.type()->base_type(), reflection::Short);

  auto friendly_field_ptr = fields->LookupByKey("friendly");
  TEST_NOTNULL(friendly_field_ptr);
  TEST_NOTNULL(friendly_field_ptr->attributes());
  TEST_NOTNULL(friendly_field_ptr->attributes()->LookupByKey("priority"));

  // Make sure the table index is what we expect it to be.
  auto pos_field_ptr = fields->LookupByKey("pos");
  TEST_NOTNULL(pos_field_ptr);
  TEST_EQ(pos_field_ptr->type()->base_type(), reflection::Obj);
  auto pos_table_ptr = schema.objects()->Get(pos_field_ptr->type()->index());
  TEST_NOTNULL(pos_table_ptr);
  TEST_EQ_STR(pos_table_ptr->name()->c_str(), "MyGame.Example.Vec3");

  // Test nullability of fields: hp is a 0-default scalar, pos is a struct =>
  // optional, and name is a required string => not optional.
  TEST_EQ(hp_field.optional(), false);
  TEST_EQ(pos_field_ptr->optional(), true);
  TEST_EQ(fields->LookupByKey("name")->optional(), false);

  // Now use it to dynamically access a buffer.
  auto &root = *flatbuffers::GetAnyRoot(flatbuf);

  // Verify the buffer first using reflection based verification
  TEST_EQ(flatbuffers::Verify(schema, *schema.root_table(), flatbuf, length),
          true);

  auto hp = flatbuffers::GetFieldI<uint16_t>(root, hp_field);
  TEST_EQ(hp, 80);

  // Rather than needing to know the type, we can also get the value of
  // any field as an int64_t/double/string, regardless of what it actually is.
  auto hp_int64 = flatbuffers::GetAnyFieldI(root, hp_field);
  TEST_EQ(hp_int64, 80);
  auto hp_double = flatbuffers::GetAnyFieldF(root, hp_field);
  TEST_EQ(hp_double, 80.0);
  auto hp_string = flatbuffers::GetAnyFieldS(root, hp_field, &schema);
  TEST_EQ_STR(hp_string.c_str(), "80");

  // Get struct field through reflection
  auto pos_struct = flatbuffers::GetFieldStruct(root, *pos_field_ptr);
  TEST_NOTNULL(pos_struct);
  TEST_EQ(flatbuffers::GetAnyFieldF(*pos_struct,
                                    *pos_table_ptr->fields()->LookupByKey("z")),
          3.0f);

  auto test3_field = pos_table_ptr->fields()->LookupByKey("test3");
  auto test3_struct = flatbuffers::GetFieldStruct(*pos_struct, *test3_field);
  TEST_NOTNULL(test3_struct);
  auto test3_object = schema.objects()->Get(test3_field->type()->index());

  TEST_EQ(flatbuffers::GetAnyFieldF(*test3_struct,
                                    *test3_object->fields()->LookupByKey("a")),
          10);

  // We can also modify it.
  flatbuffers::SetField<uint16_t>(&root, hp_field, 200);
  hp = flatbuffers::GetFieldI<uint16_t>(root, hp_field);
  TEST_EQ(hp, 200);

  // We can also set fields generically:
  flatbuffers::SetAnyFieldI(&root, hp_field, 300);
  hp_int64 = flatbuffers::GetAnyFieldI(root, hp_field);
  TEST_EQ(hp_int64, 300);
  flatbuffers::SetAnyFieldF(&root, hp_field, 300.5);
  hp_int64 = flatbuffers::GetAnyFieldI(root, hp_field);
  TEST_EQ(hp_int64, 300);
  flatbuffers::SetAnyFieldS(&root, hp_field, "300");
  hp_int64 = flatbuffers::GetAnyFieldI(root, hp_field);
  TEST_EQ(hp_int64, 300);

  // Test buffer is valid after the modifications
  TEST_EQ(flatbuffers::Verify(schema, *schema.root_table(), flatbuf, length),
          true);

  // Reset it, for further tests.
  flatbuffers::SetField<uint16_t>(&root, hp_field, 80);

  // More advanced functionality: changing the size of items in-line!
  // First we put the FlatBuffer inside an std::vector.
  std::vector<uint8_t> resizingbuf(flatbuf, flatbuf + length);
  // Find the field we want to modify.
  auto &name_field = *fields->LookupByKey("name");
  // Get the root.
  // This time we wrap the result from GetAnyRoot in a smartpointer that
  // will keep rroot valid as resizingbuf resizes.
  auto rroot = flatbuffers::piv(flatbuffers::GetAnyRoot(resizingbuf.data()),
                                resizingbuf);
  SetString(schema, "totally new string", GetFieldS(**rroot, name_field),
            &resizingbuf);
  // Here resizingbuf has changed, but rroot is still valid.
  TEST_EQ_STR(GetFieldS(**rroot, name_field)->c_str(), "totally new string");
  // Now lets extend a vector by 100 elements (10 -> 110).
  auto &inventory_field = *fields->LookupByKey("inventory");
  auto rinventory = flatbuffers::piv(
      flatbuffers::GetFieldV<uint8_t>(**rroot, inventory_field), resizingbuf);
  flatbuffers::ResizeVector<uint8_t>(schema, 110, 50, *rinventory,
                                     &resizingbuf);
  // rinventory still valid, so lets read from it.
  TEST_EQ(rinventory->Get(10), 50);

  // For reflection uses not covered already, there is a more powerful way:
  // we can simply generate whatever object we want to add/modify in a
  // FlatBuffer of its own, then add that to an existing FlatBuffer:
  // As an example, let's add a string to an array of strings.
  // First, find our field:
  auto &testarrayofstring_field = *fields->LookupByKey("testarrayofstring");
  // Find the vector value:
  auto rtestarrayofstring = flatbuffers::piv(
      flatbuffers::GetFieldV<flatbuffers::Offset<flatbuffers::String>>(
          **rroot, testarrayofstring_field),
      resizingbuf);
  // It's a vector of 2 strings, to which we add one more, initialized to
  // offset 0.
  flatbuffers::ResizeVector<flatbuffers::Offset<flatbuffers::String>>(
      schema, 3, 0, *rtestarrayofstring, &resizingbuf);
  // Here we just create a buffer that contans a single string, but this
  // could also be any complex set of tables and other values.
  flatbuffers::FlatBufferBuilder stringfbb;
  stringfbb.Finish(stringfbb.CreateString("hank"));
  // Add the contents of it to our existing FlatBuffer.
  // We do this last, so the pointer doesn't get invalidated (since it is
  // at the end of the buffer):
  auto string_ptr = flatbuffers::AddFlatBuffer(
      resizingbuf, stringfbb.GetBufferPointer(), stringfbb.GetSize());
  // Finally, set the new value in the vector.
  rtestarrayofstring->MutateOffset(2, string_ptr);
  TEST_EQ_STR(rtestarrayofstring->Get(0)->c_str(), "bob");
  TEST_EQ_STR(rtestarrayofstring->Get(2)->c_str(), "hank");
  // Test integrity of all resize operations above.
  flatbuffers::Verifier resize_verifier(
      reinterpret_cast<const uint8_t *>(resizingbuf.data()),
      resizingbuf.size());
  TEST_EQ(VerifyMonsterBuffer(resize_verifier), true);

  // Test buffer is valid using reflection as well
  TEST_EQ(flatbuffers::Verify(schema, *schema.root_table(), resizingbuf.data(),
                              resizingbuf.size()),
          true);

  // As an additional test, also set it on the name field.
  // Note: unlike the name change above, this just overwrites the offset,
  // rather than changing the string in-place.
  SetFieldT(*rroot, name_field, string_ptr);
  TEST_EQ_STR(GetFieldS(**rroot, name_field)->c_str(), "hank");

  // Using reflection, rather than mutating binary FlatBuffers, we can also copy
  // tables and other things out of other FlatBuffers into a FlatBufferBuilder,
  // either part or whole.
  flatbuffers::FlatBufferBuilder fbb;
  auto root_offset = flatbuffers::CopyTable(
      fbb, schema, *root_table, *flatbuffers::GetAnyRoot(flatbuf), true);
  fbb.Finish(root_offset, MonsterIdentifier());
  // Test that it was copied correctly:
  AccessFlatBufferTest(fbb.GetBufferPointer(), fbb.GetSize());

  // Test buffer is valid using reflection as well
  TEST_EQ(flatbuffers::Verify(schema, *schema.root_table(),
                              fbb.GetBufferPointer(), fbb.GetSize()),
          true);
}

void MiniReflectFlatBuffersTest(uint8_t *flatbuf) {
  auto s =
      flatbuffers::FlatBufferToString(flatbuf, Monster::MiniReflectTypeTable());
  TEST_EQ_STR(
      s.c_str(),
      "{ "
      "pos: { x: 1.0, y: 2.0, z: 3.0, test1: 0.0, test2: Red, test3: "
      "{ a: 10, b: 20 } }, "
      "hp: 80, "
      "name: \"MyMonster\", "
      "inventory: [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 ], "
      "test_type: Monster, "
      "test: { name: \"Fred\" }, "
      "test4: [ { a: 10, b: 20 }, { a: 30, b: 40 } ], "
      "testarrayofstring: [ \"bob\", \"fred\", \"bob\", \"fred\" ], "
      "testarrayoftables: [ { hp: 1000, name: \"Barney\" }, { name: \"Fred\" "
      "}, "
      "{ name: \"Wilma\" } ], "
      // TODO(wvo): should really print this nested buffer correctly.
      "testnestedflatbuffer: [ 124, 0, 0, 0, 77, 79, 78, 83, 0, 0, 114, 0, 16, "
      "0, 0, 0, 4, 0, 6, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "
      "0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "
      "0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "
      "0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "
      "0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 114, 0, 0, 0, 0, 0, 0, 0, "
      "8, 0, 0, 0, 0, 0, 192, 127, 13, 0, 0, 0, 78, 101, 115, 116, 101, 100, "
      "77, 111, 110, 115, 116, 101, 114, 0, 0, 0 ], "
      "testarrayofstring2: [ \"jane\", \"mary\" ], "
      "testarrayofsortedstruct: [ { id: 0, distance: 0 }, "
      "{ id: 2, distance: 20 }, { id: 3, distance: 30 }, "
      "{ id: 4, distance: 40 } ], "
      "flex: [ 210, 4, 5, 2 ], "
      "test5: [ { a: 10, b: 20 }, { a: 30, b: 40 } ], "
      "vector_of_enums: [ Blue, Green ], "
      "scalar_key_sorted_tables: [ { id: \"miss\" } ], "
      "nan_default: nan "
      "}");

  Test test(16, 32);
  Vec3 vec(1, 2, 3, 1.5, Color_Red, test);
  flatbuffers::FlatBufferBuilder vec_builder;
  vec_builder.Finish(vec_builder.CreateStruct(vec));
  auto vec_buffer = vec_builder.Release();
  auto vec_str = flatbuffers::FlatBufferToString(vec_buffer.data(),
                                                 Vec3::MiniReflectTypeTable());
  TEST_EQ_STR(vec_str.c_str(),
              "{ x: 1.0, y: 2.0, z: 3.0, test1: 1.5, test2: Red, test3: { a: "
              "16, b: 32 } }");
}

void MiniReflectFixedLengthArrayTest() {
  // VS10 does not support typed enums, exclude from tests
#if !defined(_MSC_VER) || _MSC_VER >= 1700
  flatbuffers::FlatBufferBuilder fbb;
  MyGame::Example::ArrayStruct aStruct(2, 12, 1);
  auto aTable = MyGame::Example::CreateArrayTable(fbb, &aStruct);
  fbb.Finish(aTable);

  auto flatbuf = fbb.Release();
  auto s = flatbuffers::FlatBufferToString(
      flatbuf.data(), MyGame::Example::ArrayTableTypeTable());
  TEST_EQ_STR(
      "{ "
      "a: { a: 2.0, "
      "b: [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ], "
      "c: 12, "
      "d: [ { a: [ 0, 0 ], b: A, c: [ A, A ], d: [ 0, 0 ] }, "
      "{ a: [ 0, 0 ], b: A, c: [ A, A ], d: [ 0, 0 ] } ], "
      "e: 1, f: [ 0, 0 ] } "
      "}",
      s.c_str());
#endif
}

}  // namespace tests
}  // namespace flatbuffers
