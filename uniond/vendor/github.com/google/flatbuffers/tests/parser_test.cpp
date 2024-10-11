#include "parser_test.h"

#include <cmath>
#include <limits>
#include <string>

#include "flatbuffers/idl.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {
namespace {

// Shortcuts for the infinity.
static const auto infinity_f = std::numeric_limits<float>::infinity();
static const auto infinity_d = std::numeric_limits<double>::infinity();

// Test that parser errors are actually generated.
static void TestError_(const char *src, const char *error_substr,
                       bool strict_json, const char *file, int line,
                       const char *func) {
  flatbuffers::IDLOptions opts;
  opts.strict_json = strict_json;
  flatbuffers::Parser parser(opts);
  if (parser.Parse(src)) {
    TestFail("true", "false",
             ("parser.Parse(\"" + std::string(src) + "\")").c_str(), file, line,
             func);
  } else if (!strstr(parser.error_.c_str(), error_substr)) {
    TestFail(error_substr, parser.error_.c_str(),
             ("parser.Parse(\"" + std::string(src) + "\")").c_str(), file, line,
             func);
  }
}

static void TestError_(const char *src, const char *error_substr,
                       const char *file, int line, const char *func) {
  TestError_(src, error_substr, false, file, line, func);
}

#ifdef _WIN32
#  define TestError(src, ...) \
    TestError_(src, __VA_ARGS__, __FILE__, __LINE__, __FUNCTION__)
#else
#  define TestError(src, ...) \
    TestError_(src, __VA_ARGS__, __FILE__, __LINE__, __PRETTY_FUNCTION__)
#endif

static bool FloatCompare(float a, float b) { return fabs(a - b) < 0.001; }

}  // namespace

// Test that parsing errors occur as we'd expect.
// Also useful for coverage, making sure these paths are run.
void ErrorTest() {
  // In order they appear in idl_parser.cpp
  TestError("table X { Y:byte; } root_type X; { Y: 999 }", "does not fit");
  TestError("\"\0", "illegal");
  TestError("\"\\q", "escape code");
  TestError("table ///", "documentation");
  TestError("@", "illegal");
  TestError("table 1", "expecting");
  TestError("table X { Y:[[int]]; }", "nested vector");
  TestError("table X { Y:1; }", "illegal type");
  TestError("table X { Y:int; Y:int; }", "field already");
  TestError("table Y {} table X { Y:int; }", "same as table");
  TestError("struct X { Y:string; }", "only scalar");
  TestError("struct X { a:uint = 42; }", "default values");
  TestError("enum Y:byte { Z = 1 } table X { y:Y; }", "not part of enum");
  TestError("struct X { Y:int (deprecated); }", "deprecate");
  TestError("union Z { X } table X { Y:Z; } root_type X; { Y: {}, A:1 }",
            "missing type field");
  TestError("union Z { X } table X { Y:Z; } root_type X; { Y_type: 99, Y: {",
            "type id");
  TestError("table X { Y:int; } root_type X; { Z:", "unknown field");
  TestError("table X { Y:int; } root_type X; { Y:", "string constant", true);
  TestError("table X { Y:int; } root_type X; { \"Y\":1, }", "string constant",
            true);
  TestError(
      "struct X { Y:int; Z:int; } table W { V:X; } root_type W; "
      "{ V:{ Y:1 } }",
      "wrong number");
  TestError("enum E:byte { A } table X { Y:E; } root_type X; { Y:U }",
            "unknown enum value");
  TestError("table X { Y:byte; } root_type X; { Y:; }", "starting");
  TestError("enum X:byte { Y } enum X {", "enum already");
  TestError("enum X:float {}", "underlying");
  TestError("enum X:byte { Y, Y }", "value already");
  TestError("enum X:byte { Y=2, Z=2 }", "unique");
  TestError("enum X:byte (force_align: 4) { Y }", "force_align");
  TestError("table X { Y:int; } table X {", "datatype already");
  TestError("table X { } union X { }", "datatype already");
  TestError("union X { } table X { }", "datatype already");
  TestError("namespace A; table X { } namespace A; union X { }",
            "datatype already");
  TestError("namespace A; union X { } namespace A; table X { }",
            "datatype already");
  TestError("struct X (force_align: 7) { Y:int; }", "force_align");
  TestError("struct X {}", "size 0");
  TestError("{}", "no root");
  TestError("table X { Y:byte; } root_type X; { Y:1 } { Y:1 }", "end of file");
  TestError("table X { Y:byte; } root_type X; { Y:1 } table Y{ Z:int }",
            "end of file");
  TestError("root_type X;", "unknown root");
  TestError("struct X { Y:int; } root_type X;", "a table");
  TestError("union X { Y }", "referenced");
  TestError("union Z { X } struct X { Y:int; }", "only tables");
  TestError("table X { Y:[int]; YLength:int; }", "clash");
  TestError("table X { Y:byte; } root_type X; { Y:1, Y:2 }", "more than once");
  // float to integer conversion is forbidden
  TestError("table X { Y:int; } root_type X; { Y:1.0 }", "float");
  TestError("table X { Y:bool; } root_type X; { Y:1.0 }", "float");
  TestError("enum X:bool { Y = true }", "must be integral");
  // Array of non-scalar
  TestError("table X { x:int; } struct Y { y:[X:2]; }",
            "may contain only scalar or struct fields");
  // Non-snake case field names
  TestError("table X { Y: int; } root_type Y: {Y:1.0}", "snake_case");
  // Complex defaults
  TestError("table X { y: string = 1; }", "expecting: string");
  TestError("table X { y: string = []; }", " Cannot assign token");
  TestError("table X { y: [int] = [1]; }", "Expected `]`");
  TestError("table X { y: [int] = [; }", "Expected `]`");
  TestError("table X { y: [int] = \"\"; }", "type mismatch");
  // An identifier can't start from sign (+|-)
  TestError("table X { -Y: int; } root_type Y: {Y:1.0}", "identifier");
  TestError("table X { +Y: int; } root_type Y: {Y:1.0}", "identifier");

  // Offset64
  TestError("table X { a:int (vector64); }", "`vector64` attribute");
  TestError("table X { a:int (offset64); }", "`offset64` attribute");
  TestError("table X { a:string (vector64); }", "`vector64` attribute");
  TestError("table y { a:int; } table X { a:y (offset64); }",
            "`offset64` attribute");
  TestError("struct y { a:int; } table X { a:y (offset64); }",
            "`offset64` attribute");
  TestError("table y { a:int; } table X { a:y (vector64); }",
            "`vector64` attribute");
  TestError("union Y { } table X { ys:Y (offset64); }", "`offset64` attribute");

  TestError("table Y { a:int; } table X { ys:[Y] (offset64); }",
            "only vectors of scalars are allowed to be 64-bit.");
  TestError("table Y { a:int; } table X { ys:[Y] (vector64); }",
            "only vectors of scalars are allowed to be 64-bit.");
  TestError("union Y { } table X { ys:[Y] (vector64); }",
            "only vectors of scalars are allowed to be 64-bit.");

  // TOOD(derekbailey): the following three could be allowed once the code gen
  // supports the output.
  TestError("table X { y:[string] (offset64); }",
            "only vectors of scalars are allowed to be 64-bit.");
  TestError("table X { y:[string] (vector64); }",
            "only vectors of scalars are allowed to be 64-bit.");
  TestError("enum X:byte {Z} table X { y:[X] (offset64); }",
            "only vectors of scalars are allowed to be 64-bit.");
}

void EnumOutOfRangeTest() {
  TestError("enum X:byte { Y = 128 }", "enum value does not fit");
  TestError("enum X:byte { Y = -129 }", "enum value does not fit");
  TestError("enum X:byte { Y = 126, Z0, Z1 }", "enum value does not fit");
  TestError("enum X:ubyte { Y = -1 }", "enum value does not fit");
  TestError("enum X:ubyte { Y = 256 }", "enum value does not fit");
  TestError("enum X:ubyte { Y = 255, Z }", "enum value does not fit");
  TestError("table Y{} union X { Y = -1 }", "enum value does not fit");
  TestError("table Y{} union X { Y = 256 }", "enum value does not fit");
  TestError("table Y{} union X { Y = 255, Z:Y }", "enum value does not fit");
  TestError("enum X:int { Y = -2147483649 }", "enum value does not fit");
  TestError("enum X:int { Y = 2147483648 }", "enum value does not fit");
  TestError("enum X:uint { Y = -1 }", "enum value does not fit");
  TestError("enum X:uint { Y = 4294967297 }", "enum value does not fit");
  TestError("enum X:long { Y = 9223372036854775808 }", "does not fit");
  TestError("enum X:long { Y = 9223372036854775807, Z }",
            "enum value does not fit");
  TestError("enum X:ulong { Y = -1 }", "does not fit");
  TestError("enum X:ubyte (bit_flags) { Y=8 }", "bit flag out");
  TestError("enum X:byte (bit_flags) { Y=7 }", "must be unsigned");  // -128
  // bit_flgs out of range
  TestError("enum X:ubyte (bit_flags) { Y0,Y1,Y2,Y3,Y4,Y5,Y6,Y7,Y8 }",
            "out of range");
}

void IntegerOutOfRangeTest() {
  TestError("table T { F:byte; } root_type T; { F:128 }",
            "constant does not fit");
  TestError("table T { F:byte; } root_type T; { F:-129 }",
            "constant does not fit");
  TestError("table T { F:ubyte; } root_type T; { F:256 }",
            "constant does not fit");
  TestError("table T { F:ubyte; } root_type T; { F:-1 }",
            "constant does not fit");
  TestError("table T { F:short; } root_type T; { F:32768 }",
            "constant does not fit");
  TestError("table T { F:short; } root_type T; { F:-32769 }",
            "constant does not fit");
  TestError("table T { F:ushort; } root_type T; { F:65536 }",
            "constant does not fit");
  TestError("table T { F:ushort; } root_type T; { F:-1 }",
            "constant does not fit");
  TestError("table T { F:int; } root_type T; { F:2147483648 }",
            "constant does not fit");
  TestError("table T { F:int; } root_type T; { F:-2147483649 }",
            "constant does not fit");
  TestError("table T { F:uint; } root_type T; { F:4294967296 }",
            "constant does not fit");
  TestError("table T { F:uint; } root_type T; { F:-1 }",
            "constant does not fit");
  // Check fixed width aliases
  TestError("table X { Y:uint8; } root_type X; { Y: -1 }", "does not fit");
  TestError("table X { Y:uint8; } root_type X; { Y: 256 }", "does not fit");
  TestError("table X { Y:uint16; } root_type X; { Y: -1 }", "does not fit");
  TestError("table X { Y:uint16; } root_type X; { Y: 65536 }", "does not fit");
  TestError("table X { Y:uint32; } root_type X; { Y: -1 }", "");
  TestError("table X { Y:uint32; } root_type X; { Y: 4294967296 }",
            "does not fit");
  TestError("table X { Y:uint64; } root_type X; { Y: -1 }", "");
  TestError("table X { Y:uint64; } root_type X; { Y: -9223372036854775809 }",
            "does not fit");
  TestError("table X { Y:uint64; } root_type X; { Y: 18446744073709551616 }",
            "does not fit");

  TestError("table X { Y:int8; } root_type X; { Y: -129 }", "does not fit");
  TestError("table X { Y:int8; } root_type X; { Y: 128 }", "does not fit");
  TestError("table X { Y:int16; } root_type X; { Y: -32769 }", "does not fit");
  TestError("table X { Y:int16; } root_type X; { Y: 32768 }", "does not fit");
  TestError("table X { Y:int32; } root_type X; { Y: -2147483649 }", "");
  TestError("table X { Y:int32; } root_type X; { Y: 2147483648 }",
            "does not fit");
  TestError("table X { Y:int64; } root_type X; { Y: -9223372036854775809 }",
            "does not fit");
  TestError("table X { Y:int64; } root_type X; { Y: 9223372036854775808 }",
            "does not fit");
  // check out-of-int64 as int8
  TestError("table X { Y:int8; } root_type X; { Y: -9223372036854775809 }",
            "does not fit");
  TestError("table X { Y:int8; } root_type X; { Y: 9223372036854775808 }",
            "does not fit");

  // Check default values
  TestError("table X { Y:int64=-9223372036854775809; } root_type X; {}",
            "does not fit");
  TestError("table X { Y:int64= 9223372036854775808; } root_type X; {}",
            "does not fit");
  TestError("table X { Y:uint64; } root_type X; { Y: -1 }", "");
  TestError("table X { Y:uint64=-9223372036854775809; } root_type X; {}",
            "does not fit");
  TestError("table X { Y:uint64= 18446744073709551616; } root_type X; {}",
            "does not fit");
}

void InvalidFloatTest() {
  auto invalid_msg = "invalid number";
  auto comma_msg = "expecting: ,";
  TestError("table T { F:float; } root_type T; { F:1,0 }", "");
  TestError("table T { F:float; } root_type T; { F:. }", "");
  TestError("table T { F:float; } root_type T; { F:- }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:-. }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+. }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:.e }", "");
  TestError("table T { F:float; } root_type T; { F:-e }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+e }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:-.e }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+.e }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:-e1 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+e1 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.0e+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.0e- }", invalid_msg);
  // exponent pP is mandatory for hex-float
  TestError("table T { F:float; } root_type T; { F:0x0 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:-0x. }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x. }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0Xe }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"0Xe\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"nan(1)\" }", invalid_msg);
  // eE not exponent in hex-float!
  TestError("table T { F:float; } root_type T; { F:0x0.0e+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0e- }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0p }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0p+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0p- }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0pa1 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0e+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0e- }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0e+0 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0e-0 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0ep+ }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:0x0.0ep- }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2.3 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2.e3 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2e.3 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2e0.3 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2e3. }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.2e3.0 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:+-1.0 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.0e+-1 }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"1.0e+-1\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:1.e0e }", comma_msg);
  TestError("table T { F:float; } root_type T; { F:0x1.p0e }", comma_msg);
  TestError("table T { F:float; } root_type T; { F:\" 0x10 \" }", invalid_msg);
  // floats in string
  TestError("table T { F:float; } root_type T; { F:\"1,2.\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"1.2e3.\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"0x1.p0e\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"0x1.0\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\" 0x1.0\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:\"+ 0\" }", invalid_msg);
  // disable escapes for "number-in-string"
  TestError("table T { F:float; } root_type T; { F:\"\\f1.2e3.\" }", "invalid");
  TestError("table T { F:float; } root_type T; { F:\"\\t1.2e3.\" }", "invalid");
  TestError("table T { F:float; } root_type T; { F:\"\\n1.2e3.\" }", "invalid");
  TestError("table T { F:float; } root_type T; { F:\"\\r1.2e3.\" }", "invalid");
  TestError("table T { F:float; } root_type T; { F:\"4\\x005\" }", "invalid");
  TestError("table T { F:float; } root_type T; { F:\"\'12\'\" }", invalid_msg);
  // null is not a number constant!
  TestError("table T { F:float; } root_type T; { F:\"null\" }", invalid_msg);
  TestError("table T { F:float; } root_type T; { F:null }", invalid_msg);
}

void UnicodeInvalidSurrogatesTest() {
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\\uD800\"}",
      "unpaired high surrogate");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\\uD800abcd\"}",
      "unpaired high surrogate");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\\uD800\\n\"}",
      "unpaired high surrogate");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\\uD800\\uD800\"}",
      "multiple high surrogates");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\\uDC00\"}",
      "unpaired low surrogate");
}

void InvalidUTF8Test() {
  // "1 byte" pattern, under min length of 2 bytes
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\x80\"}",
      "illegal UTF-8 sequence");
  // 2 byte pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xDF\"}",
      "illegal UTF-8 sequence");
  // 3 byte pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xEF\xBF\"}",
      "illegal UTF-8 sequence");
  // 4 byte pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xF7\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "5 byte" pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFB\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "6 byte" pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFD\xBF\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "7 byte" pattern, string too short
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFE\xBF\xBF\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "5 byte" pattern, over max length of 4 bytes
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFB\xBF\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "6 byte" pattern, over max length of 4 bytes
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFD\xBF\xBF\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");
  // "7 byte" pattern, over max length of 4 bytes
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xFE\xBF\xBF\xBF\xBF\xBF\xBF\"}",
      "illegal UTF-8 sequence");

  // Three invalid encodings for U+000A (\n, aka NEWLINE)
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xC0\x8A\"}",
      "illegal UTF-8 sequence");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xE0\x80\x8A\"}",
      "illegal UTF-8 sequence");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xF0\x80\x80\x8A\"}",
      "illegal UTF-8 sequence");

  // Two invalid encodings for U+00A9 (COPYRIGHT SYMBOL)
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xE0\x81\xA9\"}",
      "illegal UTF-8 sequence");
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xF0\x80\x81\xA9\"}",
      "illegal UTF-8 sequence");

  // Invalid encoding for U+20AC (EURO SYMBOL)
  TestError(
      "table T { F:string; }"
      "root_type T;"
      "{ F:\"\xF0\x82\x82\xAC\"}",
      "illegal UTF-8 sequence");

  // UTF-16 surrogate values between U+D800 and U+DFFF cannot be encoded in
  // UTF-8
  TestError(
      "table T { F:string; }"
      "root_type T;"
      // U+10400 "encoded" as U+D801 U+DC00
      "{ F:\"\xED\xA0\x81\xED\xB0\x80\"}",
      "illegal UTF-8 sequence");

  // Check independence of identifier from locale.
  std::string locale_ident;
  locale_ident += "table T { F";
  locale_ident += static_cast<char>(-32);  // unsigned 0xE0
  locale_ident += " :string; }";
  locale_ident += "root_type T;";
  locale_ident += "{}";
  TestError(locale_ident.c_str(), "");
}

template<typename T>
T TestValue(const char *json, const char *type_name,
            const char *decls = nullptr) {
  flatbuffers::Parser parser;
  parser.builder_.ForceDefaults(true);  // return defaults
  auto check_default = json ? false : true;
  if (check_default) { parser.opts.output_default_scalars_in_json = true; }
  // Simple schema.
  std::string schema = std::string(decls ? decls : "") + "\n" +
                       "table X { y:" + std::string(type_name) +
                       "; } root_type X;";
  auto schema_done = parser.Parse(schema.c_str());
  TEST_EQ_STR(parser.error_.c_str(), "");
  TEST_EQ(schema_done, true);

  auto done = parser.Parse(check_default ? "{}" : json);
  TEST_EQ_STR(parser.error_.c_str(), "");
  TEST_EQ(done, true);

  // Check with print.
  std::string print_back;
  parser.opts.indent_step = -1;
  TEST_NULL(GenText(parser, parser.builder_.GetBufferPointer(), &print_back));
  // restore value from its default
  if (check_default) { TEST_EQ(parser.Parse(print_back.c_str()), true); }

  auto root = flatbuffers::GetRoot<flatbuffers::Table>(
      parser.builder_.GetBufferPointer());
  return root->GetField<T>(flatbuffers::FieldIndexToOffset(0), 0);
}

// Additional parser testing not covered elsewhere.
void ValueTest() {
  // Test scientific notation numbers.
  TEST_EQ(
      FloatCompare(TestValue<float>("{ y:0.0314159e+2 }", "float"), 3.14159f),
      true);
  // number in string
  TEST_EQ(FloatCompare(TestValue<float>("{ y:\"0.0314159e+2\" }", "float"),
                       3.14159f),
          true);

  // Test conversion functions.
  TEST_EQ(FloatCompare(TestValue<float>("{ y:cos(rad(180)) }", "float"), -1),
          true);

  // int embedded to string
  TEST_EQ(TestValue<int>("{ y:\"-876\" }", "int=-123"), -876);
  TEST_EQ(TestValue<int>("{ y:\"876\" }", "int=-123"), 876);

  // Test negative hex constant.
  TEST_EQ(TestValue<int>("{ y:-0x8ea0 }", "int=-0x8ea0"), -36512);
  TEST_EQ(TestValue<int>(nullptr, "int=-0x8ea0"), -36512);

  // positive hex constant
  TEST_EQ(TestValue<int>("{ y:0x1abcdef }", "int=0x1"), 0x1abcdef);
  // with optional '+' sign
  TEST_EQ(TestValue<int>("{ y:+0x1abcdef }", "int=+0x1"), 0x1abcdef);
  // hex in string
  TEST_EQ(TestValue<int>("{ y:\"0x1abcdef\" }", "int=+0x1"), 0x1abcdef);

  // Make sure we do unsigned 64bit correctly.
  TEST_EQ(TestValue<uint64_t>("{ y:12335089644688340133 }", "ulong"),
          12335089644688340133ULL);

  // bool in string
  TEST_EQ(TestValue<bool>("{ y:\"false\" }", "bool=true"), false);
  TEST_EQ(TestValue<bool>("{ y:\"true\" }", "bool=\"true\""), true);
  TEST_EQ(TestValue<bool>("{ y:'false' }", "bool=true"), false);
  TEST_EQ(TestValue<bool>("{ y:'true' }", "bool=\"true\""), true);

  // check comments before and after json object
  TEST_EQ(TestValue<int>("/*before*/ { y:1 } /*after*/", "int"), 1);
  TEST_EQ(TestValue<int>("//before \n { y:1 } //after", "int"), 1);
}

void NestedListTest() {
  flatbuffers::Parser parser1;
  TEST_EQ(parser1.Parse("struct Test { a:short; b:byte; } table T { F:[Test]; }"
                        "root_type T;"
                        "{ F:[ [10,20], [30,40]] }"),
          true);
}

void EnumStringsTest() {
  flatbuffers::Parser parser1;
  TEST_EQ(parser1.Parse("enum E:byte { A, B, C } table T { F:[E]; }"
                        "root_type T;"
                        "{ F:[ A, B, \"C\", \"A B C\" ] }"),
          true);
  flatbuffers::Parser parser2;
  TEST_EQ(parser2.Parse("enum E:byte { A, B, C } table T { F:[int]; }"
                        "root_type T;"
                        "{ F:[ \"E.C\", \"E.A E.B E.C\" ] }"),
          true);
  // unsigned bit_flags
  flatbuffers::Parser parser3;
  TEST_EQ(
      parser3.Parse("enum E:uint16 (bit_flags) { F0, F07=7, F08, F14=14, F15 }"
                    " table T { F: E = \"F15 F08\"; }"
                    "root_type T;"),
      true);
}

void EnumValueTest() {
  // json: "{ Y:0 }", schema: table X { y: "E"}
  // 0 in enum (V=0) E then Y=0 is valid.
  TEST_EQ(TestValue<int>("{ y:0 }", "E", "enum E:int { V }"), 0);
  TEST_EQ(TestValue<int>("{ y:V }", "E", "enum E:int { V }"), 0);
  // A default value of Y is 0.
  TEST_EQ(TestValue<int>("{ }", "E", "enum E:int { V }"), 0);
  TEST_EQ(TestValue<int>("{ y:5 }", "E=V", "enum E:int { V=5 }"), 5);
  // Generate json with defaults and check.
  TEST_EQ(TestValue<int>(nullptr, "E=V", "enum E:int { V=5 }"), 5);
  // 5 in enum
  TEST_EQ(TestValue<int>("{ y:5 }", "E", "enum E:int { Z, V=5 }"), 5);
  TEST_EQ(TestValue<int>("{ y:5 }", "E=V", "enum E:int { Z, V=5 }"), 5);
  // Generate json with defaults and check.
  TEST_EQ(TestValue<int>(nullptr, "E", "enum E:int { Z, V=5 }"), 0);
  TEST_EQ(TestValue<int>(nullptr, "E=V", "enum E:int { Z, V=5 }"), 5);
  // u84 test
  TEST_EQ(TestValue<uint64_t>(nullptr, "E=V",
                              "enum E:ulong { V = 13835058055282163712 }"),
          13835058055282163712ULL);
  TEST_EQ(TestValue<uint64_t>(nullptr, "E=V",
                              "enum E:ulong { V = 18446744073709551615 }"),
          18446744073709551615ULL);
  // Assign non-enum value to enum field. Is it right?
  TEST_EQ(TestValue<int>("{ y:7 }", "E", "enum E:int { V = 0 }"), 7);
  // Check that non-ascending values are valid.
  TEST_EQ(TestValue<int>("{ y:5 }", "E=V", "enum E:int { Z=10, V=5 }"), 5);
}

void IntegerBoundaryTest() {
  // Check numerical compatibility with non-C++ languages.
  // By the C++ standard, std::numerical_limits<int64_t>::min() ==
  // -9223372036854775807 (-2^63+1) or less* The Flatbuffers grammar and most of
  // the languages (C#, Java, Rust) expect that minimum values are: -128,
  // -32768,.., -9223372036854775808. Since C++20,
  // static_cast<int64>(0x8000000000000000ULL) is well-defined two's complement
  // cast. Therefore -9223372036854775808 should be valid negative value.
  TEST_EQ(flatbuffers::numeric_limits<int8_t>::min(), -128);
  TEST_EQ(flatbuffers::numeric_limits<int8_t>::max(), 127);
  TEST_EQ(flatbuffers::numeric_limits<int16_t>::min(), -32768);
  TEST_EQ(flatbuffers::numeric_limits<int16_t>::max(), 32767);
  TEST_EQ(flatbuffers::numeric_limits<int32_t>::min() + 1, -2147483647);
  TEST_EQ(flatbuffers::numeric_limits<int32_t>::max(), 2147483647ULL);
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::min() + 1LL,
          -9223372036854775807LL);
  TEST_EQ(flatbuffers::numeric_limits<int64_t>::max(), 9223372036854775807ULL);
  TEST_EQ(flatbuffers::numeric_limits<uint8_t>::max(), 255);
  TEST_EQ(flatbuffers::numeric_limits<uint16_t>::max(), 65535);
  TEST_EQ(flatbuffers::numeric_limits<uint32_t>::max(), 4294967295ULL);
  TEST_EQ(flatbuffers::numeric_limits<uint64_t>::max(),
          18446744073709551615ULL);

  TEST_EQ(TestValue<int8_t>("{ y:127 }", "byte"), 127);
  TEST_EQ(TestValue<int8_t>("{ y:-128 }", "byte"), -128);
  TEST_EQ(TestValue<uint8_t>("{ y:255 }", "ubyte"), 255);
  TEST_EQ(TestValue<uint8_t>("{ y:0 }", "ubyte"), 0);
  TEST_EQ(TestValue<int16_t>("{ y:32767 }", "short"), 32767);
  TEST_EQ(TestValue<int16_t>("{ y:-32768 }", "short"), -32768);
  TEST_EQ(TestValue<uint16_t>("{ y:65535 }", "ushort"), 65535);
  TEST_EQ(TestValue<uint16_t>("{ y:0 }", "ushort"), 0);
  TEST_EQ(TestValue<int32_t>("{ y:2147483647 }", "int"), 2147483647);
  TEST_EQ(TestValue<int32_t>("{ y:-2147483648 }", "int") + 1, -2147483647);
  TEST_EQ(TestValue<uint32_t>("{ y:4294967295 }", "uint"), 4294967295);
  TEST_EQ(TestValue<uint32_t>("{ y:0 }", "uint"), 0);
  TEST_EQ(TestValue<int64_t>("{ y:9223372036854775807 }", "long"),
          9223372036854775807LL);
  TEST_EQ(TestValue<int64_t>("{ y:-9223372036854775808 }", "long") + 1LL,
          -9223372036854775807LL);
  TEST_EQ(TestValue<uint64_t>("{ y:18446744073709551615 }", "ulong"),
          18446744073709551615ULL);
  TEST_EQ(TestValue<uint64_t>("{ y:0 }", "ulong"), 0);
  TEST_EQ(TestValue<uint64_t>("{ y: 18446744073709551615 }", "uint64"),
          18446744073709551615ULL);
  // check that the default works
  TEST_EQ(TestValue<uint64_t>(nullptr, "uint64 = 18446744073709551615"),
          18446744073709551615ULL);
}

void ValidFloatTest() {
  // check rounding to infinity
  TEST_EQ(TestValue<float>("{ y:+3.4029e+38 }", "float"), +infinity_f);
  TEST_EQ(TestValue<float>("{ y:-3.4029e+38 }", "float"), -infinity_f);
  TEST_EQ(TestValue<double>("{ y:+1.7977e+308 }", "double"), +infinity_d);
  TEST_EQ(TestValue<double>("{ y:-1.7977e+308 }", "double"), -infinity_d);

  TEST_EQ(
      FloatCompare(TestValue<float>("{ y:0.0314159e+2 }", "float"), 3.14159f),
      true);
  // float in string
  TEST_EQ(FloatCompare(TestValue<float>("{ y:\" 0.0314159e+2  \" }", "float"),
                       3.14159f),
          true);

  TEST_EQ(TestValue<float>("{ y:1 }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:1.0 }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:1. }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:+1. }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:-1. }", "float"), -1.0f);
  TEST_EQ(TestValue<float>("{ y:1.e0 }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:1.e+0 }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:1.e-0 }", "float"), 1.0f);
  TEST_EQ(TestValue<float>("{ y:0.125 }", "float"), 0.125f);
  TEST_EQ(TestValue<float>("{ y:.125 }", "float"), 0.125f);
  TEST_EQ(TestValue<float>("{ y:-.125 }", "float"), -0.125f);
  TEST_EQ(TestValue<float>("{ y:+.125 }", "float"), +0.125f);
  TEST_EQ(TestValue<float>("{ y:5 }", "float"), 5.0f);
  TEST_EQ(TestValue<float>("{ y:\"5\" }", "float"), 5.0f);

#if defined(FLATBUFFERS_HAS_NEW_STRTOD) && (FLATBUFFERS_HAS_NEW_STRTOD > 0)
  // Old MSVC versions may have problem with this check.
  // https://www.exploringbinary.com/visual-c-plus-plus-strtod-still-broken/
  TEST_EQ(TestValue<double>("{ y:6.9294956446009195e15 }", "double"),
          6929495644600920.0);
  // check nan's
  TEST_EQ(std::isnan(TestValue<double>("{ y:nan }", "double")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:nan }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:\"nan\" }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:\"+nan\" }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:\"-nan\" }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:+nan }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>("{ y:-nan }", "float")), true);
  TEST_EQ(std::isnan(TestValue<float>(nullptr, "float=nan")), true);
  TEST_EQ(std::isnan(TestValue<float>(nullptr, "float=-nan")), true);
  // check inf
  TEST_EQ(TestValue<float>("{ y:inf }", "float"), infinity_f);
  TEST_EQ(TestValue<float>("{ y:\"inf\" }", "float"), infinity_f);
  TEST_EQ(TestValue<float>("{ y:\"-inf\" }", "float"), -infinity_f);
  TEST_EQ(TestValue<float>("{ y:\"+inf\" }", "float"), infinity_f);
  TEST_EQ(TestValue<float>("{ y:+inf }", "float"), infinity_f);
  TEST_EQ(TestValue<float>("{ y:-inf }", "float"), -infinity_f);
  TEST_EQ(TestValue<float>(nullptr, "float=inf"), infinity_f);
  TEST_EQ(TestValue<float>(nullptr, "float=-inf"), -infinity_f);
  TestValue<double>(
      "{ y: [0.2, .2, 1.0, -1.0, -2., 2., 1e0, -1e0, 1.0e0, -1.0e0, -3.e2, "
      "3.0e2] }",
      "[double]");
  TestValue<float>(
      "{ y: [0.2, .2, 1.0, -1.0, -2., 2., 1e0, -1e0, 1.0e0, -1.0e0, -3.e2, "
      "3.0e2] }",
      "[float]");

  // Test binary format of float point.
  // https://en.cppreference.com/w/cpp/language/floating_literal
  // 0x11.12p-1 = (1*16^1 + 2*16^0 + 3*16^-1 + 4*16^-2) * 2^-1 =
  TEST_EQ(TestValue<double>("{ y:0x12.34p-1 }", "double"), 9.1015625);
  // hex fraction 1.2 (decimal 1.125) scaled by 2^3, that is 9.0
  TEST_EQ(TestValue<float>("{ y:-0x0.2p0 }", "float"), -0.125f);
  TEST_EQ(TestValue<float>("{ y:-0x.2p1 }", "float"), -0.25f);
  TEST_EQ(TestValue<float>("{ y:0x1.2p3 }", "float"), 9.0f);
  TEST_EQ(TestValue<float>("{ y:0x10.1p0 }", "float"), 16.0625f);
  TEST_EQ(TestValue<double>("{ y:0x1.2p3 }", "double"), 9.0);
  TEST_EQ(TestValue<double>("{ y:0x10.1p0 }", "double"), 16.0625);
  TEST_EQ(TestValue<double>("{ y:0xC.68p+2 }", "double"), 49.625);
  TestValue<double>("{ y: [0x20.4ep1, +0x20.4ep1, -0x20.4ep1] }", "[double]");
  TestValue<float>("{ y: [0x20.4ep1, +0x20.4ep1, -0x20.4ep1] }", "[float]");

#else   // FLATBUFFERS_HAS_NEW_STRTOD
  TEST_OUTPUT_LINE("FLATBUFFERS_HAS_NEW_STRTOD tests skipped");
#endif  // !FLATBUFFERS_HAS_NEW_STRTOD
}

void UnicodeTest() {
  flatbuffers::Parser parser;
  // Without setting allow_non_utf8 = true, we treat \x sequences as byte
  // sequences which are then validated as UTF-8.
  TEST_EQ(parser.Parse("table T { F:string; }"
                       "root_type T;"
                       "{ F:\"\\u20AC\\u00A2\\u30E6\\u30FC\\u30B6\\u30FC"
                       "\\u5225\\u30B5\\u30A4\\u30C8\\xE2\\x82\\xAC\\u0080\\uD8"
                       "3D\\uDE0E\" }"),
          true);
  std::string jsongen;
  parser.opts.indent_step = -1;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(),
              "{F: \"\\u20AC\\u00A2\\u30E6\\u30FC\\u30B6\\u30FC"
              "\\u5225\\u30B5\\u30A4\\u30C8\\u20AC\\u0080\\uD83D\\uDE0E\"}");
}

void UnicodeTestAllowNonUTF8() {
  flatbuffers::Parser parser;
  parser.opts.allow_non_utf8 = true;
  TEST_EQ(
      parser.Parse(
          "table T { F:string; }"
          "root_type T;"
          "{ F:\"\\u20AC\\u00A2\\u30E6\\u30FC\\u30B6\\u30FC"
          "\\u5225\\u30B5\\u30A4\\u30C8\\x01\\x80\\u0080\\uD83D\\uDE0E\" }"),
      true);
  std::string jsongen;
  parser.opts.indent_step = -1;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(
      jsongen.c_str(),
      "{F: \"\\u20AC\\u00A2\\u30E6\\u30FC\\u30B6\\u30FC"
      "\\u5225\\u30B5\\u30A4\\u30C8\\u0001\\x80\\u0080\\uD83D\\uDE0E\"}");
}

void UnicodeTestGenerateTextFailsOnNonUTF8() {
  flatbuffers::Parser parser;
  // Allow non-UTF-8 initially to model what happens when we load a binary
  // flatbuffer from disk which contains non-UTF-8 strings.
  parser.opts.allow_non_utf8 = true;
  TEST_EQ(
      parser.Parse(
          "table T { F:string; }"
          "root_type T;"
          "{ F:\"\\u20AC\\u00A2\\u30E6\\u30FC\\u30B6\\u30FC"
          "\\u5225\\u30B5\\u30A4\\u30C8\\x01\\x80\\u0080\\uD83D\\uDE0E\" }"),
      true);
  std::string jsongen;
  parser.opts.indent_step = -1;
  // Now, disallow non-UTF-8 (the default behavior) so GenText indicates
  // failure.
  parser.opts.allow_non_utf8 = false;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_EQ_STR(result, "string contains non-utf8 bytes");
}

void UnicodeSurrogatesTest() {
  flatbuffers::Parser parser;

  TEST_EQ(parser.Parse("table T { F:string (id: 0); }"
                       "root_type T;"
                       "{ F:\"\\uD83D\\uDCA9\"}"),
          true);
  auto root = flatbuffers::GetRoot<flatbuffers::Table>(
      parser.builder_.GetBufferPointer());
  auto string = root->GetPointer<flatbuffers::String *>(
      flatbuffers::FieldIndexToOffset(0));
  TEST_EQ_STR(string->c_str(), "\xF0\x9F\x92\xA9");
}

void UnknownFieldsTest() {
  flatbuffers::IDLOptions opts;
  opts.skip_unexpected_fields_in_json = true;
  flatbuffers::Parser parser(opts);

  TEST_EQ(parser.Parse("table T { str:string; i:int;}"
                       "root_type T;"
                       "{ str:\"test\","
                       "unknown_string:\"test\","
                       "\"unknown_string\":\"test\","
                       "unknown_int:10,"
                       "unknown_float:1.0,"
                       "unknown_array: [ 1, 2, 3, 4],"
                       "unknown_object: { i: 10 },"
                       "\"unknown_object\": { \"i\": 10 },"
                       "i:10}"),
          true);

  std::string jsongen;
  parser.opts.indent_step = -1;
  auto result = GenText(parser, parser.builder_.GetBufferPointer(), &jsongen);
  TEST_NULL(result);
  TEST_EQ_STR(jsongen.c_str(), "{str: \"test\",i: 10}");
}

void ParseUnionTest() {
  // Unions must be parseable with the type field following the object.
  flatbuffers::Parser parser;
  TEST_EQ(parser.Parse("table T { A:int; }"
                       "union U { T }"
                       "table V { X:U; }"
                       "root_type V;"
                       "{ X:{ A:1 }, X_type: T }"),
          true);
  // Unions must be parsable with prefixed namespace.
  flatbuffers::Parser parser2;
  TEST_EQ(parser2.Parse("namespace N; table A {} namespace; union U { N.A }"
                        "table B { e:U; } root_type B;"
                        "{ e_type: N_A, e: {} }"),
          true);

  // Test union underlying type
  const char *source = "table A {} table B {} union U : int {A, B} table C {test_union: U; test_vector_of_union: [U];}";
  flatbuffers::Parser parser3;
  parser3.opts.lang_to_generate = flatbuffers::IDLOptions::kCpp | flatbuffers::IDLOptions::kTs;
  TEST_EQ(parser3.Parse(source), true);
  
  parser3.opts.lang_to_generate &= flatbuffers::IDLOptions::kJava;
  TEST_EQ(parser3.Parse(source), false);
}

void ValidSameNameDifferentNamespaceTest() {
  // Duplicate table names in different namespaces must be parsable
  TEST_ASSERT(flatbuffers::Parser().Parse(
      "namespace A; table X {} namespace B; table X {}"));
  // Duplicate union names in different namespaces must be parsable
  TEST_ASSERT(flatbuffers::Parser().Parse(
      "namespace A; union X {} namespace B; union X {}"));
  // Clashing table and union names in different namespaces must be parsable
  TEST_ASSERT(flatbuffers::Parser().Parse(
      "namespace A; table X {} namespace B; union X {}"));
  TEST_ASSERT(flatbuffers::Parser().Parse(
      "namespace A; union X {} namespace B; table X {}"));
}

void WarningsAsErrorsTest() {
  {
    flatbuffers::IDLOptions opts;
    // opts.warnings_as_errors should default to false
    flatbuffers::Parser parser(opts);
    TEST_EQ(parser.Parse("table T { THIS_NAME_CAUSES_A_WARNING:string;}\n"
                         "root_type T;"),
            true);
  }
  {
    flatbuffers::IDLOptions opts;
    opts.warnings_as_errors = true;
    flatbuffers::Parser parser(opts);
    TEST_EQ(parser.Parse("table T { THIS_NAME_CAUSES_A_WARNING:string;}\n"
                         "root_type T;"),
            false);
  }
}

void StringVectorDefaultsTest() {
  std::vector<std::string> schemas;
  schemas.push_back("table Monster { mana: string = \"\"; }");
  schemas.push_back("table Monster { mana: string = \"mystr\"; }");
  schemas.push_back("table Monster { mana: string = \"  \"; }");
  schemas.push_back("table Monster { mana: string = \"null\"; }");
  schemas.push_back("table Monster { mana: [int] = []; }");
  schemas.push_back("table Monster { mana: [uint] = [  ]; }");
  schemas.push_back("table Monster { mana: [byte] = [\t\t\n]; }");
  schemas.push_back("enum E:int{}table Monster{mana:[E]=[];}");
  for (auto s = schemas.begin(); s < schemas.end(); s++) {
    flatbuffers::Parser parser;
    TEST_ASSERT(parser.Parse(s->c_str()));
    const auto *mana = parser.structs_.Lookup("Monster")->fields.Lookup("mana");
    TEST_EQ(mana->IsDefault(), true);
  }
}

void FieldIdentifierTest() {
  using flatbuffers::Parser;
  TEST_EQ(true, Parser().Parse("table T{ f: int (id:0); }"));
  // non-integer `id` should be rejected
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:text); }"));
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:\"text\"); }"));
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:0text); }"));
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:1.0); }"));
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:-1); g: int (id:0); }"));
  TEST_EQ(false, Parser().Parse("table T{ f: int (id:129496726); }"));
  // A unuion filed occupys two ids: enumerator + pointer (offset).
  TEST_EQ(false,
          Parser().Parse("union X{} table T{ u: X(id:0); table F{x:int;\n}"));
  // Positive tests for unions
  TEST_EQ(true, Parser().Parse("union X{} table T{ u: X (id:1); }"));
  TEST_EQ(true, Parser().Parse("union X{} table T{ u: X; }"));
  // Test using 'inf' and 'nan' words both as identifiers and as default values.
  TEST_EQ(true, Parser().Parse("table T{ nan: string; }"));
  TEST_EQ(true, Parser().Parse("table T{ inf: string; }"));
#if defined(FLATBUFFERS_HAS_NEW_STRTOD) && (FLATBUFFERS_HAS_NEW_STRTOD > 0)
  TEST_EQ(true, Parser().Parse("table T{ inf: float = inf; }"));
  TEST_EQ(true, Parser().Parse("table T{ nan: float = inf; }"));
#endif
}

}  // namespace tests
}  // namespace flatbuffers
