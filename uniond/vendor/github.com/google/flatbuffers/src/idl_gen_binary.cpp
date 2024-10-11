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

// independent from idl_parser, since this code is not needed for most clients

#include "idl_gen_binary.h"

#include <limits>
#include <memory>
#include <string>
#include <unordered_set>

#include "flatbuffers/base.h"
#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/flatc.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"

namespace flatbuffers {
namespace {

static std::string BinaryFileName(const Parser &parser, const std::string &path,
                                  const std::string &file_name) {
  auto ext = parser.file_extension_.length() ? parser.file_extension_ : "bin";
  return path + file_name + "." + ext;
}

static bool GenerateBinary(const Parser &parser, const std::string &path,
                           const std::string &file_name) {
  if (parser.opts.use_flexbuffers) {
    auto data_vec = parser.flex_builder_.GetBuffer();
    auto data_ptr = reinterpret_cast<char *>(data(data_vec));
    return !parser.flex_builder_.GetSize() ||
           flatbuffers::SaveFile(
               BinaryFileName(parser, path, file_name).c_str(), data_ptr,
               parser.flex_builder_.GetSize(), true);
  }
  return !parser.builder_.GetSize() ||
         flatbuffers::SaveFile(
             BinaryFileName(parser, path, file_name).c_str(),
             reinterpret_cast<char *>(parser.builder_.GetBufferPointer()),
             parser.builder_.GetSize(), true);
}

static std::string BinaryMakeRule(const Parser &parser, const std::string &path,
                                  const std::string &file_name) {
  if (!parser.builder_.GetSize()) return "";
  std::string filebase =
      flatbuffers::StripPath(flatbuffers::StripExtension(file_name));
  std::string make_rule =
      BinaryFileName(parser, path, filebase) + ": " + file_name;
  auto included_files =
      parser.GetIncludedFilesRecursive(parser.root_struct_def_->file);
  for (auto it = included_files.begin(); it != included_files.end(); ++it) {
    make_rule += " " + *it;
  }
  return make_rule;
}

class BinaryCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateBinary(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  // Generate code from the provided `buffer` of given `length`. The buffer is a
  // serialized reflection.fbs.
  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = BinaryMakeRule(parser, path, filename);
    return Status::OK;
  }

  Status GenerateGrpcCode(const Parser &parser, const std::string &path,
                          const std::string &filename) override {
    (void)parser;
    (void)path;
    (void)filename;
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateRootFile(const Parser &parser,
                          const std::string &path) override {
    (void)parser;
    (void)path;
    return Status::NOT_IMPLEMENTED;
  }

  bool IsSchemaOnly() const override { return false; }

  bool SupportsBfbsGeneration() const override { return false; }

  bool SupportsRootFileGeneration() const override { return false; }

  IDLOptions::Language Language() const override { return IDLOptions::kBinary; }

  std::string LanguageName() const override { return "binary"; }
};

}  // namespace

std::unique_ptr<CodeGenerator> NewBinaryCodeGenerator() {
  return std::unique_ptr<BinaryCodeGenerator>(new BinaryCodeGenerator());
}

}  // namespace flatbuffers
