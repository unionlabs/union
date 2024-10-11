#include "util_test.h"

#include "flatbuffers/util.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {
namespace {

template<typename T>
void NumericUtilsTestInteger(const char *lower, const char *upper) {
  T x;
  TEST_EQ(flatbuffers::StringToNumber("1q", &x), false);
  TEST_EQ(x, 0);
  TEST_EQ(flatbuffers::StringToNumber(upper, &x), false);
  TEST_EQ(x, flatbuffers::numeric_limits<T>::max());
  TEST_EQ(flatbuffers::StringToNumber(lower, &x), false);
  auto expval = flatbuffers::is_unsigned<T>::value
                    ? flatbuffers::numeric_limits<T>::max()
                    : flatbuffers::numeric_limits<T>::lowest();
  TEST_EQ(x, expval);
}

template<typename T>
void NumericUtilsTestFloat(const char *lower, const char *upper) {
  T f;
  TEST_EQ(flatbuffers::StringToNumber("", &f), false);
  TEST_EQ(flatbuffers::StringToNumber("1q", &f), false);
  TEST_EQ(f, 0);
  TEST_EQ(flatbuffers::StringToNumber(upper, &f), true);
  TEST_EQ(f, +flatbuffers::numeric_limits<T>::infinity());
  TEST_EQ(flatbuffers::StringToNumber(lower, &f), true);
  TEST_EQ(f, -flatbuffers::numeric_limits<T>::infinity());
}
}  // namespace

void NumericUtilsTest() {
  NumericUtilsTestInteger<uint64_t>("-1", "18446744073709551616");
  NumericUtilsTestInteger<uint8_t>("-1", "256");
  NumericUtilsTestInteger<int64_t>("-9223372036854775809",
                                   "9223372036854775808");
  NumericUtilsTestInteger<int8_t>("-129", "128");
  NumericUtilsTestFloat<float>("-3.4029e+38", "+3.4029e+38");
  NumericUtilsTestFloat<float>("-1.7977e+308", "+1.7977e+308");
}

void IsAsciiUtilsTest() {
  char c = -128;
  for (int cnt = 0; cnt < 256; cnt++) {
    auto alpha = (('a' <= c) && (c <= 'z')) || (('A' <= c) && (c <= 'Z'));
    auto dec = (('0' <= c) && (c <= '9'));
    auto hex = (('a' <= c) && (c <= 'f')) || (('A' <= c) && (c <= 'F'));
    TEST_EQ(flatbuffers::is_alpha(c), alpha);
    TEST_EQ(flatbuffers::is_alnum(c), alpha || dec);
    TEST_EQ(flatbuffers::is_digit(c), dec);
    TEST_EQ(flatbuffers::is_xdigit(c), dec || hex);
    c += 1;
  }
}

void UtilConvertCase() {
  {
    struct TestCase {
      std::string input;
      flatbuffers::Case output_case;
      std::string expected_output;
    };

    std::vector<TestCase> cases;

    // Tests for the common cases
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kUpperCamel,
                      "TheQuickBrownFox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kLowerCamel,
                      "theQuickBrownFox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kSnake,
                      "the_quick_brown_fox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kScreamingSnake,
                      "THE_QUICK_BROWN_FOX" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kAllLower,
                      "the_quick_brown_fox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kAllUpper,
                      "THE_QUICK_BROWN_FOX" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kUnknown,
                      "the_quick_brown_fox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kKeep,
                      "the_quick_brown_fox" });
    cases.push_back({ "the_quick_brown_fox", flatbuffers::Case::kSnake2,
                      "the_quick_brown_fox" });

    // Tests for some snake_cases where the _ is oddly placed or
    // missing.
    cases.push_back({ "single", flatbuffers::Case::kUpperCamel, "Single" });
    cases.push_back({ "Single", flatbuffers::Case::kUpperCamel, "Single" });
    cases.push_back({ "_leading", flatbuffers::Case::kUpperCamel, "_Leading" });
    cases.push_back(
        { "trailing_", flatbuffers::Case::kUpperCamel, "Trailing_" });
    cases.push_back({ "double__underscore", flatbuffers::Case::kUpperCamel,
                      "Double_underscore" });
    cases.push_back({ "single", flatbuffers::Case::kLowerCamel, "single" });
    cases.push_back({ "Single", flatbuffers::Case::kLowerCamel, "single" });
    cases.push_back({ "_leading", flatbuffers::Case::kLowerCamel, "_Leading" });
    cases.push_back(
        { "trailing_", flatbuffers::Case::kLowerCamel, "trailing_" });
    cases.push_back({ "double__underscore", flatbuffers::Case::kLowerCamel,
                      "double_underscore" });

    // Tests for some output snake_cases
    cases.push_back({ "single", flatbuffers::Case::kSnake, "single" });
    cases.push_back({ "single", flatbuffers::Case::kScreamingSnake, "SINGLE" });
    cases.push_back(
        { "_leading", flatbuffers::Case::kScreamingSnake, "_LEADING" });
    cases.push_back(
        { "trailing_", flatbuffers::Case::kScreamingSnake, "TRAILING_" });
    cases.push_back({ "double__underscore", flatbuffers::Case::kScreamingSnake,
                      "DOUBLE__UNDERSCORE" });

    for (auto &test_case : cases) {
      TEST_EQ(test_case.expected_output,
              flatbuffers::ConvertCase(test_case.input, test_case.output_case));
    }
  }

  // Tests for the non snake_case inputs.
  {
    struct TestCase {
      flatbuffers::Case input_case;
      std::string input;
      flatbuffers::Case output_case;
      std::string expected_output;
    };

    std::vector<TestCase> cases;

    cases.push_back({ flatbuffers::Case::kUpperCamel, "TheQuickBrownFox",
                      flatbuffers::Case::kSnake, "the_quick_brown_fox" });
    cases.push_back({ flatbuffers::Case::kLowerCamel, "theQuickBrownFox",
                      flatbuffers::Case::kSnake, "the_quick_brown_fox" });
    cases.push_back({ flatbuffers::Case::kSnake, "the_quick_brown_fox",
                      flatbuffers::Case::kSnake, "the_quick_brown_fox" });
    cases.push_back({ flatbuffers::Case::kScreamingSnake, "THE_QUICK_BROWN_FOX",
                      flatbuffers::Case::kSnake, "THE_QUICK_BROWN_FOX" });
    cases.push_back({ flatbuffers::Case::kAllUpper, "SINGLE",
                      flatbuffers::Case::kSnake, "SINGLE" });
    cases.push_back({ flatbuffers::Case::kAllLower, "single",
                      flatbuffers::Case::kSnake, "single" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "ABCtest",
                      flatbuffers::Case::kSnake, "abctest" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "tHe_qUiCk_BrOwN_fOx",
                      flatbuffers::Case::kKeep, "tHe_qUiCk_BrOwN_fOx" });
    cases.push_back({ flatbuffers::Case::kLowerCamel, "theQuick12345Fox",
                      flatbuffers::Case::kSnake, "the_quick_12345_fox" });
    cases.push_back({ flatbuffers::Case::kLowerCamel, "a12b34c45",
                      flatbuffers::Case::kSnake, "a_12b_34c_45" });
    cases.push_back({ flatbuffers::Case::kLowerCamel, "a12b34c45",
                      flatbuffers::Case::kSnake2, "a12_b34_c45" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "Int32Stamped",
                      flatbuffers::Case::kSnake, "int_32_stamped" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "101DogsTest",
                      flatbuffers::Case::kSnake, "101_dogs_test" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "Int32Stamped",
                      flatbuffers::Case::kScreamingSnake, "INT_32_STAMPED" });
    cases.push_back({ flatbuffers::Case::kUpperCamel, "101DogsTest",
                      flatbuffers::Case::kScreamingSnake, "101_DOGS_TEST" });

    for (auto &test_case : cases) {
      TEST_EQ(test_case.expected_output,
              flatbuffers::ConvertCase(test_case.input, test_case.output_case,
                                       test_case.input_case));
    }
  }
}

}  // namespace tests
}  // namespace flatbuffers
