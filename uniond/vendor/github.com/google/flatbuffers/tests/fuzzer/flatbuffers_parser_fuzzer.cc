// Copyright 2015 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
#include <stddef.h>
#include <stdint.h>
#include <clocale>
#include <string>

#include "flatbuffers/idl.h"
#include "test_init.h"

static constexpr size_t kMinInputLength = 1;
static constexpr size_t kMaxInputLength = 16384;

static constexpr uint8_t flags_strict_json = 0x80;
static constexpr uint8_t flags_skip_unexpected_fields_in_json = 0x40;
static constexpr uint8_t flags_allow_non_utf8 = 0x20;

// Utility for test run.
OneTimeTestInit OneTimeTestInit::one_time_init_;

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

  flatbuffers::Parser parser(opts);

  // Guarantee 0-termination in the input.
  auto parse_input = input.c_str();

  // Check Parser.
  parser.Parse(parse_input);
  // TODO:
  // Need to add additional checks for inputs passed Parse(parse_input) successfully:
  // 1. Serialization to bfbs.
  // 2. Generation of a default object.
  // 3. Verification of the object using reflection.
  // 3. Printing to json.
  return 0;
}
