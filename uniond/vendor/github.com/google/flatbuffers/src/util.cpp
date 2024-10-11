/*
 * Copyright 2016 Google Inc. All rights reserved.
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

// clang-format off
// Dont't remove `format off`, it prevent reordering of win-includes.

#include <cstring>
#if defined(__MINGW32__) || defined(__MINGW64__) || defined(__CYGWIN__) || \
    defined(__QNXNTO__)
#  define _POSIX_C_SOURCE 200809L
#  define _XOPEN_SOURCE 700L
#endif

#if defined(_WIN32) || defined(__MINGW32__) || defined(__MINGW64__) || defined(__CYGWIN__)
#  ifndef WIN32_LEAN_AND_MEAN
#    define WIN32_LEAN_AND_MEAN
#  endif
#  ifndef NOMINMAX
#    define NOMINMAX
#  endif
#  ifdef _MSC_VER
#    include <crtdbg.h>
#  endif
#  include <windows.h>  // Must be included before <direct.h>
#  ifndef __CYGWIN__
#    include <direct.h>
#  endif
#  include <winbase.h>
#  undef interface  // This is also important because of reasons
#endif
// clang-format on

#include "flatbuffers/util.h"

#include <sys/stat.h>

#include <clocale>
#include <cstdlib>
#include <fstream>
#include <functional>

#include "flatbuffers/base.h"

namespace flatbuffers {

namespace {

static bool FileExistsRaw(const char *name) {
  std::ifstream ifs(name);
  return ifs.good();
}

static bool LoadFileRaw(const char *name, bool binary, std::string *buf) {
  if (DirExists(name)) return false;
  std::ifstream ifs(name, binary ? std::ifstream::binary : std::ifstream::in);
  if (!ifs.is_open()) return false;
  if (binary) {
    // The fastest way to read a file into a string.
    ifs.seekg(0, std::ios::end);
    auto size = ifs.tellg();
    (*buf).resize(static_cast<size_t>(size));
    ifs.seekg(0, std::ios::beg);
    ifs.read(&(*buf)[0], (*buf).size());
  } else {
    // This is slower, but works correctly on all platforms for text files.
    std::ostringstream oss;
    oss << ifs.rdbuf();
    *buf = oss.str();
  }
  return !ifs.bad();
}

LoadFileFunction g_load_file_function = LoadFileRaw;
FileExistsFunction g_file_exists_function = FileExistsRaw;

static std::string ToCamelCase(const std::string &input, bool is_upper) {
  std::string s;
  for (size_t i = 0; i < input.length(); i++) {
    if (!i && input[i] == '_') {
      s += input[i];
      // we ignore leading underscore but make following
      // alphabet char upper.
      if (i + 1 < input.length() && is_alpha(input[i + 1]))
        s += CharToUpper(input[++i]);
    } else if (!i)
      s += is_upper ? CharToUpper(input[i]) : CharToLower(input[i]);
    else if (input[i] == '_' && i + 1 < input.length())
      s += CharToUpper(input[++i]);
    else
      s += input[i];
  }
  return s;
}

static std::string ToSnakeCase(const std::string &input, bool screaming) {
  std::string s;
  for (size_t i = 0; i < input.length(); i++) {
    if (i == 0) {
      s += screaming ? CharToUpper(input[i]) : CharToLower(input[i]);
    } else if (input[i] == '_') {
      s += '_';
    } else if (!islower(input[i])) {
      // Prevent duplicate underscores for Upper_Snake_Case strings
      // and UPPERCASE strings.
      if (islower(input[i - 1]) ||
          (isdigit(input[i - 1]) && !isdigit(input[i]))) {
        s += '_';
      }
      s += screaming ? CharToUpper(input[i]) : CharToLower(input[i]);
    } else {
      s += screaming ? CharToUpper(input[i]) : input[i];
    }
  }
  return s;
}

std::string ToAll(const std::string &input,
                  std::function<char(const char)> transform) {
  std::string s;
  for (size_t i = 0; i < input.length(); i++) { s += transform(input[i]); }
  return s;
}

std::string CamelToSnake(const std::string &input) {
  std::string s;
  for (size_t i = 0; i < input.length(); i++) {
    if (i == 0) {
      s += CharToLower(input[i]);
    } else if (input[i] == '_') {
      s += '_';
    } else if (!islower(input[i])) {
      // Prevent duplicate underscores for Upper_Snake_Case strings
      // and UPPERCASE strings.
      if (islower(input[i - 1]) ||
          (isdigit(input[i - 1]) && !isdigit(input[i]))) {
        s += '_';
      }
      s += CharToLower(input[i]);
    } else {
      s += input[i];
    }
  }
  return s;
}

std::string DasherToSnake(const std::string &input) {
  std::string s;
  for (size_t i = 0; i < input.length(); i++) {
    if (input[i] == '-') {
      s += "_";
    } else {
      s += input[i];
    }
  }
  return s;
}

std::string ToDasher(const std::string &input) {
  std::string s;
  char p = 0;
  for (size_t i = 0; i < input.length(); i++) {
    char const &c = input[i];
    if (c == '_') {
      if (i > 0 && p != kPathSeparator &&
          // The following is a special case to ignore digits after a _. This is
          // because ThisExample3 would be converted to this_example_3 in the
          // CamelToSnake conversion, and then dasher would do this-example-3,
          // but it expects this-example3.
          !(i + 1 < input.length() && isdigit(input[i + 1])))
        s += "-";
    } else {
      s += c;
    }
    p = c;
  }
  return s;
}

// Converts foo_bar_123baz_456 to foo_bar123_baz456
std::string SnakeToSnake2(const std::string &s) {
  if (s.length() <= 1) return s;
  std::string result;
  result.reserve(s.size());
  for (size_t i = 0; i < s.length() - 1; i++) {
    if (s[i] == '_' && isdigit(s[i + 1])) {
      continue;  // Move the `_` until after the digits.
    }

    result.push_back(s[i]);

    if (isdigit(s[i]) && isalpha(s[i + 1]) && islower(s[i + 1])) {
      result.push_back('_');
    }
  }
  result.push_back(s.back());

  return result;
}

}  // namespace

bool LoadFile(const char *name, bool binary, std::string *buf) {
  FLATBUFFERS_ASSERT(g_load_file_function);
  return g_load_file_function(name, binary, buf);
}

bool FileExists(const char *name) {
  FLATBUFFERS_ASSERT(g_file_exists_function);
  return g_file_exists_function(name);
}

bool DirExists(const char *name) {
  // clang-format off

  #ifdef _WIN32
    #define flatbuffers_stat _stat
    #define FLATBUFFERS_S_IFDIR _S_IFDIR
  #else
    #define flatbuffers_stat stat
    #define FLATBUFFERS_S_IFDIR S_IFDIR
  #endif
  // clang-format on
  struct flatbuffers_stat file_info;
  if (flatbuffers_stat(name, &file_info) != 0) return false;
  return (file_info.st_mode & FLATBUFFERS_S_IFDIR) != 0;
}

LoadFileFunction SetLoadFileFunction(LoadFileFunction load_file_function) {
  LoadFileFunction previous_function = g_load_file_function;
  g_load_file_function = load_file_function ? load_file_function : LoadFileRaw;
  return previous_function;
}

FileExistsFunction SetFileExistsFunction(
    FileExistsFunction file_exists_function) {
  FileExistsFunction previous_function = g_file_exists_function;
  g_file_exists_function =
      file_exists_function ? file_exists_function : FileExistsRaw;
  return previous_function;
}

bool SaveFile(const char *name, const char *buf, size_t len, bool binary) {
  std::ofstream ofs(name, binary ? std::ofstream::binary : std::ofstream::out);
  if (!ofs.is_open()) return false;
  ofs.write(buf, len);
  return !ofs.bad();
}

// We internally store paths in posix format ('/'). Paths supplied
// by the user should go through PosixPath to ensure correct behavior
// on Windows when paths are string-compared.

static const char kPathSeparatorWindows = '\\';
static const char *PathSeparatorSet = "\\/";  // Intentionally no ':'

std::string StripExtension(const std::string &filepath) {
  size_t i = filepath.find_last_of('.');
  return i != std::string::npos ? filepath.substr(0, i) : filepath;
}

std::string GetExtension(const std::string &filepath) {
  size_t i = filepath.find_last_of('.');
  return i != std::string::npos ? filepath.substr(i + 1) : "";
}

std::string StripPath(const std::string &filepath) {
  size_t i = filepath.find_last_of(PathSeparatorSet);
  return i != std::string::npos ? filepath.substr(i + 1) : filepath;
}

std::string StripFileName(const std::string &filepath) {
  size_t i = filepath.find_last_of(PathSeparatorSet);
  return i != std::string::npos ? filepath.substr(0, i) : "";
}

std::string StripPrefix(const std::string &filepath,
                        const std::string &prefix_to_remove) {
  if (!strncmp(filepath.c_str(), prefix_to_remove.c_str(),
               prefix_to_remove.size())) {
    return filepath.substr(prefix_to_remove.size());
  }
  return filepath;
}

std::string ConCatPathFileName(const std::string &path,
                               const std::string &filename) {
  std::string filepath = path;
  if (filepath.length()) {
    char &filepath_last_character = filepath.back();
    if (filepath_last_character == kPathSeparatorWindows) {
      filepath_last_character = kPathSeparator;
    } else if (filepath_last_character != kPathSeparator) {
      filepath += kPathSeparator;
    }
  }
  filepath += filename;
  // Ignore './' at the start of filepath.
  if (filepath[0] == '.' && filepath[1] == kPathSeparator) {
    filepath.erase(0, 2);
  }
  return filepath;
}

std::string PosixPath(const char *path) {
  std::string p = path;
  std::replace(p.begin(), p.end(), '\\', '/');
  return p;
}
std::string PosixPath(const std::string &path) {
  return PosixPath(path.c_str());
}

void EnsureDirExists(const std::string &filepath) {
  auto parent = StripFileName(filepath);
  if (parent.length()) EnsureDirExists(parent);
    // clang-format off

  #ifdef _WIN32
    (void)_mkdir(filepath.c_str());
  #else
    mkdir(filepath.c_str(), S_IRWXU|S_IRGRP|S_IXGRP);
  #endif
  // clang-format on
}

std::string FilePath(const std::string& project, const std::string& filePath, bool absolute) {
    return (absolute) ? AbsolutePath(filePath) : RelativeToRootPath(project, filePath);
}

std::string AbsolutePath(const std::string &filepath) {
  // clang-format off

  #ifdef FLATBUFFERS_NO_ABSOLUTE_PATH_RESOLUTION
    return filepath;
  #else
    #if defined(_WIN32) || defined(__MINGW32__) || defined(__MINGW64__) || defined(__CYGWIN__)
      char abs_path[MAX_PATH];
      return GetFullPathNameA(filepath.c_str(), MAX_PATH, abs_path, nullptr)
    #else
      char *abs_path_temp = realpath(filepath.c_str(), nullptr);
      bool success = abs_path_temp != nullptr;
      std::string abs_path;
      if(success) {
        abs_path = abs_path_temp;
        free(abs_path_temp);
      }
      return success
    #endif
      ? abs_path
      : filepath;
  #endif // FLATBUFFERS_NO_ABSOLUTE_PATH_RESOLUTION
  // clang-format on
}

std::string RelativeToRootPath(const std::string &project,
                               const std::string &filepath) {
  std::string absolute_project = PosixPath(AbsolutePath(project));
  if (absolute_project.back() != '/') absolute_project += "/";
  std::string absolute_filepath = PosixPath(AbsolutePath(filepath));

  // Find the first character where they disagree.
  // The previous directory is the lowest common ancestor;
  const char *a = absolute_project.c_str();
  const char *b = absolute_filepath.c_str();
  size_t common_prefix_len = 0;
  while (*a != '\0' && *b != '\0' && *a == *b) {
    if (*a == '/') common_prefix_len = a - absolute_project.c_str();
    a++;
    b++;
  }
  // the number of ../ to prepend to b depends on the number of remaining
  // directories in A.
  const char *suffix = absolute_project.c_str() + common_prefix_len;
  size_t num_up = 0;
  while (*suffix != '\0')
    if (*suffix++ == '/') num_up++;
  num_up--;  // last one is known to be '/'.
  std::string result = "//";
  for (size_t i = 0; i < num_up; i++) result += "../";
  result += absolute_filepath.substr(common_prefix_len + 1);

  return result;
}

// Locale-independent code.
#if defined(FLATBUFFERS_LOCALE_INDEPENDENT) && \
    (FLATBUFFERS_LOCALE_INDEPENDENT > 0)

// clang-format off
// Allocate locale instance at startup of application.
ClassicLocale ClassicLocale::instance_;

#ifdef _MSC_VER
  ClassicLocale::ClassicLocale()
    : locale_(_create_locale(LC_ALL, "C")) {}
  ClassicLocale::~ClassicLocale() { _free_locale(locale_); }
#else
  ClassicLocale::ClassicLocale()
    : locale_(newlocale(LC_ALL, "C", nullptr)) {}
  ClassicLocale::~ClassicLocale() { freelocale(locale_); }
#endif
// clang-format on

#endif  // !FLATBUFFERS_LOCALE_INDEPENDENT

std::string RemoveStringQuotes(const std::string &s) {
  auto ch = *s.c_str();
  return ((s.size() >= 2) && (ch == '\"' || ch == '\'') && (ch == s.back()))
             ? s.substr(1, s.length() - 2)
             : s;
}

bool SetGlobalTestLocale(const char *locale_name, std::string *_value) {
  const auto the_locale = setlocale(LC_ALL, locale_name);
  if (!the_locale) return false;
  if (_value) *_value = std::string(the_locale);
  return true;
}

bool ReadEnvironmentVariable(const char *var_name, std::string *_value) {
#ifdef _MSC_VER
  __pragma(warning(disable : 4996));  // _CRT_SECURE_NO_WARNINGS
#endif
  auto env_str = std::getenv(var_name);
  if (!env_str) return false;
  if (_value) *_value = std::string(env_str);
  return true;
}

std::string ConvertCase(const std::string &input, Case output_case,
                        Case input_case) {
  if (output_case == Case::kKeep) return input;
  // The output cases expect snake_case inputs, so if we don't have that input
  // format, try to convert to snake_case.
  switch (input_case) {
    case Case::kLowerCamel:
    case Case::kUpperCamel:
      return ConvertCase(CamelToSnake(input), output_case);
    case Case::kDasher: return ConvertCase(DasherToSnake(input), output_case);
    case Case::kKeep: printf("WARNING: Converting from kKeep case.\n"); break;
    default:
    case Case::kSnake:
    case Case::kScreamingSnake:
    case Case::kAllLower:
    case Case::kAllUpper: break;
  }

  switch (output_case) {
    case Case::kUpperCamel: return ToCamelCase(input, true);
    case Case::kLowerCamel: return ToCamelCase(input, false);
    case Case::kSnake: return input;
    case Case::kScreamingSnake: return ToSnakeCase(input, true);
    case Case::kAllUpper: return ToAll(input, CharToUpper);
    case Case::kAllLower: return ToAll(input, CharToLower);
    case Case::kDasher: return ToDasher(input);
    case Case::kSnake2: return SnakeToSnake2(input);
    default:
    case Case::kUnknown: return input;
  }
}

}  // namespace flatbuffers
