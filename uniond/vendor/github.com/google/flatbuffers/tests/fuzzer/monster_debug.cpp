#include <iostream>
#include <assert.h>

#include "flatbuffers/util.h"

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size);

int main(int argc, char *argv[]) {
  if (argc < 2) {
    std::cerr << "Usage: monster_debug <path to fuzzer crash file>\n";
    return 0;
  }
  std::string crash_file_name(argv[1]);
  std::string crash_file_data;
  auto done =
      flatbuffers::LoadFile(crash_file_name.c_str(), true, &crash_file_data);
  if (!done) {
    std::cerr << "Can not load file: '" << crash_file_name << "'";
    return -1;
  }
  if (crash_file_data.size() < 3) {
    std::cerr << "Invalid file data: '" << crash_file_data << "'";
    return -2;
  }
  auto rc = LLVMFuzzerTestOneInput(
      reinterpret_cast<const uint8_t *>(crash_file_data.data()),
      crash_file_data.size());
  std::cout << "LLVMFuzzerTestOneInput finished with code " << rc << "\n\n";
  return rc;
}
