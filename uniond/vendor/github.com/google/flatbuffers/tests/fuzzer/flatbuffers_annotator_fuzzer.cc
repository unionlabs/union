
#include <filesystem>
#include <string>

#include "binary_annotator.h"
#include "test_init.h"

static std::filesystem::path exe_path_;
static const uint8_t *schema_bfbs_;
static size_t schema_bfbs_length_;

bool TestFileExists(std::filesystem::path file_path) {
  if (file_path.has_filename() && std::filesystem::exists(file_path))
    return true;

  TEST_OUTPUT_LINE("@DEBUG: file '%s' not found", file_path.string().c_str());
  for (const auto &entry :
       std::filesystem::directory_iterator(file_path.parent_path())) {
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

extern "C" int LLVMFuzzerInitialize(int *, char ***argv) {
  exe_path_ = (*argv)[0];
  static const std::string schema_file =
      LoadBinarySchema("annotated_binary.bfbs");
  schema_bfbs_ = reinterpret_cast<const uint8_t *>(schema_file.c_str());
  schema_bfbs_length_ = schema_file.size();
  return 0;
}

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  flatbuffers::BinaryAnnotator annotator(schema_bfbs_, schema_bfbs_length_,
                                         data, size, false);

  annotator.Annotate();
  return 0;
}