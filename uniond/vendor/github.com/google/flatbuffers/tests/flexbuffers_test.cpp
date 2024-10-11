#include "flexbuffers_test.h"

#include <limits>

#include "flatbuffers/flexbuffers.h"
#include "flatbuffers/idl.h"
#include "is_quiet_nan.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

// Shortcuts for the infinity.
static const auto infinity_d = std::numeric_limits<double>::infinity();

void FlexBuffersTest() {
  flexbuffers::Builder slb(512,
                           flexbuffers::BUILDER_FLAG_SHARE_KEYS_AND_STRINGS);

  // Write the equivalent of:
  // { vec: [ -100, "Fred", 4.0, false ], bar: [ 1, 2, 3 ], bar3: [ 1, 2, 3 ],
  // foo: 100, bool: true, mymap: { foo: "Fred" } }

  // It's possible to do this without std::function support as well.
  slb.Map([&]() {
    slb.Vector("vec", [&]() {
      slb += -100;  // Equivalent to slb.Add(-100) or slb.Int(-100);
      slb += "Fred";
      slb.IndirectFloat(4.0f);
      auto i_f = slb.LastValue();
      uint8_t blob[] = { 77 };
      slb.Blob(blob, 1);
      slb += false;
      slb.ReuseValue(i_f);
    });
    int ints[] = { 1, 2, 3 };
    slb.Vector("bar", ints, 3);
    slb.FixedTypedVector("bar3", ints, 3);
    bool bools[] = { true, false, true, false };
    slb.Vector("bools", bools, 4);
    slb.Bool("bool", true);
    slb.Double("foo", 100);
    slb.Map("mymap", [&]() {
      slb.String("foo", "Fred");  // Testing key and string reuse.
    });
  });
  slb.Finish();

  // clang-format off
  #ifdef FLATBUFFERS_TEST_VERBOSE
    for (size_t i = 0; i < slb.GetBuffer().size(); i++)
      printf("%d ", slb.GetBuffer().data()[i]);
    printf("\n");
  #endif
  // clang-format on

  std::vector<uint8_t> reuse_tracker;
  TEST_EQ(flexbuffers::VerifyBuffer(slb.GetBuffer().data(),
                                    slb.GetBuffer().size(), &reuse_tracker),
          true);

  auto map = flexbuffers::GetRoot(slb.GetBuffer()).AsMap();
  TEST_EQ(map.size(), 7);
  auto vec = map["vec"].AsVector();
  TEST_EQ(vec.size(), 6);
  TEST_EQ(vec[0].AsInt64(), -100);
  TEST_EQ_STR(vec[1].AsString().c_str(), "Fred");
  TEST_EQ(vec[1].AsInt64(), 0);  // Number parsing failed.
  TEST_EQ(vec[2].AsDouble(), 4.0);
  TEST_EQ(vec[2].AsString().IsTheEmptyString(), true);  // Wrong Type.
  TEST_EQ_STR(vec[2].AsString().c_str(), "");     // This still works though.
  TEST_EQ_STR(vec[2].ToString().c_str(), "4.0");  // Or have it converted.
  // Few tests for templated version of As.
  TEST_EQ(vec[0].As<int64_t>(), -100);
  TEST_EQ_STR(vec[1].As<std::string>().c_str(), "Fred");
  TEST_EQ(vec[1].As<int64_t>(), 0);  // Number parsing failed.
  TEST_EQ(vec[2].As<double>(), 4.0);
  // Test that the blob can be accessed.
  TEST_EQ(vec[3].IsBlob(), true);
  auto blob = vec[3].AsBlob();
  TEST_EQ(blob.size(), 1);
  TEST_EQ(blob.data()[0], 77);
  TEST_EQ(vec[4].IsBool(), true);   // Check if type is a bool
  TEST_EQ(vec[4].AsBool(), false);  // Check if value is false
  TEST_EQ(vec[5].AsDouble(), 4.0);  // This is shared with vec[2] !
  auto tvec = map["bar"].AsTypedVector();
  TEST_EQ(tvec.size(), 3);
  TEST_EQ(tvec[2].AsInt8(), 3);
  auto tvec3 = map["bar3"].AsFixedTypedVector();
  TEST_EQ(tvec3.size(), 3);
  TEST_EQ(tvec3[2].AsInt8(), 3);
  TEST_EQ(map["bool"].AsBool(), true);
  auto tvecb = map["bools"].AsTypedVector();
  TEST_EQ(tvecb.ElementType(), flexbuffers::FBT_BOOL);
  TEST_EQ(map["foo"].AsUInt8(), 100);
  TEST_EQ(map["unknown"].IsNull(), true);
  auto mymap = map["mymap"].AsMap();
  // These should be equal by pointer equality, since key and value are shared.
  TEST_EQ(mymap.Keys()[0].AsKey(), map.Keys()[4].AsKey());
  TEST_EQ(mymap.Values()[0].AsString().c_str(), vec[1].AsString().c_str());
  // We can mutate values in the buffer.
  TEST_EQ(vec[0].MutateInt(-99), true);
  TEST_EQ(vec[0].AsInt64(), -99);
  TEST_EQ(vec[1].MutateString("John"), true);  // Size must match.
  TEST_EQ_STR(vec[1].AsString().c_str(), "John");
  TEST_EQ(vec[1].MutateString("Alfred"), false);  // Too long.
  TEST_EQ(vec[2].MutateFloat(2.0f), true);
  TEST_EQ(vec[2].AsFloat(), 2.0f);
  TEST_EQ(vec[2].MutateFloat(3.14159), false);  // Double does not fit in float.
  TEST_EQ(vec[4].AsBool(), false);              // Is false before change
  TEST_EQ(vec[4].MutateBool(true), true);       // Can change a bool
  TEST_EQ(vec[4].AsBool(), true);               // Changed bool is now true

  // Parse from JSON:
  flatbuffers::Parser parser;
  slb.Clear();
  auto jsontest = "{ a: [ 123, 456.0 ], b: \"hello\", c: true, d: false }";
  TEST_EQ(parser.ParseFlexBuffer(jsontest, nullptr, &slb), true);
  TEST_EQ(flexbuffers::VerifyBuffer(slb.GetBuffer().data(),
                                    slb.GetBuffer().size(), &reuse_tracker),
          true);
  auto jroot = flexbuffers::GetRoot(slb.GetBuffer());
  auto jmap = jroot.AsMap();
  auto jvec = jmap["a"].AsVector();
  TEST_EQ(jvec[0].AsInt64(), 123);
  TEST_EQ(jvec[1].AsDouble(), 456.0);
  TEST_EQ_STR(jmap["b"].AsString().c_str(), "hello");
  TEST_EQ(jmap["c"].IsBool(), true);   // Parsed correctly to a bool
  TEST_EQ(jmap["c"].AsBool(), true);   // Parsed correctly to true
  TEST_EQ(jmap["d"].IsBool(), true);   // Parsed correctly to a bool
  TEST_EQ(jmap["d"].AsBool(), false);  // Parsed correctly to false
  // And from FlexBuffer back to JSON:
  auto jsonback = jroot.ToString();
  TEST_EQ_STR(jsontest, jsonback.c_str());
  // With indentation:
  std::string jsonback_indented;
  jroot.ToString(true, false, jsonback_indented, true, 0, "  ");
  auto jsontest_indented =
    "{\n  a: [\n    123,\n    456.0\n  ],\n  b: \"hello\",\n  c: true,\n  d: false\n}";
  TEST_EQ_STR(jsontest_indented, jsonback_indented.c_str());

  slb.Clear();
  slb.Vector([&]() {
    for (int i = 0; i < 130; ++i) slb.Add(static_cast<uint8_t>(255));
    slb.Vector([&]() {
      for (int i = 0; i < 130; ++i) slb.Add(static_cast<uint8_t>(255));
      slb.Vector([] {});
    });
  });
  slb.Finish();
  TEST_EQ(slb.GetSize(), 664);
}

void FlexBuffersReuseBugTest() {
  flexbuffers::Builder slb;
  slb.Map([&]() {
    slb.Vector("vec", [&]() {});
    slb.Bool("bool", true);
  });
  slb.Finish();
  std::vector<uint8_t> reuse_tracker;
  // This would fail before, since the reuse_tracker would use the address of
  // the vector reference to check for reuse, but in this case we have an empty
  // vector, and since the size field is before the pointer, its address is the
  // same as thing after it, the key "bool".
  // We fix this by using the address of the size field for tracking reuse.
  TEST_EQ(flexbuffers::VerifyBuffer(slb.GetBuffer().data(),
                                    slb.GetBuffer().size(), &reuse_tracker),
          true);
}

void FlexBuffersFloatingPointTest() {
#if defined(FLATBUFFERS_HAS_NEW_STRTOD) && (FLATBUFFERS_HAS_NEW_STRTOD > 0)
  flexbuffers::Builder slb(512,
                           flexbuffers::BUILDER_FLAG_SHARE_KEYS_AND_STRINGS);
  // Parse floating-point values from JSON:
  flatbuffers::Parser parser;
  slb.Clear();
  auto jsontest =
      "{ a: [1.0, nan, inf, infinity, -inf, +inf, -infinity, 8.0] }";
  TEST_EQ(parser.ParseFlexBuffer(jsontest, nullptr, &slb), true);
  auto jroot = flexbuffers::GetRoot(slb.GetBuffer());
  TEST_EQ(flexbuffers::VerifyBuffer(slb.GetBuffer().data(),
                                    slb.GetBuffer().size(), nullptr),
          true);
  auto jmap = jroot.AsMap();
  auto jvec = jmap["a"].AsVector();
  TEST_EQ(8, jvec.size());
  TEST_EQ(1.0, jvec[0].AsDouble());
  TEST_ASSERT(is_quiet_nan(jvec[1].AsDouble()));
  TEST_EQ(infinity_d, jvec[2].AsDouble());
  TEST_EQ(infinity_d, jvec[3].AsDouble());
  TEST_EQ(-infinity_d, jvec[4].AsDouble());
  TEST_EQ(+infinity_d, jvec[5].AsDouble());
  TEST_EQ(-infinity_d, jvec[6].AsDouble());
  TEST_EQ(8.0, jvec[7].AsDouble());
#endif
}

void FlexBuffersDeprecatedTest() {
  // FlexBuffers as originally designed had a flaw involving the
  // FBT_VECTOR_STRING datatype, and this test documents/tests the fix for it.
  // Discussion: https://github.com/google/flatbuffers/issues/5627
  flexbuffers::Builder slb;
  // FBT_VECTOR_* are "typed vectors" where all elements are of the same type.
  // Problem is, when storing FBT_STRING elements, it relies on that type to
  // get the bit-width for the size field of the string, which in this case
  // isn't present, and instead defaults to 8-bit. This means that any strings
  // stored inside such a vector, when accessed thru the old API that returns
  // a String reference, will appear to be truncated if the string stored is
  // actually >=256 bytes.
  std::string test_data(300, 'A');
  auto start = slb.StartVector();
  // This one will have a 16-bit size field.
  slb.String(test_data);
  // This one will have an 8-bit size field.
  slb.String("hello");
  // We're asking this to be serialized as a typed vector (true), but not
  // fixed size (false). The type will be FBT_VECTOR_STRING with a bit-width
  // of whatever the offsets in the vector need, the bit-widths of the strings
  // are not stored(!) <- the actual design flaw.
  // Note that even in the fixed code, we continue to serialize the elements of
  // FBT_VECTOR_STRING as FBT_STRING, since there may be old code out there
  // reading new data that we want to continue to function.
  // Thus, FBT_VECTOR_STRING, while deprecated, will always be represented the
  // same way, the fix lies on the reading side.
  slb.EndVector(start, true, false);
  slb.Finish();
  // Verify because why not.
  TEST_EQ(flexbuffers::VerifyBuffer(slb.GetBuffer().data(),
                                    slb.GetBuffer().size(), nullptr),
          true);
  // So now lets read this data back.
  // For existing data, since we have no way of knowing what the actual
  // bit-width of the size field of the string is, we are going to ignore this
  // field, and instead treat these strings as FBT_KEY (null-terminated), so we
  // can deal with strings of arbitrary length. This of course truncates strings
  // with embedded nulls, but we think that that is preferrable over truncating
  // strings >= 256 bytes.
  auto vec = flexbuffers::GetRoot(slb.GetBuffer()).AsTypedVector();
  // Even though this was serialized as FBT_VECTOR_STRING, it is read as
  // FBT_VECTOR_KEY:
  TEST_EQ(vec.ElementType(), flexbuffers::FBT_KEY);
  // Access the long string. Previously, this would return a string of size 1,
  // since it would read the high-byte of the 16-bit length.
  // This should now correctly test the full 300 bytes, using AsKey():
  TEST_EQ_STR(vec[0].AsKey(), test_data.c_str());
  // Old code that called AsString will continue to work, as the String
  // accessor objects now use a cached size that can come from a key as well.
  TEST_EQ_STR(vec[0].AsString().c_str(), test_data.c_str());
  // Short strings work as before:
  TEST_EQ_STR(vec[1].AsKey(), "hello");
  TEST_EQ_STR(vec[1].AsString().c_str(), "hello");
  // So, while existing code and data mostly "just work" with the fixes applied
  // to AsTypedVector and AsString, what do you do going forward?
  // Code accessing existing data doesn't necessarily need to change, though
  // you could consider using AsKey instead of AsString for a) documenting
  // that you are accessing keys, or b) a speedup if you don't actually use
  // the string size.
  // For new data, or data that doesn't need to be backwards compatible,
  // instead serialize as FBT_VECTOR (call EndVector with typed = false, then
  // read elements with AsString), or, for maximum compactness, use
  // FBT_VECTOR_KEY (call slb.Key above instead, read with AsKey or AsString).
}

void ParseFlexbuffersFromJsonWithNullTest() {
  // Test nulls are handled appropriately through flexbuffers to exercise other
  // code paths of ParseSingleValue in the optional scalars change.
  // TODO(cneo): Json -> Flatbuffers test once some language can generate code
  // with optional scalars.
  {
    char json[] = "{\"opt_field\": 123 }";
    flatbuffers::Parser parser;
    flexbuffers::Builder flexbuild;
    parser.ParseFlexBuffer(json, nullptr, &flexbuild);
    auto root = flexbuffers::GetRoot(flexbuild.GetBuffer());
    TEST_EQ(root.AsMap()["opt_field"].AsInt64(), 123);
  }
  {
    char json[] = "{\"opt_field\": 123.4 }";
    flatbuffers::Parser parser;
    flexbuffers::Builder flexbuild;
    parser.ParseFlexBuffer(json, nullptr, &flexbuild);
    auto root = flexbuffers::GetRoot(flexbuild.GetBuffer());
    TEST_EQ(root.AsMap()["opt_field"].AsDouble(), 123.4);
  }
  {
    char json[] = "{\"opt_field\": null }";
    flatbuffers::Parser parser;
    flexbuffers::Builder flexbuild;
    parser.ParseFlexBuffer(json, nullptr, &flexbuild);
    auto root = flexbuffers::GetRoot(flexbuild.GetBuffer());
    TEST_ASSERT(!root.AsMap().IsTheEmptyMap());
    TEST_ASSERT(root.AsMap()["opt_field"].IsNull());
    TEST_EQ(root.ToString(), std::string("{ opt_field: null }"));
  }
}

}  // namespace tests
}  // namespace flatbuffers
