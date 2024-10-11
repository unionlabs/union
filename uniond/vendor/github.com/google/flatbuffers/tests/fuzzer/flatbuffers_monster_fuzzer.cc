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

#include <stddef.h>
#include <stdint.h>

#include <clocale>
#include <filesystem>
#include <string>

#include "cpp17/generated_cpp17/monster_test_generated.h"
#include "flatbuffers/idl.h"
#include "test_assert.h"
#include "test_init.h"

namespace fs = std::filesystem;

// Utility for test run.
OneTimeTestInit OneTimeTestInit::one_time_init_;
// The current executable path (see LLVMFuzzerInitialize).
static fs::path exe_path_;

static flatbuffers::Parser parser_;

namespace {

static constexpr size_t kMinInputLength = 1;
static constexpr size_t kMaxInputLength = 16384;

static constexpr uint8_t flags_strict_json = 0x80;
static constexpr uint8_t flags_skip_unexpected_fields_in_json = 0x40;
static constexpr uint8_t flags_allow_non_utf8 = 0x20;

bool TestFileExists(fs::path file_path) {
  if (file_path.has_filename() && fs::exists(file_path)) return true;

  TEST_OUTPUT_LINE("@DEBUG: file '%s' not found", file_path.string().c_str());
  for (const auto &entry : fs::directory_iterator(file_path.parent_path())) {
    TEST_OUTPUT_LINE("@DEBUG: parent path entry: '%s'",
                     entry.path().string().c_str());
  }
  return false;
}

std::string LoadBinarySchema(const char *file_name) {
  const auto file_path = exe_path_.parent_path() / file_name;
  TEST_EQ(true, TestFileExists(file_path));
  std::string schemafile;
  TEST_EQ(true,
          flatbuffers::LoadFile(file_path.string().c_str(), true, &schemafile));

  flatbuffers::Verifier verifier(
      reinterpret_cast<const uint8_t *>(schemafile.c_str()), schemafile.size());
  TEST_EQ(true, reflection::VerifySchemaBuffer(verifier));
  return schemafile;
}

std::string do_test(const flatbuffers::IDLOptions &opts,
                    const std::string input_json, const bool check_parser) {
  // (re)define parser options
  parser_.opts = opts;

  std::string jsongen;
  if (parser_.ParseJson(input_json.c_str())) {
    flatbuffers::Verifier verifier(parser_.builder_.GetBufferPointer(),
                                   parser_.builder_.GetSize());
    TEST_EQ(true, MyGame::Example::VerifyMonsterBuffer(verifier));
    TEST_NULL(
        GenText(parser_, parser_.builder_.GetBufferPointer(), &jsongen));
  } else if (check_parser) {
    TEST_OUTPUT_LINE("parser failed with JSON:\n%s", input_json.c_str());
    TEST_EQ_STR("", parser_.error_.c_str());
    TEST_ASSERT(false);
  }
  return jsongen;
};
}  // namespace

// https://google.github.io/oss-fuzz/further-reading/fuzzer-environment/
// Current working directory
// You should not make any assumptions about the current working directory of
// your fuzz target. If you need to load data files, please use argv[0] to get
// the directory where your fuzz target executable is located.
// You must not modify argv[0].
extern "C" int LLVMFuzzerInitialize(int *argc, char ***argv) {
  (void)argc;
  exe_path_ = (*argv)[0];

  static const std::string schemafile = LoadBinarySchema("monster_test.bfbs");
  // parse schema first, so we can use it to parse the data after
  parser_.Deserialize(reinterpret_cast<const uint8_t *>(schemafile.c_str()),
                      schemafile.size());
  return 0;
}

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  // Reserve one byte for Parser flags and one byte for repetition counter.
  if (size < 3) return 0;
  const uint8_t flags = data[0];
  (void)data[1];  //  reserved
  data += 2;
  size -= 2;  // bypass

  const std::string original(reinterpret_cast<const char *>(data), size);
  auto input = std::string(original.c_str());  // until '\0'
  if (input.size() < kMinInputLength || input.size() > kMaxInputLength)
    return 0;

  flatbuffers::IDLOptions opts;
  opts.strict_json = (flags & flags_strict_json);
  opts.skip_unexpected_fields_in_json =
      (flags & flags_skip_unexpected_fields_in_json);
  opts.allow_non_utf8 = (flags & flags_allow_non_utf8);

  do {
    const std::string jsongen_1 = do_test(opts, input, false);
    if (!jsongen_1.empty()) {
      const std::string jsongen_2 = do_test(opts, jsongen_1, true);
      if (jsongen_1 != jsongen_2 && !opts.output_default_scalars_in_json) {
        // This gets tricky when the jsongen_1 includes a default-value, as the
        // generated jsongen_2 doesn't emit default-values. So enable default
        // scalars and re-run it.
        opts.output_default_scalars_in_json = true;
        continue;
      }
      TEST_EQ(jsongen_1, jsongen_2);
    }
  } while (0);

  return 0;
}
