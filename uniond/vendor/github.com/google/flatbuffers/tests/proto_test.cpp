#include "proto_test.h"

#include "flatbuffers/code_generator.h"
#include "idl_gen_fbs.h"
#include "test_assert.h"

namespace flatbuffers {
namespace tests {

void RunTest(const flatbuffers::IDLOptions &opts, const std::string &proto_path,
             const std::string &proto_file, const std::string &golden_file,
             const std::string import_proto_file) {
  const char *include_directories[] = { proto_path.c_str(), nullptr };

  // Parse proto.
  flatbuffers::Parser parser(opts);
  TEST_EQ(parser.Parse(proto_file.c_str(), include_directories), true);

  // Generate fbs.
  std::unique_ptr<CodeGenerator> fbs_generator = NewFBSCodeGenerator(true);
  std::string fbs;
  TEST_EQ(fbs_generator->GenerateCodeString(parser, "test", fbs),
          CodeGenerator::Status::OK);

  // Ensure generated file is parsable.
  flatbuffers::Parser parser2;

  if (!import_proto_file.empty()) {
    // Generate fbs from import.proto
    flatbuffers::Parser import_parser(opts);
    TEST_EQ(import_parser.Parse(import_proto_file.c_str(), include_directories),
            true);
    std::string import_fbs;
    TEST_EQ(
        fbs_generator->GenerateCodeString(import_parser, "test", import_fbs),
        CodeGenerator::Status::OK);
    // auto import_fbs = flatbuffers::GenerateFBS(import_parser, "test", true);
    // Since `imported.fbs` isn't in the filesystem AbsolutePath can't figure it
    // out by itself. We manually construct it so Parser works.
    std::string imported_fbs = flatbuffers::PosixPath(
        flatbuffers::AbsolutePath(proto_path) + "/imported.fbs");
    TEST_EQ(parser2.Parse(import_fbs.c_str(), include_directories,
                          imported_fbs.c_str()),
            true);
  }

  TEST_EQ(parser2.Parse(fbs.c_str(), nullptr), true);
  TEST_EQ_STR(fbs.c_str(), golden_file.c_str());
}

void proto_test(const std::string &proto_path, const std::string &proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  // load the .proto and the golden file from disk
  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile((proto_path + "test.golden.fbs").c_str(), false,
                                &golden_file),
          true);

  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_id(const std::string &proto_path,
                   const std::string &proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.keep_proto_id = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  // load the .proto and the golden file from disk
  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile((proto_path + "test_id.golden.fbs").c_str(),
                                false, &golden_file),
          true);

  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_union(const std::string &proto_path,
                      const std::string &proto_file) {
  // Parse proto with --oneof-union option.
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.proto_oneof_union = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile((proto_path + "test_union.golden.fbs").c_str(),
                                false, &golden_file),
          true);
  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_union_id(const std::string &proto_path,
                         const std::string &proto_file) {
  // Parse proto with --oneof-union option.
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.proto_oneof_union = true;
  opts.keep_proto_id = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(
      flatbuffers::LoadFile((proto_path + "test_union_id.golden.fbs").c_str(),
                            false, &golden_file),
      true);
  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_union_suffix(const std::string &proto_path,
                             const std::string &proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.proto_namespace_suffix = "test_namespace_suffix";
  opts.proto_oneof_union = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile(
              (proto_path + "test_union_suffix.golden.fbs").c_str(), false,
              &golden_file),
          true);
  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_union_suffix_id(const std::string &proto_path,
                                const std::string &proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = false;
  opts.proto_mode = true;
  opts.proto_namespace_suffix = "test_namespace_suffix";
  opts.proto_oneof_union = true;
  opts.keep_proto_id = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile(
              (proto_path + "test_union_suffix_id.golden.fbs").c_str(), false,
              &golden_file),
          true);
  RunTest(opts, proto_path, proto_file, golden_file);
}

void proto_test_include(const std::string &proto_path,
                        const std::string &proto_file,
                        const std::string &import_proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = true;
  opts.proto_mode = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(
      flatbuffers::LoadFile((proto_path + "test_include.golden.fbs").c_str(),
                            false, &golden_file),
      true);

  RunTest(opts, proto_path, proto_file, golden_file, import_proto_file);
}

void proto_test_include_id(const std::string &proto_path,
                           const std::string &proto_file,
                           const std::string &import_proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = true;
  opts.proto_mode = true;
  opts.keep_proto_id = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(
      flatbuffers::LoadFile((proto_path + "test_include_id.golden.fbs").c_str(),
                            false, &golden_file),
      true);

  RunTest(opts, proto_path, proto_file, golden_file, import_proto_file);
}

void proto_test_include_union(const std::string &proto_path,
                              const std::string &proto_file,
                              const std::string &import_proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = true;
  opts.proto_mode = true;
  opts.proto_oneof_union = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile(
              (proto_path + "test_union_include.golden.fbs").c_str(), false,
              &golden_file),
          true);

  RunTest(opts, proto_path, proto_file, golden_file, import_proto_file);
}

void proto_test_include_union_id(const std::string &proto_path,
                                 const std::string &proto_file,
                                 const std::string &import_proto_file) {
  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = true;
  opts.proto_mode = true;
  opts.proto_oneof_union = true;
  opts.keep_proto_id = true;
  opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::NO_OP;

  std::string golden_file;
  TEST_EQ(flatbuffers::LoadFile(
              (proto_path + "test_union_include_id.golden.fbs").c_str(), false,
              &golden_file),
          true);

  RunTest(opts, proto_path, proto_file, golden_file, import_proto_file);
}

void ParseCorruptedProto(const std::string &proto_path) {
  const char *include_directories[] = { proto_path.c_str(), nullptr };

  flatbuffers::IDLOptions opts;
  opts.include_dependence_headers = true;
  opts.proto_mode = true;
  opts.proto_oneof_union = true;

  std::string proto_file;

  std::unique_ptr<CodeGenerator> fbs_generator = NewFBSCodeGenerator(true);

  // Parse proto with non positive id.
  {
    flatbuffers::Parser parser(opts);
    TEST_EQ(
        flatbuffers::LoadFile((proto_path + "non-positive-id.proto").c_str(),
                              false, &proto_file),
        true);
    TEST_EQ(parser.Parse(proto_file.c_str(), include_directories), true);
    TEST_NE(fbs_generator->GenerateCode(parser, "temp.fbs", "test"),
            CodeGenerator::Status::OK);
  }

  // Parse proto with twice id.
  {
    flatbuffers::Parser parser(opts);
    TEST_EQ(flatbuffers::LoadFile((proto_path + "twice-id.proto").c_str(),
                                  false, &proto_file),
            true);
    TEST_EQ(parser.Parse(proto_file.c_str(), include_directories), true);
    TEST_NE(fbs_generator->GenerateCode(parser, "temp.fbs", "test"),
            CodeGenerator::Status::OK);
  }

  // Parse proto with using reserved id.
  {
    flatbuffers::Parser parser(opts);
    TEST_EQ(flatbuffers::LoadFile((proto_path + "twice-id.proto").c_str(),
                                  false, &proto_file),
            true);
    TEST_EQ(parser.Parse(proto_file.c_str(), include_directories), true);
    TEST_NE(fbs_generator->GenerateCode(parser, "temp.fbs", "test"),
            CodeGenerator::Status::OK);
  }

  // Parse proto with error on gap.
  {
    opts.proto_id_gap_action = IDLOptions::ProtoIdGapAction::ERROR;
    flatbuffers::Parser parser(opts);
    TEST_EQ(flatbuffers::LoadFile((proto_path + "test.proto").c_str(), false,
                                  &proto_file),
            true);
    TEST_EQ(parser.Parse(proto_file.c_str(), include_directories), true);

    TEST_NE(fbs_generator->GenerateCode(parser, "temp.fbs", "test"),
            CodeGenerator::Status::OK);
  }
}

// Parse a .proto schema, output as .fbs
void ParseProtoTest(const std::string &tests_data_path) {
  auto proto_path = tests_data_path + "prototest/";
  std::string proto_file;
  TEST_EQ(
      flatbuffers::LoadFile((tests_data_path + "prototest/test.proto").c_str(),
                            false, &proto_file),
      true);

  std::string import_proto_file;
  TEST_EQ(flatbuffers::LoadFile(
              (tests_data_path + "prototest/imported.proto").c_str(), false,
              &import_proto_file),
          true);

  proto_test(proto_path, proto_file);
  proto_test_union(proto_path, proto_file);
  proto_test_union_suffix(proto_path, proto_file);
  proto_test_include(proto_path, proto_file, import_proto_file);
  proto_test_include_union(proto_path, proto_file, import_proto_file);

  proto_test_id(proto_path, proto_file);
  proto_test_union_id(proto_path, proto_file);
  proto_test_union_suffix_id(proto_path, proto_file);
  proto_test_include_id(proto_path, proto_file, import_proto_file);
  proto_test_include_union_id(proto_path, proto_file, import_proto_file);

  ParseCorruptedProto(proto_path);
}

void ParseProtoBufAsciiTest() {
  // We can put the parser in a mode where it will accept JSON that looks more
  // like Protobuf ASCII, for users that have data in that format.
  // This uses no "" for field names (which we already support by default,
  // omits `,`, `:` before `{` and a couple of other features.
  flatbuffers::Parser parser;
  parser.opts.protobuf_ascii_alike = true;
  TEST_EQ(
      parser.Parse("table S { B:int; } table T { A:[int]; C:S; } root_type T;"),
      true);
  TEST_EQ(parser.Parse("{ A [1 2] C { B:2 }}"), true);
  // Similarly, in text output, it should omit these.
  std::string text;
  auto err =
      flatbuffers::GenText(
      parser, parser.builder_.GetBufferPointer(), &text);
  TEST_NULL(err);
  TEST_EQ_STR(text.c_str(),
              "{\n  A [\n    1\n    2\n  ]\n  C {\n    B: 2\n  }\n}\n");
}

}  // namespace tests
}  // namespace flatbuffers
