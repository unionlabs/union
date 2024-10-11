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

#include "idl_gen_cpp.h"

#include <limits>
#include <memory>
#include <string>
#include <unordered_set>
#include <utility>

#include "flatbuffers/base.h"
#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/flatc.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"

namespace flatbuffers {

// Make numerical literal with type-suffix.
// This function is only needed for C++! Other languages do not need it.
static inline std::string NumToStringCpp(std::string val, BaseType type) {
  // Avoid issues with -2147483648, -9223372036854775808.
  switch (type) {
    case BASE_TYPE_INT:
      return (val != "-2147483648") ? val : ("(-2147483647 - 1)");
    case BASE_TYPE_ULONG: return (val == "0") ? val : (val + "ULL");
    case BASE_TYPE_LONG:
      if (val == "-9223372036854775808")
        return "(-9223372036854775807LL - 1LL)";
      else
        return (val == "0") ? val : (val + "LL");
    default: return val;
  }
}

static std::string GenIncludeGuard(const std::string &file_name,
                                   const Namespace &name_space,
                                   const std::string &postfix = "") {
  // Generate include guard.
  std::string guard = file_name;
  // Remove any non-alpha-numeric characters that may appear in a filename.
  struct IsAlnum {
    bool operator()(char c) const { return !is_alnum(c); }
  };
  guard.erase(std::remove_if(guard.begin(), guard.end(), IsAlnum()),
              guard.end());
  guard = "FLATBUFFERS_GENERATED_" + guard;
  guard += "_";
  // For further uniqueness, also add the namespace.
  for (const std::string &component : name_space.components) {
    guard += component + "_";
  }
  // Anything extra to add to the guard?
  if (!postfix.empty()) { guard += postfix + "_"; }
  guard += "H_";
  std::transform(guard.begin(), guard.end(), guard.begin(), CharToUpper);
  return guard;
}

static bool IsVectorOfPointers(const FieldDef &field) {
  const auto &type = field.value.type;
  const auto &vector_type = type.VectorType();
  return IsVector(type) && vector_type.base_type == BASE_TYPE_STRUCT &&
         !vector_type.struct_def->fixed && !field.native_inline;
}

static bool IsPointer(const FieldDef &field) {
  return field.value.type.base_type == BASE_TYPE_STRUCT &&
         !IsStruct(field.value.type);
}

namespace cpp {

enum CppStandard { CPP_STD_X0 = 0, CPP_STD_11, CPP_STD_17 };

// Define a style of 'struct' constructor if it has 'Array' fields.
enum GenArrayArgMode {
  kArrayArgModeNone,        // don't generate initialization args
  kArrayArgModeSpanStatic,  // generate ::flatbuffers::span<T,N>
};

// Extension of IDLOptions for cpp-generator.
struct IDLOptionsCpp : public IDLOptions {
  // All fields start with 'g_' prefix to distinguish from the base IDLOptions.
  CppStandard g_cpp_std;    // Base version of C++ standard.
  bool g_only_fixed_enums;  // Generate underlaying type for all enums.

  IDLOptionsCpp(const IDLOptions &opts)
      : IDLOptions(opts), g_cpp_std(CPP_STD_11), g_only_fixed_enums(true) {}
};

// Iterates over all the fields of the object first by Offset type (Offset64
// before Offset32) and then by definition order.
static void ForAllFieldsOrderedByOffset(
    const StructDef &object, std::function<void(const FieldDef *field)> func) {
  // Loop over all the fields and call the func on all offset64 fields.
  for (const FieldDef *field_def : object.fields.vec) {
    if (field_def->offset64) { func(field_def); }
  }
  // Loop over all the fields a second time and call the func on all offset
  // fields.
  for (const FieldDef *field_def : object.fields.vec) {
    if (!field_def->offset64) { func(field_def); }
  }
}

class CppGenerator : public BaseGenerator {
 public:
  CppGenerator(const Parser &parser, const std::string &path,
               const std::string &file_name, IDLOptionsCpp opts)
      : BaseGenerator(parser, path, file_name, "", "::", "h"),
        cur_name_space_(nullptr),
        opts_(opts),
        float_const_gen_("std::numeric_limits<double>::",
                         "std::numeric_limits<float>::", "quiet_NaN()",
                         "infinity()") {
    static const char *const keywords[] = {
      "alignas",
      "alignof",
      "and",
      "and_eq",
      "asm",
      "atomic_cancel",
      "atomic_commit",
      "atomic_noexcept",
      "auto",
      "bitand",
      "bitor",
      "bool",
      "break",
      "case",
      "catch",
      "char",
      "char16_t",
      "char32_t",
      "class",
      "compl",
      "concept",
      "const",
      "constexpr",
      "const_cast",
      "continue",
      "co_await",
      "co_return",
      "co_yield",
      "decltype",
      "default",
      "delete",
      "do",
      "double",
      "dynamic_cast",
      "else",
      "enum",
      "explicit",
      "export",
      "extern",
      "false",
      "float",
      "for",
      "friend",
      "goto",
      "if",
      "import",
      "inline",
      "int",
      "long",
      "module",
      "mutable",
      "namespace",
      "new",
      "noexcept",
      "not",
      "not_eq",
      "nullptr",
      "operator",
      "or",
      "or_eq",
      "private",
      "protected",
      "public",
      "register",
      "reinterpret_cast",
      "requires",
      "return",
      "short",
      "signed",
      "sizeof",
      "static",
      "static_assert",
      "static_cast",
      "struct",
      "switch",
      "synchronized",
      "template",
      "this",
      "thread_local",
      "throw",
      "true",
      "try",
      "typedef",
      "typeid",
      "typename",
      "union",
      "unsigned",
      "using",
      "virtual",
      "void",
      "volatile",
      "wchar_t",
      "while",
      "xor",
      "xor_eq",
      nullptr,
    };
    for (auto kw = keywords; *kw; kw++) keywords_.insert(*kw);
  }

  // Adds code to check that the included flatbuffers.h is of the same version
  // as the generated code. This check currently looks for exact version match,
  // as we would guarantee that they are compatible, but in theory a newer
  // version of flatbuffers.h should work with a old code gen if we do proper
  // backwards support.
  void GenFlatbuffersVersionCheck() {
    code_ +=
        "// Ensure the included flatbuffers.h is the same version as when this "
        "file was";
    code_ += "// generated, otherwise it may not be compatible.";
    code_ += "static_assert(FLATBUFFERS_VERSION_MAJOR == " +
             std::to_string(FLATBUFFERS_VERSION_MAJOR) + " &&";
    code_ += "              FLATBUFFERS_VERSION_MINOR == " +
             std::to_string(FLATBUFFERS_VERSION_MINOR) + " &&";
    code_ += "              FLATBUFFERS_VERSION_REVISION == " +
             std::to_string(FLATBUFFERS_VERSION_REVISION) + ",";
    code_ += "             \"Non-compatible flatbuffers version included\");";
  }

  void GenIncludeDependencies() {
    if (opts_.generate_object_based_api) {
      for (const std::string &native_included_file :
           parser_.native_included_files_) {
        code_ += "#include \"" + native_included_file + "\"";
      }
    }

    // Get the directly included file of the file being parsed.
    std::vector<IncludedFile> included_files(parser_.GetIncludedFiles());

    // We are safe to sort them alphabetically, since there shouldn't be any
    // interdependence between them.
    std::stable_sort(included_files.begin(), included_files.end());

    for (const IncludedFile &included_file : included_files) {
      // Get the name of the included file as defined by the schema, and strip
      // the .fbs extension.
      const std::string name_without_ext =
          StripExtension(included_file.schema_name);

      // If we are told to keep the prefix of the included schema, leave it
      // unchanged, otherwise strip the leading path off so just the "basename"
      // of the include is retained.
      const std::string basename =
          opts_.keep_prefix ? name_without_ext : StripPath(name_without_ext);

      code_ += "#include \"" +
               GeneratedFileName(opts_.include_prefix, basename, opts_) + "\"";
    }

    if (!parser_.native_included_files_.empty() || !included_files.empty()) {
      code_ += "";
    }
  }

  void MarkIf64BitBuilderIsNeeded() {
    if (needs_64_bit_builder_) { return; }
    for (auto t : parser_.structs_.vec) {
      if (t == nullptr) continue;
      for (auto f : t->fields.vec) {
        if (f == nullptr) continue;
        if (f->offset64) {
          needs_64_bit_builder_ = true;
          break;
        }
      }
    }
  }

  std::string GetBuilder() {
    return std::string("::flatbuffers::FlatBufferBuilder") +
           (needs_64_bit_builder_ ? "64" : "");
  }

  void GenExtraIncludes() {
    for (const std::string &cpp_include : opts_.cpp_includes) {
      code_ += "#include \"" + cpp_include + "\"";
    }
    if (!opts_.cpp_includes.empty()) { code_ += ""; }
  }

  void GenEmbeddedIncludes() {
    if (parser_.opts.binary_schema_gen_embed && parser_.root_struct_def_) {
      const std::string file_path =
          GeneratedFileName(opts_.include_prefix, file_name_ + "_bfbs", opts_);
      code_ += "// For access to the binary schema that produced this file.";
      code_ += "#include \"" + file_path + "\"";
      code_ += "";
    }
  }

  std::string EscapeKeyword(const std::string &name) const {
    return keywords_.find(name) == keywords_.end() ? name : name + "_";
  }

  std::string Name(const FieldDef &field) const {
    // the union type field suffix is immutable.
    static size_t union_suffix_len = strlen(UnionTypeFieldSuffix());
    const bool is_union_type = field.value.type.base_type == BASE_TYPE_UTYPE;
    // early return if no case transformation required
    if (opts_.cpp_object_api_field_case_style ==
        IDLOptions::CaseStyle_Unchanged)
      return EscapeKeyword(field.name);
    std::string name = field.name;
    // do not change the case style of the union type field suffix
    if (is_union_type) {
      FLATBUFFERS_ASSERT(name.length() > union_suffix_len);
      name.erase(name.length() - union_suffix_len, union_suffix_len);
    }
    if (opts_.cpp_object_api_field_case_style == IDLOptions::CaseStyle_Upper)
      name = ConvertCase(name, Case::kUpperCamel);
    else if (opts_.cpp_object_api_field_case_style ==
             IDLOptions::CaseStyle_Lower)
      name = ConvertCase(name, Case::kLowerCamel);
    // restore the union field type suffix
    if (is_union_type) name.append(UnionTypeFieldSuffix(), union_suffix_len);
    return EscapeKeyword(name);
  }

  std::string Name(const Definition &def) const {
    return EscapeKeyword(def.name);
  }

  std::string Name(const EnumVal &ev) const { return EscapeKeyword(ev.name); }

  bool generate_bfbs_embed() {
    code_.Clear();
    code_ += "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n";

    // If we don't have a root struct definition,
    if (!parser_.root_struct_def_) {
      // put a comment in the output why there is no code generated.
      code_ += "// Binary schema not generated, no root struct found";
    } else {
      auto &struct_def = *parser_.root_struct_def_;
      const auto include_guard =
          GenIncludeGuard(file_name_, *struct_def.defined_namespace, "bfbs");

      code_ += "#ifndef " + include_guard;
      code_ += "#define " + include_guard;
      code_ += "";
      if (parser_.opts.gen_nullable) {
        code_ += "#pragma clang system_header\n\n";
      }

      code_ += "#include <cstddef>";
      code_ += "#include <cstdint>";

      SetNameSpace(struct_def.defined_namespace);
      auto name = Name(struct_def);
      code_.SetValue("STRUCT_NAME", name);

      // Create code to return the binary schema data.
      auto binary_schema_hex_text =
          BufferToHexText(parser_.builder_.GetBufferPointer(),
                          parser_.builder_.GetSize(), 105, "      ", "");

      code_ += "struct {{STRUCT_NAME}}BinarySchema {";
      code_ += "  static const uint8_t *data() {";
      code_ += "    // Buffer containing the binary schema.";
      code_ += "    static const uint8_t bfbsData[" +
               NumToString(parser_.builder_.GetSize()) + "] = {";
      code_ += binary_schema_hex_text;
      code_ += "    };";
      code_ += "    return bfbsData;";
      code_ += "  }";
      code_ += "  static size_t size() {";
      code_ += "    return " + NumToString(parser_.builder_.GetSize()) + ";";
      code_ += "  }";
      code_ += "  const uint8_t *begin() {";
      code_ += "    return data();";
      code_ += "  }";
      code_ += "  const uint8_t *end() {";
      code_ += "    return data() + size();";
      code_ += "  }";
      code_ += "};";
      code_ += "";

      if (cur_name_space_) SetNameSpace(nullptr);

      // Close the include guard.
      code_ += "#endif  // " + include_guard;
    }

    // We are just adding "_bfbs" to the generated filename.
    const auto file_path =
        GeneratedFileName(path_, file_name_ + "_bfbs", opts_);
    const auto final_code = code_.ToString();

    return SaveFile(file_path.c_str(), final_code, false);
  }

  // Iterate through all definitions we haven't generate code for (enums,
  // structs, and tables) and output them to a single file.
  bool generate() {
    // Check if we require a 64-bit flatbuffer builder.
    MarkIf64BitBuilderIsNeeded();

    code_.Clear();
    code_ += "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n";

    const auto include_guard =
        GenIncludeGuard(file_name_, *parser_.current_namespace_);
    code_ += "#ifndef " + include_guard;
    code_ += "#define " + include_guard;
    code_ += "";

    if (opts_.gen_nullable) { code_ += "#pragma clang system_header\n\n"; }

    code_ += "#include \"flatbuffers/flatbuffers.h\"";
    if (parser_.uses_flexbuffers_) {
      code_ += "#include \"flatbuffers/flexbuffers.h\"";
      code_ += "#include \"flatbuffers/flex_flat_util.h\"";
    }
    code_ += "";
    GenFlatbuffersVersionCheck();
    code_ += "";

    if (opts_.include_dependence_headers) { GenIncludeDependencies(); }
    GenExtraIncludes();
    GenEmbeddedIncludes();

    FLATBUFFERS_ASSERT(!cur_name_space_);

    // Generate forward declarations for all structs/tables, since they may
    // have circular references.
    for (const auto &struct_def : parser_.structs_.vec) {
      if (!struct_def->generated) {
        SetNameSpace(struct_def->defined_namespace);
        code_ += "struct " + Name(*struct_def) + ";";
        if (!struct_def->fixed) {
          code_ += "struct " + Name(*struct_def) + "Builder;";
        }
        if (opts_.generate_object_based_api) {
          auto nativeName = NativeName(Name(*struct_def), struct_def, opts_);
          if (!struct_def->fixed) { code_ += "struct " + nativeName + ";"; }
        }
        code_ += "";
      }
    }

    // Generate forward declarations for all equal operators
    if (opts_.generate_object_based_api && opts_.gen_compare) {
      for (const auto &struct_def : parser_.structs_.vec) {
        if (!struct_def->generated) {
          SetNameSpace(struct_def->defined_namespace);
          auto nativeName = NativeName(Name(*struct_def), struct_def, opts_);
          code_ += "bool operator==(const " + nativeName + " &lhs, const " +
                   nativeName + " &rhs);";
          code_ += "bool operator!=(const " + nativeName + " &lhs, const " +
                   nativeName + " &rhs);";
        }
      }
      code_ += "";
    }

    // Generate preablmle code for mini reflection.
    if (opts_.mini_reflect != IDLOptions::kNone) {
      // To break cyclic dependencies, first pre-declare all tables/structs.
      for (const auto &struct_def : parser_.structs_.vec) {
        if (!struct_def->generated) {
          SetNameSpace(struct_def->defined_namespace);
          GenMiniReflectPre(struct_def);
        }
      }
    }

    // Generate code for all the enum declarations.
    for (const auto &enum_def : parser_.enums_.vec) {
      if (!enum_def->generated) {
        SetNameSpace(enum_def->defined_namespace);
        GenEnum(*enum_def);
      }
    }

    // Generate code for all structs, then all tables.
    for (const auto &struct_def : parser_.structs_.vec) {
      if (struct_def->fixed && !struct_def->generated) {
        SetNameSpace(struct_def->defined_namespace);
        GenStruct(*struct_def);
      }
    }
    for (const auto &struct_def : parser_.structs_.vec) {
      if (!struct_def->fixed && !struct_def->generated) {
        SetNameSpace(struct_def->defined_namespace);
        GenTable(*struct_def);
      }
    }
    for (const auto &struct_def : parser_.structs_.vec) {
      if (!struct_def->fixed && !struct_def->generated) {
        SetNameSpace(struct_def->defined_namespace);
        GenTablePost(*struct_def);
      }
    }

    // Generate code for union verifiers.
    for (const auto &enum_def : parser_.enums_.vec) {
      if (enum_def->is_union && !enum_def->generated) {
        SetNameSpace(enum_def->defined_namespace);
        GenUnionPost(*enum_def);
      }
    }

    // Generate code for mini reflection.
    if (opts_.mini_reflect != IDLOptions::kNone) {
      // Then the unions/enums that may refer to them.
      for (const auto &enum_def : parser_.enums_.vec) {
        if (!enum_def->generated) {
          SetNameSpace(enum_def->defined_namespace);
          GenMiniReflect(nullptr, enum_def);
        }
      }
      // Then the full tables/structs.
      for (const auto &struct_def : parser_.structs_.vec) {
        if (!struct_def->generated) {
          SetNameSpace(struct_def->defined_namespace);
          GenMiniReflect(struct_def, nullptr);
        }
      }
    }

    // Generate convenient global helper functions:
    if (parser_.root_struct_def_) {
      auto &struct_def = *parser_.root_struct_def_;
      SetNameSpace(struct_def.defined_namespace);
      auto name = Name(struct_def);
      auto qualified_name = cur_name_space_->GetFullyQualifiedName(name);
      auto cpp_name = TranslateNameSpace(qualified_name);

      code_.SetValue("STRUCT_NAME", name);
      code_.SetValue("CPP_NAME", cpp_name);
      code_.SetValue("NULLABLE_EXT", NullableExtension());
      code_.SetValue(
          "SIZE_T", needs_64_bit_builder_ ? ",::flatbuffers::uoffset64_t" : "");

      // The root datatype accessor:
      code_ += "inline \\";
      code_ +=
          "const {{CPP_NAME}} *{{NULLABLE_EXT}}Get{{STRUCT_NAME}}(const void "
          "*buf) {";
      code_ += "  return ::flatbuffers::GetRoot<{{CPP_NAME}}>(buf);";
      code_ += "}";
      code_ += "";

      code_ += "inline \\";
      code_ +=
          "const {{CPP_NAME}} "
          "*{{NULLABLE_EXT}}GetSizePrefixed{{STRUCT_NAME}}(const void "
          "*buf) {";
      code_ +=
          "  return "
          "::flatbuffers::GetSizePrefixedRoot<{{CPP_NAME}}{{SIZE_T}}>(buf);";
      code_ += "}";
      code_ += "";

      if (opts_.mutable_buffer) {
        code_ += "inline \\";
        code_ += "{{STRUCT_NAME}} *GetMutable{{STRUCT_NAME}}(void *buf) {";
        code_ +=
            "  return ::flatbuffers::GetMutableRoot<{{STRUCT_NAME}}>(buf);";
        code_ += "}";
        code_ += "";

        code_ += "inline \\";
        code_ +=
            "{{CPP_NAME}} "
            "*{{NULLABLE_EXT}}GetMutableSizePrefixed{{STRUCT_NAME}}(void "
            "*buf) {";
        code_ +=
            "  return "
            "::flatbuffers::GetMutableSizePrefixedRoot<{{CPP_NAME}}{{SIZE_T}}>("
            "buf);";
        code_ += "}";
        code_ += "";
      }

      if (parser_.file_identifier_.length()) {
        // Return the identifier
        code_ += "inline const char *{{STRUCT_NAME}}Identifier() {";
        code_ += "  return \"" + parser_.file_identifier_ + "\";";
        code_ += "}";
        code_ += "";

        // Check if a buffer has the identifier.
        code_ += "inline \\";
        code_ += "bool {{STRUCT_NAME}}BufferHasIdentifier(const void *buf) {";
        code_ += "  return ::flatbuffers::BufferHasIdentifier(";
        code_ += "      buf, {{STRUCT_NAME}}Identifier());";
        code_ += "}";
        code_ += "";

        // Check if a size-prefixed buffer has the identifier.
        code_ += "inline \\";
        code_ +=
            "bool SizePrefixed{{STRUCT_NAME}}BufferHasIdentifier(const void "
            "*buf) {";
        code_ += "  return ::flatbuffers::BufferHasIdentifier(";
        code_ += "      buf, {{STRUCT_NAME}}Identifier(), true);";
        code_ += "}";
        code_ += "";
      }

      // The root verifier.
      if (parser_.file_identifier_.length()) {
        code_.SetValue("ID", name + "Identifier()");
      } else {
        code_.SetValue("ID", "nullptr");
      }

      code_ += "inline bool Verify{{STRUCT_NAME}}Buffer(";
      code_ += "    ::flatbuffers::Verifier &verifier) {";
      code_ += "  return verifier.VerifyBuffer<{{CPP_NAME}}>({{ID}});";
      code_ += "}";
      code_ += "";

      code_ += "inline bool VerifySizePrefixed{{STRUCT_NAME}}Buffer(";
      code_ += "    ::flatbuffers::Verifier &verifier) {";
      code_ +=
          "  return "
          "verifier.VerifySizePrefixedBuffer<{{CPP_NAME}}{{SIZE_T}}>({{ID}});";
      code_ += "}";
      code_ += "";

      if (parser_.file_extension_.length()) {
        // Return the extension
        code_ += "inline const char *{{STRUCT_NAME}}Extension() {";
        code_ += "  return \"" + parser_.file_extension_ + "\";";
        code_ += "}";
        code_ += "";
      }

      // Finish a buffer with a given root object:
      code_ += "inline void Finish{{STRUCT_NAME}}Buffer(";
      code_ += "    " + GetBuilder() + " &fbb,";
      code_ += "    ::flatbuffers::Offset<{{CPP_NAME}}> root) {";
      if (parser_.file_identifier_.length())
        code_ += "  fbb.Finish(root, {{STRUCT_NAME}}Identifier());";
      else
        code_ += "  fbb.Finish(root);";
      code_ += "}";
      code_ += "";

      code_ += "inline void FinishSizePrefixed{{STRUCT_NAME}}Buffer(";
      code_ += "    " + GetBuilder() + " &fbb,";
      code_ += "    ::flatbuffers::Offset<{{CPP_NAME}}> root) {";
      if (parser_.file_identifier_.length())
        code_ += "  fbb.FinishSizePrefixed(root, {{STRUCT_NAME}}Identifier());";
      else
        code_ += "  fbb.FinishSizePrefixed(root);";
      code_ += "}";
      code_ += "";

      if (opts_.generate_object_based_api) {
        // A convenient root unpack function.
        auto native_name = WrapNativeNameInNameSpace(struct_def, opts_);
        code_.SetValue("UNPACK_RETURN",
                       GenTypeNativePtr(native_name, nullptr, false));
        code_.SetValue("UNPACK_TYPE",
                       GenTypeNativePtr(native_name, nullptr, true));

        code_ += "inline {{UNPACK_RETURN}} UnPack{{STRUCT_NAME}}(";
        code_ += "    const void *buf,";
        code_ +=
            "    const ::flatbuffers::resolver_function_t *res = nullptr) {";
        code_ += "  return {{UNPACK_TYPE}}\\";
        code_ += "(Get{{STRUCT_NAME}}(buf)->UnPack(res));";
        code_ += "}";
        code_ += "";

        code_ += "inline {{UNPACK_RETURN}} UnPackSizePrefixed{{STRUCT_NAME}}(";
        code_ += "    const void *buf,";
        code_ +=
            "    const ::flatbuffers::resolver_function_t *res = nullptr) {";
        code_ += "  return {{UNPACK_TYPE}}\\";
        code_ += "(GetSizePrefixed{{STRUCT_NAME}}(buf)->UnPack(res));";
        code_ += "}";
        code_ += "";
      }
    }

    if (cur_name_space_) SetNameSpace(nullptr);

    // Close the include guard.
    code_ += "#endif  // " + include_guard;

    const auto file_path = GeneratedFileName(path_, file_name_, opts_);
    const auto final_code = code_.ToString();

    // Save the file and optionally generate the binary schema code.
    return SaveFile(file_path.c_str(), final_code, false) &&
           (!parser_.opts.binary_schema_gen_embed || generate_bfbs_embed());
  }

 private:
  CodeWriter code_;

  std::unordered_set<std::string> keywords_;

  // This tracks the current namespace so we can insert namespace declarations.
  const Namespace *cur_name_space_;

  const IDLOptionsCpp opts_;
  const TypedFloatConstantGenerator float_const_gen_;
  bool needs_64_bit_builder_ = false;

  const Namespace *CurrentNameSpace() const { return cur_name_space_; }

  // Translates a qualified name in flatbuffer text format to the same name in
  // the equivalent C++ namespace.
  static std::string TranslateNameSpace(const std::string &qualified_name) {
    std::string cpp_qualified_name = qualified_name;
    size_t start_pos = 0;
    while ((start_pos = cpp_qualified_name.find('.', start_pos)) !=
           std::string::npos) {
      cpp_qualified_name.replace(start_pos, 1, "::");
    }
    return cpp_qualified_name;
  }

  bool TypeHasKey(const Type &type) {
    if (type.base_type != BASE_TYPE_STRUCT) { return false; }
    for (auto &field : type.struct_def->fields.vec) {
      if (field->key) { return true; }
    }
    return false;
  }

  bool VectorElementUserFacing(const Type &type) const {
    return (opts_.scoped_enums && IsEnum(type)) ||
           (opts_.g_cpp_std >= cpp::CPP_STD_17 && opts_.g_only_fixed_enums &&
            IsEnum(type));
  }

  void GenComment(const std::vector<std::string> &dc, const char *prefix = "") {
    std::string text;
    ::flatbuffers::GenComment(dc, &text, nullptr, prefix);
    code_ += text + "\\";
  }

  // Return a C++ type from the table in idl.h
  std::string GenTypeBasic(const Type &type, bool user_facing_type) const {
    if (user_facing_type) {
      if (type.enum_def) return WrapInNameSpace(*type.enum_def);
      if (type.base_type == BASE_TYPE_BOOL) return "bool";
    }
    // Get real underlying type for union type
    auto base_type = type.base_type;
    if (type.base_type == BASE_TYPE_UTYPE && type.enum_def != nullptr) {
      base_type = type.enum_def->underlying_type.base_type;
    }
    return StringOf(base_type);
  }

  // Return a C++ pointer type, specialized to the actual struct/table types,
  // and vector element types.
  std::string GenTypePointer(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: {
        return "::flatbuffers::String";
      }
      case BASE_TYPE_VECTOR64:
      case BASE_TYPE_VECTOR: {
        const auto type_name = GenTypeWire(
            type.VectorType(), "", VectorElementUserFacing(type.VectorType()));
        return "::flatbuffers::Vector" +
               std::string((type.base_type == BASE_TYPE_VECTOR64) ? "64<"
                                                                  : "<") +
               type_name + ">";
      }
      case BASE_TYPE_STRUCT: {
        return WrapInNameSpace(*type.struct_def);
      }
      case BASE_TYPE_UNION:
        // fall through
      default: {
        return "void";
      }
    }
  }

  // Return a C++ type for any type (scalar/pointer) specifically for
  // building a flatbuffer.
  std::string GenTypeWire(const Type &type, const char *postfix,
                          bool user_facing_type,
                          bool _64_bit_offset = false) const {
    if (IsScalar(type.base_type)) {
      return GenTypeBasic(type, user_facing_type) + postfix;
    } else if (IsStruct(type)) {
      return "const " + GenTypePointer(type) + " *";
    } else {
      return "::flatbuffers::Offset" + std::string(_64_bit_offset ? "64" : "") +
             "<" + GenTypePointer(type) + ">" + postfix;
    }
  }

  // Return a C++ type for any type (scalar/pointer) that reflects its
  // serialized size.
  std::string GenTypeSize(const Type &type) const {
    if (IsScalar(type.base_type)) {
      return GenTypeBasic(type, false);
    } else if (IsStruct(type)) {
      return GenTypePointer(type);
    } else {
      return "::flatbuffers::uoffset_t";
    }
  }

  std::string NullableExtension() {
    return opts_.gen_nullable ? " _Nullable " : "";
  }

  static std::string NativeName(const std::string &name, const StructDef *sd,
                                const IDLOptions &opts) {
    return sd && !sd->fixed ? opts.object_prefix + name + opts.object_suffix
                            : name;
  }

  std::string WrapNativeNameInNameSpace(const StructDef &struct_def,
                                        const IDLOptions &opts) {
    return WrapInNameSpace(struct_def.defined_namespace,
                           NativeName(Name(struct_def), &struct_def, opts));
  }

  const std::string &PtrType(const FieldDef *field) {
    auto attr = field ? field->attributes.Lookup("cpp_ptr_type") : nullptr;
    return attr ? attr->constant : opts_.cpp_object_api_pointer_type;
  }

  const std::string NativeString(const FieldDef *field) {
    auto attr = field ? field->attributes.Lookup("cpp_str_type") : nullptr;
    auto &ret = attr ? attr->constant : opts_.cpp_object_api_string_type;
    if (ret.empty()) { return "std::string"; }
    return ret;
  }

  bool FlexibleStringConstructor(const FieldDef *field) {
    auto attr = field != nullptr &&
                (field->attributes.Lookup("cpp_str_flex_ctor") != nullptr);
    auto ret = attr ? attr : opts_.cpp_object_api_string_flexible_constructor;
    return ret && NativeString(field) !=
                      "std::string";  // Only for custom string types.
  }

  std::string GenTypeNativePtr(const std::string &type, const FieldDef *field,
                               bool is_constructor) {
    auto &ptr_type = PtrType(field);
    if (ptr_type != "naked") {
      return (ptr_type != "default_ptr_type"
                  ? ptr_type
                  : opts_.cpp_object_api_pointer_type) +
             "<" + type + ">";
    } else if (is_constructor) {
      return "";
    } else {
      return type + " *";
    }
  }

  std::string GenPtrGet(const FieldDef &field) {
    auto cpp_ptr_type_get = field.attributes.Lookup("cpp_ptr_type_get");
    if (cpp_ptr_type_get) return cpp_ptr_type_get->constant;
    auto &ptr_type = PtrType(&field);
    return ptr_type == "naked" ? "" : ".get()";
  }

  std::string GenOptionalNull() { return "::flatbuffers::nullopt"; }

  std::string GenOptionalDecl(const Type &type) {
    return "::flatbuffers::Optional<" + GenTypeBasic(type, true) + ">";
  }

  std::string GenTypeNative(const Type &type, bool invector,
                            const FieldDef &field, bool forcopy = false) {
    switch (type.base_type) {
      case BASE_TYPE_STRING: {
        return NativeString(&field);
      }
      case BASE_TYPE_VECTOR64:
      case BASE_TYPE_VECTOR: {
        const auto type_name = GenTypeNative(type.VectorType(), true, field);
        if (type.struct_def &&
            type.struct_def->attributes.Lookup("native_custom_alloc")) {
          auto native_custom_alloc =
              type.struct_def->attributes.Lookup("native_custom_alloc");
          return "std::vector<" + type_name + "," +
                 native_custom_alloc->constant + "<" + type_name + ">>";
        } else {
          return "std::vector<" + type_name + ">";
        }
      }
      case BASE_TYPE_STRUCT: {
        auto type_name = WrapInNameSpace(*type.struct_def);
        if (IsStruct(type)) {
          auto native_type = type.struct_def->attributes.Lookup("native_type");
          if (native_type) { type_name = native_type->constant; }
          if (invector || field.native_inline || forcopy) {
            return type_name;
          } else {
            return GenTypeNativePtr(type_name, &field, false);
          }
        } else {
          const auto nn = WrapNativeNameInNameSpace(*type.struct_def, opts_);
          return (forcopy || field.native_inline)
                     ? nn
                     : GenTypeNativePtr(nn, &field, false);
        }
      }
      case BASE_TYPE_UNION: {
        auto type_name = WrapInNameSpace(*type.enum_def);
        return type_name + "Union";
      }
      default: {
        return field.IsScalarOptional() ? GenOptionalDecl(type)
                                        : GenTypeBasic(type, true);
      }
    }
  }

  // Return a C++ type for any type (scalar/pointer) specifically for
  // using a flatbuffer.
  std::string GenTypeGet(const Type &type, const char *afterbasic,
                         const char *beforeptr, const char *afterptr,
                         bool user_facing_type) {
    if (IsScalar(type.base_type)) {
      return GenTypeBasic(type, user_facing_type) + afterbasic;
    } else if (IsArray(type)) {
      auto element_type = type.VectorType();
      // Check if enum arrays are used in C++ without specifying --scoped-enums
      if (IsEnum(element_type) && !opts_.g_only_fixed_enums) {
        LogCompilerError(
            "--scoped-enums must be enabled to use enum arrays in C++");
        FLATBUFFERS_ASSERT(true);
      }
      return beforeptr +
             (IsScalar(element_type.base_type)
                  ? GenTypeBasic(element_type, user_facing_type)
                  : GenTypePointer(element_type)) +
             afterptr;
    } else {
      return beforeptr + GenTypePointer(type) + afterptr;
    }
  }

  std::string GenTypeSpan(const Type &type, bool immutable, size_t extent) {
    // Generate "::flatbuffers::span<const U, extent>".
    FLATBUFFERS_ASSERT(IsSeries(type) && "unexpected type");
    auto element_type = type.VectorType();
    std::string text = "::flatbuffers::span<";
    text += immutable ? "const " : "";
    if (IsScalar(element_type.base_type)) {
      text += GenTypeBasic(element_type, IsEnum(element_type));
    } else {
      switch (element_type.base_type) {
        case BASE_TYPE_STRING: {
          text += "char";
          break;
        }
        case BASE_TYPE_STRUCT: {
          FLATBUFFERS_ASSERT(type.struct_def);
          text += WrapInNameSpace(*type.struct_def);
          break;
        }
        default:
          FLATBUFFERS_ASSERT(false && "unexpected element's type");
          break;
      }
    }
    if (extent != dynamic_extent) {
      text += ", ";
      text += NumToString(extent);
    }
    text += "> ";
    return text;
  }

  std::string GenEnumValDecl(const EnumDef &enum_def,
                             const std::string &enum_val) const {
    return opts_.prefixed_enums ? Name(enum_def) + "_" + enum_val : enum_val;
  }

  std::string GetEnumValUse(const EnumDef &enum_def,
                            const EnumVal &enum_val) const {
    if (opts_.scoped_enums) {
      return Name(enum_def) + "::" + Name(enum_val);
    } else if (opts_.prefixed_enums) {
      return Name(enum_def) + "_" + Name(enum_val);
    } else {
      return Name(enum_val);
    }
  }

  std::string StripUnionType(const std::string &name) {
    return name.substr(0, name.size() - strlen(UnionTypeFieldSuffix()));
  }

  std::string GetUnionElement(const EnumVal &ev, bool native_type,
                              const IDLOptions &opts) {
    if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
      std::string name = ev.union_type.struct_def->name;
      if (native_type) {
        name = NativeName(std::move(name), ev.union_type.struct_def, opts);
      }
      return WrapInNameSpace(ev.union_type.struct_def->defined_namespace, name);
    } else if (IsString(ev.union_type)) {
      return native_type ? "std::string" : "::flatbuffers::String";
    } else {
      FLATBUFFERS_ASSERT(false);
      return Name(ev);
    }
  }

  std::string UnionVerifySignature(const EnumDef &enum_def) {
    return "bool Verify" + Name(enum_def) +
           "(::flatbuffers::Verifier &verifier, const void *obj, " +
           Name(enum_def) + " type)";
  }

  std::string UnionVectorVerifySignature(const EnumDef &enum_def) {
    const std::string name = Name(enum_def);
    const std::string &type =
        opts_.scoped_enums ? name
                           : GenTypeBasic(enum_def.underlying_type, false);
    return "bool Verify" + name + "Vector" +
           "(::flatbuffers::Verifier &verifier, " +
           "const ::flatbuffers::Vector<::flatbuffers::Offset<void>> "
           "*values, " +
           "const ::flatbuffers::Vector<" + type + "> *types)";
  }

  std::string UnionUnPackSignature(const EnumDef &enum_def, bool inclass) {
    return (inclass ? "static " : "") + std::string("void *") +
           (inclass ? "" : Name(enum_def) + "Union::") +
           "UnPack(const void *obj, " + Name(enum_def) +
           " type, const ::flatbuffers::resolver_function_t *resolver)";
  }

  std::string UnionPackSignature(const EnumDef &enum_def, bool inclass) {
    return "::flatbuffers::Offset<void> " +
           (inclass ? "" : Name(enum_def) + "Union::") + "Pack(" +
           GetBuilder() + " &_fbb, " +
           "const ::flatbuffers::rehasher_function_t *_rehasher" +
           (inclass ? " = nullptr" : "") + ") const";
  }

  std::string TableCreateSignature(const StructDef &struct_def, bool predecl,
                                   const IDLOptions &opts) {
    return "::flatbuffers::Offset<" + Name(struct_def) + "> Create" +
           Name(struct_def) + "(" + GetBuilder() + " &_fbb, const " +
           NativeName(Name(struct_def), &struct_def, opts) +
           " *_o, const ::flatbuffers::rehasher_function_t *_rehasher" +
           (predecl ? " = nullptr" : "") + ")";
  }

  std::string TablePackSignature(const StructDef &struct_def, bool inclass,
                                 const IDLOptions &opts) {
    return std::string(inclass ? "static " : "") + "::flatbuffers::Offset<" +
           Name(struct_def) + "> " + (inclass ? "" : Name(struct_def) + "::") +
           "Pack(" + GetBuilder() + " &_fbb, " + "const " +
           NativeName(Name(struct_def), &struct_def, opts) + "* _o, " +
           "const ::flatbuffers::rehasher_function_t *_rehasher" +
           (inclass ? " = nullptr" : "") + ")";
  }

  std::string TableUnPackSignature(const StructDef &struct_def, bool inclass,
                                   const IDLOptions &opts) {
    return NativeName(Name(struct_def), &struct_def, opts) + " *" +
           (inclass ? "" : Name(struct_def) + "::") +
           "UnPack(const ::flatbuffers::resolver_function_t *_resolver" +
           (inclass ? " = nullptr" : "") + ") const";
  }

  std::string TableUnPackToSignature(const StructDef &struct_def, bool inclass,
                                     const IDLOptions &opts) {
    return "void " + (inclass ? "" : Name(struct_def) + "::") + "UnPackTo(" +
           NativeName(Name(struct_def), &struct_def, opts) + " *" +
           "_o, const ::flatbuffers::resolver_function_t *_resolver" +
           (inclass ? " = nullptr" : "") + ") const";
  }

  void GenMiniReflectPre(const StructDef *struct_def) {
    code_.SetValue("NAME", struct_def->name);
    code_ += "inline const ::flatbuffers::TypeTable *{{NAME}}TypeTable();";
    code_ += "";
  }

  void GenMiniReflect(const StructDef *struct_def, const EnumDef *enum_def) {
    code_.SetValue("NAME", struct_def ? struct_def->name : enum_def->name);
    code_.SetValue("SEQ_TYPE",
                   struct_def ? (struct_def->fixed ? "ST_STRUCT" : "ST_TABLE")
                              : (enum_def->is_union ? "ST_UNION" : "ST_ENUM"));
    auto num_fields =
        struct_def ? struct_def->fields.vec.size() : enum_def->size();
    code_.SetValue("NUM_FIELDS", NumToString(num_fields));
    std::vector<std::string> names;
    std::vector<Type> types;

    if (struct_def) {
      for (const auto &field : struct_def->fields.vec) {
        names.push_back(Name(*field));
        types.push_back(field->value.type);
      }
    } else {
      for (auto it = enum_def->Vals().begin(); it != enum_def->Vals().end();
           ++it) {
        const auto &ev = **it;
        names.push_back(Name(ev));
        types.push_back(enum_def->is_union ? ev.union_type
                                           : Type(enum_def->underlying_type));
      }
    }
    std::string ts;
    std::vector<std::string> type_refs;
    std::vector<uint16_t> array_sizes;
    for (auto &type : types) {
      if (!ts.empty()) ts += ",\n    ";
      auto is_vector = IsVector(type);
      auto is_array = IsArray(type);
      auto bt = is_vector || is_array ? type.element : type.base_type;
      auto et = IsScalar(bt) || bt == BASE_TYPE_STRING
                    ? bt - BASE_TYPE_UTYPE + ET_UTYPE
                    : ET_SEQUENCE;
      int ref_idx = -1;
      std::string ref_name = type.struct_def ? WrapInNameSpace(*type.struct_def)
                             : type.enum_def ? WrapInNameSpace(*type.enum_def)
                                             : "";
      if (!ref_name.empty()) {
        auto rit = type_refs.begin();
        for (; rit != type_refs.end(); ++rit) {
          if (*rit == ref_name) {
            ref_idx = static_cast<int>(rit - type_refs.begin());
            break;
          }
        }
        if (rit == type_refs.end()) {
          ref_idx = static_cast<int>(type_refs.size());
          type_refs.push_back(ref_name);
        }
      }
      if (is_array) { array_sizes.push_back(type.fixed_length); }
      ts += "{ ::flatbuffers::" + std::string(ElementaryTypeNames()[et]) +
            ", " + NumToString(is_vector || is_array) + ", " +
            NumToString(ref_idx) + " }";
    }
    std::string rs;
    for (auto &type_ref : type_refs) {
      if (!rs.empty()) rs += ",\n    ";
      rs += type_ref + "TypeTable";
    }
    std::string as;
    for (auto &array_size : array_sizes) {
      as += NumToString(array_size);
      as += ", ";
    }
    std::string ns;
    for (auto &name : names) {
      if (!ns.empty()) ns += ",\n    ";
      ns += "\"" + name + "\"";
    }
    std::string vs;
    const auto consecutive_enum_from_zero =
        enum_def && enum_def->MinValue()->IsZero() &&
        ((enum_def->size() - 1) == enum_def->Distance());
    if (enum_def && !consecutive_enum_from_zero) {
      for (auto it = enum_def->Vals().begin(); it != enum_def->Vals().end();
           ++it) {
        const auto &ev = **it;
        if (!vs.empty()) vs += ", ";
        vs += NumToStringCpp(enum_def->ToString(ev),
                             enum_def->underlying_type.base_type);
      }
    } else if (struct_def && struct_def->fixed) {
      for (const auto field : struct_def->fields.vec) {
        vs += NumToString(field->value.offset);
        vs += ", ";
      }
      vs += NumToString(struct_def->bytesize);
    }
    code_.SetValue("TYPES", ts);
    code_.SetValue("REFS", rs);
    code_.SetValue("ARRAYSIZES", as);
    code_.SetValue("NAMES", ns);
    code_.SetValue("VALUES", vs);
    code_ += "inline const ::flatbuffers::TypeTable *{{NAME}}TypeTable() {";
    if (num_fields) {
      code_ += "  static const ::flatbuffers::TypeCode type_codes[] = {";
      code_ += "    {{TYPES}}";
      code_ += "  };";
    }
    if (!type_refs.empty()) {
      code_ += "  static const ::flatbuffers::TypeFunction type_refs[] = {";
      code_ += "    {{REFS}}";
      code_ += "  };";
    }
    if (!as.empty()) {
      code_ += "  static const int16_t array_sizes[] = { {{ARRAYSIZES}} };";
    }
    if (!vs.empty()) {
      // Problem with uint64_t values greater than 9223372036854775807ULL.
      code_ += "  static const int64_t values[] = { {{VALUES}} };";
    }
    auto has_names =
        num_fields && opts_.mini_reflect == IDLOptions::kTypesAndNames;
    if (has_names) {
      code_ += "  static const char * const names[] = {";
      code_ += "    {{NAMES}}";
      code_ += "  };";
    }
    code_ += "  static const ::flatbuffers::TypeTable tt = {";
    code_ += std::string("    ::flatbuffers::{{SEQ_TYPE}}, {{NUM_FIELDS}}, ") +
             (num_fields ? "type_codes, " : "nullptr, ") +
             (!type_refs.empty() ? "type_refs, " : "nullptr, ") +
             (!as.empty() ? "array_sizes, " : "nullptr, ") +
             (!vs.empty() ? "values, " : "nullptr, ") +
             (has_names ? "names" : "nullptr");
    code_ += "  };";
    code_ += "  return &tt;";
    code_ += "}";
    code_ += "";
  }

  // Generate an enum declaration,
  // an enum string lookup table,
  // and an enum array of values

  void GenEnum(const EnumDef &enum_def) {
    code_.SetValue("ENUM_NAME", Name(enum_def));
    code_.SetValue("BASE_TYPE", GenTypeBasic(enum_def.underlying_type, false));

    GenComment(enum_def.doc_comment);
    code_ +=
        (opts_.scoped_enums ? "enum class " : "enum ") + Name(enum_def) + "\\";
    if (opts_.g_only_fixed_enums) { code_ += " : {{BASE_TYPE}}\\"; }
    code_ += " {";

    code_.SetValue("SEP", ",");
    auto add_sep = false;
    for (const auto ev : enum_def.Vals()) {
      if (add_sep) code_ += "{{SEP}}";
      GenComment(ev->doc_comment, "  ");
      code_.SetValue("KEY", GenEnumValDecl(enum_def, Name(*ev)));
      code_.SetValue("VALUE",
                     NumToStringCpp(enum_def.ToString(*ev),
                                    enum_def.underlying_type.base_type));
      code_ += "  {{KEY}} = {{VALUE}}\\";
      add_sep = true;
    }
    if (opts_.cpp_minify_enums) {
      code_ += "";
      code_ += "};";
      return;
    }
    const EnumVal *minv = enum_def.MinValue();
    const EnumVal *maxv = enum_def.MaxValue();

    if (opts_.scoped_enums || opts_.prefixed_enums) {
      FLATBUFFERS_ASSERT(minv && maxv);

      code_.SetValue("SEP", ",\n");

      // MIN & MAX are useless for bit_flags
      if (enum_def.attributes.Lookup("bit_flags")) {
        code_.SetValue("KEY", GenEnumValDecl(enum_def, "NONE"));
        code_.SetValue("VALUE", "0");
        code_ += "{{SEP}}  {{KEY}} = {{VALUE}}\\";

        code_.SetValue("KEY", GenEnumValDecl(enum_def, "ANY"));
        code_.SetValue("VALUE",
                       NumToStringCpp(enum_def.AllFlags(),
                                      enum_def.underlying_type.base_type));
        code_ += "{{SEP}}  {{KEY}} = {{VALUE}}\\";
      } else if (opts_.emit_min_max_enum_values) {
        code_.SetValue("KEY", GenEnumValDecl(enum_def, "MIN"));
        code_.SetValue("VALUE", GenEnumValDecl(enum_def, Name(*minv)));
        code_ += "{{SEP}}  {{KEY}} = {{VALUE}}\\";

        code_.SetValue("KEY", GenEnumValDecl(enum_def, "MAX"));
        code_.SetValue("VALUE", GenEnumValDecl(enum_def, Name(*maxv)));
        code_ += "{{SEP}}  {{KEY}} = {{VALUE}}\\";
      }
    }
    code_ += "";
    code_ += "};";

    if (opts_.scoped_enums && enum_def.attributes.Lookup("bit_flags")) {
      code_ +=
          "FLATBUFFERS_DEFINE_BITMASK_OPERATORS({{ENUM_NAME}}, {{BASE_TYPE}})";
    }
    code_ += "";
    GenEnumArray(enum_def);
    GenEnumStringTable(enum_def);

    // Generate type traits for unions to map from a type to union enum value.
    if (enum_def.is_union && !enum_def.uses_multiple_type_instances) {
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        const auto &ev = **it;

        if (it == enum_def.Vals().begin()) {
          code_ += "template<typename T> struct {{ENUM_NAME}}Traits {";
        } else {
          auto name = GetUnionElement(ev, false, opts_);
          code_ += "template<> struct {{ENUM_NAME}}Traits<" + name + "> {";
        }

        auto value = GetEnumValUse(enum_def, ev);
        code_ += "  static const {{ENUM_NAME}} enum_value = " + value + ";";
        code_ += "};";
        code_ += "";
      }
    }

    GenEnumObjectBasedAPI(enum_def);

    if (enum_def.is_union) {
      code_ += UnionVerifySignature(enum_def) + ";";
      code_ += UnionVectorVerifySignature(enum_def) + ";";
      code_ += "";
    }
  }

  // Generate a union type and a trait type for it.
  void GenEnumObjectBasedAPI(const EnumDef &enum_def) {
    if (!(opts_.generate_object_based_api && enum_def.is_union)) { return; }
    code_.SetValue("NAME", Name(enum_def));
    FLATBUFFERS_ASSERT(enum_def.Lookup("NONE"));
    code_.SetValue("NONE", GetEnumValUse(enum_def, *enum_def.Lookup("NONE")));

    if (!enum_def.uses_multiple_type_instances) {
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        const auto &ev = **it;

        if (it == enum_def.Vals().begin()) {
          code_ += "template<typename T> struct {{NAME}}UnionTraits {";
        } else {
          auto name = GetUnionElement(ev, true, opts_);
          code_ += "template<> struct {{NAME}}UnionTraits<" + name + "> {";
        }

        auto value = GetEnumValUse(enum_def, ev);
        code_ += "  static const {{ENUM_NAME}} enum_value = " + value + ";";
        code_ += "};";
        code_ += "";
      }
    }

    code_ += "struct {{NAME}}Union {";
    code_ += "  {{NAME}} type;";
    code_ += "  void *value;";
    code_ += "";
    code_ += "  {{NAME}}Union() : type({{NONE}}), value(nullptr) {}";
    code_ += "  {{NAME}}Union({{NAME}}Union&& u) FLATBUFFERS_NOEXCEPT :";
    code_ += "    type({{NONE}}), value(nullptr)";
    code_ += "    { std::swap(type, u.type); std::swap(value, u.value); }";
    code_ += "  {{NAME}}Union(const {{NAME}}Union &);";
    code_ += "  {{NAME}}Union &operator=(const {{NAME}}Union &u)";
    code_ +=
        "    { {{NAME}}Union t(u); std::swap(type, t.type); std::swap(value, "
        "t.value); return *this; }";
    code_ +=
        "  {{NAME}}Union &operator=({{NAME}}Union &&u) FLATBUFFERS_NOEXCEPT";
    code_ +=
        "    { std::swap(type, u.type); std::swap(value, u.value); return "
        "*this; }";
    code_ += "  ~{{NAME}}Union() { Reset(); }";
    code_ += "";
    code_ += "  void Reset();";
    code_ += "";
    if (!enum_def.uses_multiple_type_instances) {
      code_ += "  template <typename T>";
      code_ += "  void Set(T&& val) {";
      code_ += "    typedef typename std::remove_reference<T>::type RT;";
      code_ += "    Reset();";
      code_ += "    type = {{NAME}}UnionTraits<RT>::enum_value;";
      code_ += "    if (type != {{NONE}}) {";
      code_ += "      value = new RT(std::forward<T>(val));";
      code_ += "    }";
      code_ += "  }";
      code_ += "";
    }
    code_ += "  " + UnionUnPackSignature(enum_def, true) + ";";
    code_ += "  " + UnionPackSignature(enum_def, true) + ";";
    code_ += "";

    for (const auto ev : enum_def.Vals()) {
      if (ev->IsZero()) { continue; }

      const auto native_type = GetUnionElement(*ev, true, opts_);
      code_.SetValue("NATIVE_TYPE", native_type);
      code_.SetValue("NATIVE_NAME", Name(*ev));
      code_.SetValue("NATIVE_ID", GetEnumValUse(enum_def, *ev));

      code_ += "  {{NATIVE_TYPE}} *As{{NATIVE_NAME}}() {";
      code_ += "    return type == {{NATIVE_ID}} ?";
      code_ += "      reinterpret_cast<{{NATIVE_TYPE}} *>(value) : nullptr;";
      code_ += "  }";

      code_ += "  const {{NATIVE_TYPE}} *As{{NATIVE_NAME}}() const {";
      code_ += "    return type == {{NATIVE_ID}} ?";
      code_ +=
          "      reinterpret_cast<const {{NATIVE_TYPE}} *>(value) : nullptr;";
      code_ += "  }";
    }
    code_ += "};";
    code_ += "";

    GenEnumEquals(enum_def);
  }

  void GenEnumEquals(const EnumDef &enum_def) {
    if (opts_.gen_compare) {
      code_ += "";
      code_ +=
          "inline bool operator==(const {{NAME}}Union &lhs, const "
          "{{NAME}}Union &rhs) {";
      code_ += "  if (lhs.type != rhs.type) return false;";
      code_ += "  switch (lhs.type) {";

      for (const auto &ev : enum_def.Vals()) {
        code_.SetValue("NATIVE_ID", GetEnumValUse(enum_def, *ev));
        if (ev->IsNonZero()) {
          const auto native_type = GetUnionElement(*ev, true, opts_);
          code_.SetValue("NATIVE_TYPE", native_type);
          code_ += "    case {{NATIVE_ID}}: {";
          code_ +=
              "      return *(reinterpret_cast<const {{NATIVE_TYPE}} "
              "*>(lhs.value)) ==";
          code_ +=
              "             *(reinterpret_cast<const {{NATIVE_TYPE}} "
              "*>(rhs.value));";
          code_ += "    }";
        } else {
          code_ += "    case {{NATIVE_ID}}: {";
          code_ += "      return true;";  // "NONE" enum value.
          code_ += "    }";
        }
      }
      code_ += "    default: {";
      code_ += "      return false;";
      code_ += "    }";
      code_ += "  }";
      code_ += "}";

      code_ += "";
      code_ +=
          "inline bool operator!=(const {{NAME}}Union &lhs, const "
          "{{NAME}}Union &rhs) {";
      code_ += "    return !(lhs == rhs);";
      code_ += "}";
      code_ += "";
    }
  }

  // Generate an array of all enumeration values
  void GenEnumArray(const EnumDef &enum_def) {
    auto num_fields = NumToString(enum_def.size());
    code_ += "inline const {{ENUM_NAME}} (&EnumValues{{ENUM_NAME}}())[" +
             num_fields + "] {";
    code_ += "  static const {{ENUM_NAME}} values[] = {";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      const auto &ev = **it;
      auto value = GetEnumValUse(enum_def, ev);
      auto suffix = *it != enum_def.Vals().back() ? "," : "";
      code_ += "    " + value + suffix;
    }
    code_ += "  };";
    code_ += "  return values;";
    code_ += "}";
    code_ += "";
  }

  // Generate a string table for enum values.
  // Problem is, if values are very sparse that could generate huge tables.
  // Ideally in that case we generate a map lookup instead, but for the moment
  // we simply don't output a table at all.
  void GenEnumStringTable(const EnumDef &enum_def) {
    auto range = enum_def.Distance();
    // Average distance between values above which we consider a table
    // "too sparse". Change at will.
    static const uint64_t kMaxSparseness = 5;
    if (range / static_cast<uint64_t>(enum_def.size()) < kMaxSparseness) {
      code_ += "inline const char * const *EnumNames{{ENUM_NAME}}() {";
      code_ += "  static const char * const names[" +
               NumToString(range + 1 + 1) + "] = {";

      auto val = enum_def.Vals().front();
      for (const auto &enum_value : enum_def.Vals()) {
        for (auto k = enum_def.Distance(val, enum_value); k > 1; --k) {
          code_ += "    \"\",";
        }
        val = enum_value;
        code_ += "    \"" + Name(*enum_value) + "\",";
      }
      code_ += "    nullptr";
      code_ += "  };";

      code_ += "  return names;";
      code_ += "}";
      code_ += "";

      code_ += "inline const char *EnumName{{ENUM_NAME}}({{ENUM_NAME}} e) {";

      code_ += "  if (::flatbuffers::IsOutRange(e, " +
               GetEnumValUse(enum_def, *enum_def.MinValue()) + ", " +
               GetEnumValUse(enum_def, *enum_def.MaxValue()) +
               ")) return \"\";";

      code_ += "  const size_t index = static_cast<size_t>(e)\\";
      if (enum_def.MinValue()->IsNonZero()) {
        auto vals = GetEnumValUse(enum_def, *enum_def.MinValue());
        code_ += " - static_cast<size_t>(" + vals + ")\\";
      }
      code_ += ";";

      code_ += "  return EnumNames{{ENUM_NAME}}()[index];";
      code_ += "}";
      code_ += "";
    } else {
      code_ += "inline const char *EnumName{{ENUM_NAME}}({{ENUM_NAME}} e) {";
      code_ += "  switch (e) {";
      for (const auto &ev : enum_def.Vals()) {
        code_ += "    case " + GetEnumValUse(enum_def, *ev) + ": return \"" +
                 Name(*ev) + "\";";
      }
      code_ += "    default: return \"\";";
      code_ += "  }";
      code_ += "}";
      code_ += "";
    }
  }

  void GenUnionPost(const EnumDef &enum_def) {
    // Generate a verifier function for this union that can be called by the
    // table verifier functions. It uses a switch case to select a specific
    // verifier function to call, this should be safe even if the union type
    // has been corrupted, since the verifiers will simply fail when called
    // on the wrong type.
    code_.SetValue("ENUM_NAME", Name(enum_def));

    code_ += "inline " + UnionVerifySignature(enum_def) + " {";
    code_ += "  switch (type) {";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      const auto &ev = **it;
      code_.SetValue("LABEL", GetEnumValUse(enum_def, ev));

      if (ev.IsNonZero()) {
        code_.SetValue("TYPE", GetUnionElement(ev, false, opts_));
        code_ += "    case {{LABEL}}: {";
        auto getptr =
            "      auto ptr = reinterpret_cast<const {{TYPE}} *>(obj);";
        if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
          if (ev.union_type.struct_def->fixed) {
            code_.SetValue("ALIGN",
                           NumToString(ev.union_type.struct_def->minalign));
            code_ +=
                "      return verifier.VerifyField<{{TYPE}}>("
                "static_cast<const uint8_t *>(obj), 0, {{ALIGN}});";
          } else {
            code_ += getptr;
            code_ += "      return verifier.VerifyTable(ptr);";
          }
        } else if (IsString(ev.union_type)) {
          code_ += getptr;
          code_ += "      return verifier.VerifyString(ptr);";
        } else {
          FLATBUFFERS_ASSERT(false);
        }
        code_ += "    }";
      } else {
        code_ += "    case {{LABEL}}: {";
        code_ += "      return true;";  // "NONE" enum value.
        code_ += "    }";
      }
    }
    code_ += "    default: return true;";  // unknown values are OK.
    code_ += "  }";
    code_ += "}";
    code_ += "";

    code_ += "inline " + UnionVectorVerifySignature(enum_def) + " {";
    code_ += "  if (!values || !types) return !values && !types;";
    code_ += "  if (values->size() != types->size()) return false;";
    code_ +=
        "  for (::flatbuffers::uoffset_t i = 0; i < values->size(); ++i) {";
    code_ += "    if (!Verify" + Name(enum_def) + "(";
    code_ += "        verifier,  values->Get(i), types->GetEnum<" +
             Name(enum_def) + ">(i))) {";
    code_ += "      return false;";
    code_ += "    }";
    code_ += "  }";
    code_ += "  return true;";
    code_ += "}";
    code_ += "";

    if (opts_.generate_object_based_api) {
      // Generate union Unpack() and Pack() functions.
      code_ += "inline " + UnionUnPackSignature(enum_def, false) + " {";
      code_ += "  (void)resolver;";
      code_ += "  switch (type) {";
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        const auto &ev = **it;
        if (ev.IsZero()) { continue; }

        code_.SetValue("LABEL", GetEnumValUse(enum_def, ev));
        code_.SetValue("TYPE", GetUnionElement(ev, false, opts_));
        code_ += "    case {{LABEL}}: {";
        code_ += "      auto ptr = reinterpret_cast<const {{TYPE}} *>(obj);";
        if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
          if (ev.union_type.struct_def->fixed) {
            code_ += "      return new " +
                     WrapInNameSpace(*ev.union_type.struct_def) + "(*ptr);";
          } else {
            code_ += "      return ptr->UnPack(resolver);";
          }
        } else if (IsString(ev.union_type)) {
          code_ += "      return new std::string(ptr->c_str(), ptr->size());";
        } else {
          FLATBUFFERS_ASSERT(false);
        }
        code_ += "    }";
      }
      code_ += "    default: return nullptr;";
      code_ += "  }";
      code_ += "}";
      code_ += "";

      code_ += "inline " + UnionPackSignature(enum_def, false) + " {";
      code_ += "  (void)_rehasher;";
      code_ += "  switch (type) {";
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        auto &ev = **it;
        if (ev.IsZero()) { continue; }

        code_.SetValue("LABEL", GetEnumValUse(enum_def, ev));
        code_.SetValue("TYPE", GetUnionElement(ev, true, opts_));
        code_ += "    case {{LABEL}}: {";
        code_ += "      auto ptr = reinterpret_cast<const {{TYPE}} *>(value);";
        if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
          if (ev.union_type.struct_def->fixed) {
            code_ += "      return _fbb.CreateStruct(*ptr).Union();";
          } else {
            code_.SetValue("NAME", ev.union_type.struct_def->name);
            code_ +=
                "      return Create{{NAME}}(_fbb, ptr, _rehasher).Union();";
          }
        } else if (IsString(ev.union_type)) {
          code_ += "      return _fbb.CreateString(*ptr).Union();";
        } else {
          FLATBUFFERS_ASSERT(false);
        }
        code_ += "    }";
      }
      code_ += "    default: return 0;";
      code_ += "  }";
      code_ += "}";
      code_ += "";

      // Union copy constructor
      code_ +=
          "inline {{ENUM_NAME}}Union::{{ENUM_NAME}}Union(const "
          "{{ENUM_NAME}}Union &u) : type(u.type), value(nullptr) {";
      code_ += "  switch (type) {";
      for (const auto &ev : enum_def.Vals()) {
        if (ev->IsZero()) { continue; }
        code_.SetValue("LABEL", GetEnumValUse(enum_def, *ev));
        code_.SetValue("TYPE", GetUnionElement(*ev, true, opts_));
        code_ += "    case {{LABEL}}: {";
        bool copyable = true;
        if (opts_.g_cpp_std < cpp::CPP_STD_11 &&
            ev->union_type.base_type == BASE_TYPE_STRUCT &&
            !ev->union_type.struct_def->fixed) {
          // Don't generate code to copy if table is not copyable.
          // TODO(wvo): make tables copyable instead.
          for (const auto &field : ev->union_type.struct_def->fields.vec) {
            if (!field->deprecated && field->value.type.struct_def &&
                !field->native_inline) {
              copyable = false;
              break;
            }
          }
        }
        if (copyable) {
          code_ +=
              "      value = new {{TYPE}}(*reinterpret_cast<{{TYPE}} *>"
              "(u.value));";
        } else {
          code_ +=
              "      FLATBUFFERS_ASSERT(false);  // {{TYPE}} not copyable.";
        }
        code_ += "      break;";
        code_ += "    }";
      }
      code_ += "    default:";
      code_ += "      break;";
      code_ += "  }";
      code_ += "}";
      code_ += "";

      // Union Reset() function.
      FLATBUFFERS_ASSERT(enum_def.Lookup("NONE"));
      code_.SetValue("NONE", GetEnumValUse(enum_def, *enum_def.Lookup("NONE")));

      code_ += "inline void {{ENUM_NAME}}Union::Reset() {";
      code_ += "  switch (type) {";
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        const auto &ev = **it;
        if (ev.IsZero()) { continue; }
        code_.SetValue("LABEL", GetEnumValUse(enum_def, ev));
        code_.SetValue("TYPE", GetUnionElement(ev, true, opts_));
        code_ += "    case {{LABEL}}: {";
        code_ += "      auto ptr = reinterpret_cast<{{TYPE}} *>(value);";
        code_ += "      delete ptr;";
        code_ += "      break;";
        code_ += "    }";
      }
      code_ += "    default: break;";
      code_ += "  }";
      code_ += "  value = nullptr;";
      code_ += "  type = {{NONE}};";
      code_ += "}";
      code_ += "";
    }
  }

  // Generates a value with optionally a cast applied if the field has a
  // different underlying type from its interface type (currently only the
  // case for enums. "from" specify the direction, true meaning from the
  // underlying type to the interface type.
  std::string GenUnderlyingCast(const FieldDef &field, bool from,
                                const std::string &val) {
    if (from && field.value.type.base_type == BASE_TYPE_BOOL) {
      return val + " != 0";
    } else if ((field.value.type.enum_def &&
                IsScalar(field.value.type.base_type)) ||
               field.value.type.base_type == BASE_TYPE_BOOL) {
      return "static_cast<" + GenTypeBasic(field.value.type, from) + ">(" +
             val + ")";
    } else {
      return val;
    }
  }

  std::string GenFieldOffsetName(const FieldDef &field) {
    std::string uname = Name(field);
    std::transform(uname.begin(), uname.end(), uname.begin(), CharToUpper);
    return "VT_" + uname;
  }

  void GenFullyQualifiedNameGetter(const StructDef &struct_def,
                                   const std::string &name) {
    if (!opts_.generate_name_strings) { return; }
    auto fullname = struct_def.defined_namespace->GetFullyQualifiedName(name);
    code_.SetValue("NAME", fullname);
    code_.SetValue("CONSTEXPR", "FLATBUFFERS_CONSTEXPR_CPP11");
    code_ += "  static {{CONSTEXPR}} const char *GetFullyQualifiedName() {";
    code_ += "    return \"{{NAME}}\";";
    code_ += "  }";
  }

  std::string GenDefaultConstant(const FieldDef &field) {
    if (IsFloat(field.value.type.base_type))
      return float_const_gen_.GenFloatConstant(field);
    else
      return NumToStringCpp(field.value.constant, field.value.type.base_type);
  }

  std::string GetDefaultScalarValue(const FieldDef &field, bool is_ctor) {
    const auto &type = field.value.type;
    if (field.IsScalarOptional()) {
      return GenOptionalNull();
    } else if (type.enum_def && IsScalar(type.base_type)) {
      auto ev = type.enum_def->FindByValue(field.value.constant);
      if (ev) {
        return WrapInNameSpace(type.enum_def->defined_namespace,
                               GetEnumValUse(*type.enum_def, *ev));
      } else {
        return GenUnderlyingCast(
            field, true, NumToStringCpp(field.value.constant, type.base_type));
      }
    } else if (type.base_type == BASE_TYPE_BOOL) {
      return field.value.constant == "0" ? "false" : "true";
    } else if (field.attributes.Lookup("cpp_type")) {
      if (is_ctor) {
        if (PtrType(&field) == "naked") {
          return "nullptr";
        } else {
          return "";
        }
      } else {
        return "0";
      }
    } else if (IsStruct(type) && (field.value.constant == "0")) {
      return "nullptr";
    } else {
      return GenDefaultConstant(field);
    }
  }

  void GenParam(const FieldDef &field, bool direct, const char *prefix) {
    code_.SetValue("PRE", prefix);
    code_.SetValue("PARAM_NAME", Name(field));
    if (direct && IsString(field.value.type)) {
      code_.SetValue("PARAM_TYPE", "const char *");
      code_.SetValue("PARAM_VALUE", "nullptr");
    } else if (direct && IsVector(field.value.type)) {
      const auto vtype = field.value.type.VectorType();
      std::string type;
      if (IsStruct(vtype)) {
        type = WrapInNameSpace(*vtype.struct_def);
      } else {
        type = GenTypeWire(vtype, "", VectorElementUserFacing(vtype),
                           field.offset64);
      }
      if (TypeHasKey(vtype)) {
        code_.SetValue("PARAM_TYPE", "std::vector<" + type + "> *");
      } else {
        code_.SetValue("PARAM_TYPE", "const std::vector<" + type + "> *");
      }
      code_.SetValue("PARAM_VALUE", "nullptr");
    } else {
      const auto &type = field.value.type;
      code_.SetValue("PARAM_VALUE", GetDefaultScalarValue(field, false));
      if (field.IsScalarOptional())
        code_.SetValue("PARAM_TYPE", GenOptionalDecl(type) + " ");
      else
        code_.SetValue("PARAM_TYPE",
                       GenTypeWire(type, " ", true, field.offset64));
    }
    code_ += "{{PRE}}{{PARAM_TYPE}}{{PARAM_NAME}} = {{PARAM_VALUE}}\\";
  }

  // Generate a member, including a default value for scalars and raw pointers.
  void GenMember(const FieldDef &field) {
    if (!field.deprecated &&  // Deprecated fields won't be accessible.
        field.value.type.base_type != BASE_TYPE_UTYPE &&
        (!IsVector(field.value.type) ||
         field.value.type.element != BASE_TYPE_UTYPE)) {
      auto type = GenTypeNative(field.value.type, false, field);
      auto cpp_type = field.attributes.Lookup("cpp_type");
      const std::string &full_type =
          (cpp_type
               ? (IsVector(field.value.type)
                      ? "std::vector<" +
                            GenTypeNativePtr(cpp_type->constant, &field,
                                             false) +
                            "> "
                      : GenTypeNativePtr(cpp_type->constant, &field, false))
               : type + " ");
      // Generate default member initializers for >= C++11.
      std::string field_di;
      if (opts_.g_cpp_std >= cpp::CPP_STD_11) {
        field_di = "{}";
        auto native_default = field.attributes.Lookup("native_default");
        // Scalar types get parsed defaults, raw pointers get nullptrs.
        if (IsScalar(field.value.type.base_type)) {
          field_di =
              " = " + (native_default ? std::string(native_default->constant)
                                      : GetDefaultScalarValue(field, true));
        } else if (field.value.type.base_type == BASE_TYPE_STRUCT) {
          if (IsStruct(field.value.type) && native_default) {
            field_di = " = " + native_default->constant;
          }
        }
      }
      code_.SetValue("FIELD_TYPE", full_type);
      code_.SetValue("FIELD_NAME", Name(field));
      code_.SetValue("FIELD_DI", field_di);
      code_ += "  {{FIELD_TYPE}}{{FIELD_NAME}}{{FIELD_DI}};";
    }
  }

  // Returns true if `struct_def` needs a copy constructor and assignment
  // operator because it has one or more table members, struct members with a
  // custom cpp_type and non-naked pointer type, or vector members of those.
  bool NeedsCopyCtorAssignOp(const StructDef &struct_def) {
    for (const auto &field : struct_def.fields.vec) {
      const auto &type = field->value.type;
      if (field->deprecated) continue;
      if (type.base_type == BASE_TYPE_STRUCT) {
        const auto cpp_type = field->attributes.Lookup("cpp_type");
        const auto cpp_ptr_type = field->attributes.Lookup("cpp_ptr_type");
        const bool is_ptr = !(IsStruct(type) && field->native_inline) ||
                            (cpp_type && cpp_ptr_type->constant != "naked");
        if (is_ptr) { return true; }
      } else if (IsVector(type)) {
        const auto vec_type = type.VectorType();
        if (vec_type.base_type == BASE_TYPE_UTYPE) continue;
        const auto cpp_type = field->attributes.Lookup("cpp_type");
        const auto cpp_ptr_type = field->attributes.Lookup("cpp_ptr_type");
        const bool is_ptr = IsVectorOfPointers(*field) ||
                            (cpp_type && cpp_ptr_type->constant != "naked");
        if (is_ptr) { return true; }
      }
    }
    return false;
  }

  // Generate the default constructor for this struct. Properly initialize all
  // scalar members with default values.
  void GenDefaultConstructor(const StructDef &struct_def) {
    code_.SetValue("NATIVE_NAME",
                   NativeName(Name(struct_def), &struct_def, opts_));
    // In >= C++11, default member initializers are generated. To allow for
    // aggregate initialization, do not emit a default constructor at all, with
    // the exception of types that need a copy/move ctors and assignment
    // operators.
    if (opts_.g_cpp_std >= cpp::CPP_STD_11) {
      if (NeedsCopyCtorAssignOp(struct_def)) {
        code_ += "  {{NATIVE_NAME}}() = default;";
      }
      return;
    }
    std::string initializer_list;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (!field.deprecated &&  // Deprecated fields won't be accessible.
          field.value.type.base_type != BASE_TYPE_UTYPE) {
        auto cpp_type = field.attributes.Lookup("cpp_type");
        auto native_default = field.attributes.Lookup("native_default");
        // Scalar types get parsed defaults, raw pointers get nullptrs.
        if (IsScalar(field.value.type.base_type)) {
          if (!initializer_list.empty()) { initializer_list += ",\n        "; }
          initializer_list += Name(field);
          initializer_list +=
              "(" +
              (native_default ? std::string(native_default->constant)
                              : GetDefaultScalarValue(field, true)) +
              ")";
        } else if (field.value.type.base_type == BASE_TYPE_STRUCT) {
          if (IsStruct(field.value.type)) {
            if (native_default) {
              if (!initializer_list.empty()) {
                initializer_list += ",\n        ";
              }
              initializer_list +=
                  Name(field) + "(" + native_default->constant + ")";
            }
          }
        } else if (cpp_type && !IsVector(field.value.type)) {
          if (!initializer_list.empty()) { initializer_list += ",\n        "; }
          initializer_list += Name(field) + "(0)";
        }
      }
    }
    if (!initializer_list.empty()) {
      initializer_list = "\n      : " + initializer_list;
    }

    code_.SetValue("INIT_LIST", initializer_list);

    code_ += "  {{NATIVE_NAME}}(){{INIT_LIST}} {";
    code_ += "  }";
  }

  // Generate the >= C++11 copy/move constructor and assignment operator
  // declarations if required. Tables that are default-copyable do not get
  // user-provided copy/move constructors and assignment operators so they
  // remain aggregates.
  void GenCopyMoveCtorAndAssigOpDecls(const StructDef &struct_def) {
    if (opts_.g_cpp_std < cpp::CPP_STD_11) return;
    if (!NeedsCopyCtorAssignOp(struct_def)) return;
    code_.SetValue("NATIVE_NAME",
                   NativeName(Name(struct_def), &struct_def, opts_));
    code_ += "  {{NATIVE_NAME}}(const {{NATIVE_NAME}} &o);";
    code_ +=
        "  {{NATIVE_NAME}}({{NATIVE_NAME}}&&) FLATBUFFERS_NOEXCEPT = "
        "default;";
    code_ +=
        "  {{NATIVE_NAME}} &operator=({{NATIVE_NAME}} o) FLATBUFFERS_NOEXCEPT;";
  }

  // Generate the >= C++11 copy constructor and assignment operator definitions.
  void GenCopyCtorAssignOpDefs(const StructDef &struct_def) {
    if (opts_.g_cpp_std < cpp::CPP_STD_11) return;
    if (!NeedsCopyCtorAssignOp(struct_def)) return;
    std::string initializer_list;
    std::string vector_copies;
    std::string swaps;
    for (const auto &field : struct_def.fields.vec) {
      const auto &type = field->value.type;
      if (field->deprecated || type.base_type == BASE_TYPE_UTYPE) continue;
      if (type.base_type == BASE_TYPE_STRUCT) {
        if (!initializer_list.empty()) { initializer_list += ",\n        "; }
        const auto cpp_type = field->attributes.Lookup("cpp_type");
        const auto cpp_ptr_type = field->attributes.Lookup("cpp_ptr_type");
        const std::string &type_name =
            (cpp_type) ? cpp_type->constant
                       : GenTypeNative(type, /*invector*/ false, *field,
                                       /*forcopy*/ true);
        const bool is_ptr = !(IsStruct(type) && field->native_inline) ||
                            (cpp_type && cpp_ptr_type->constant != "naked");
        CodeWriter cw;
        cw.SetValue("FIELD", Name(*field));
        cw.SetValue("TYPE", type_name);
        if (is_ptr) {
          cw +=
              "{{FIELD}}((o.{{FIELD}}) ? new {{TYPE}}(*o.{{FIELD}}) : "
              "nullptr)\\";
          initializer_list += cw.ToString();
        } else {
          cw += "{{FIELD}}(o.{{FIELD}})\\";
          initializer_list += cw.ToString();
        }
      } else if (IsVector(type)) {
        const auto vec_type = type.VectorType();
        if (vec_type.base_type == BASE_TYPE_UTYPE) continue;
        const auto cpp_type = field->attributes.Lookup("cpp_type");
        const auto cpp_ptr_type = field->attributes.Lookup("cpp_ptr_type");
        const std::string &type_name =
            (cpp_type) ? cpp_type->constant
                       : GenTypeNative(vec_type, /*invector*/ true, *field,
                                       /*forcopy*/ true);
        const bool is_ptr = IsVectorOfPointers(*field) ||
                            (cpp_type && cpp_ptr_type->constant != "naked");
        CodeWriter cw("  ");
        cw.SetValue("FIELD", Name(*field));
        cw.SetValue("TYPE", type_name);
        if (is_ptr) {
          // Use emplace_back to construct the potentially-smart pointer element
          // from a raw pointer to a new-allocated copy.
          cw.IncrementIdentLevel();
          cw += "{{FIELD}}.reserve(o.{{FIELD}}.size());";
          cw +=
              "for (const auto &{{FIELD}}_ : o.{{FIELD}}) { "
              "{{FIELD}}.emplace_back(({{FIELD}}_) ? new {{TYPE}}(*{{FIELD}}_) "
              ": nullptr); }";
          vector_copies += cw.ToString();
        } else {
          // For non-pointer elements, use std::vector's copy constructor in the
          // initializer list. This will yield better performance than an insert
          // range loop for trivially-copyable element types.
          if (!initializer_list.empty()) { initializer_list += ",\n        "; }
          cw += "{{FIELD}}(o.{{FIELD}})\\";
          initializer_list += cw.ToString();
        }
      } else {
        if (!initializer_list.empty()) { initializer_list += ",\n        "; }
        CodeWriter cw;
        cw.SetValue("FIELD", Name(*field));
        cw += "{{FIELD}}(o.{{FIELD}})\\";
        initializer_list += cw.ToString();
      }
      {
        if (!swaps.empty()) { swaps += "\n  "; }
        CodeWriter cw;
        cw.SetValue("FIELD", Name(*field));
        cw += "std::swap({{FIELD}}, o.{{FIELD}});\\";
        swaps += cw.ToString();
      }
    }
    if (!initializer_list.empty()) {
      initializer_list = "\n      : " + initializer_list;
    }
    if (!swaps.empty()) { swaps = "  " + swaps; }

    code_.SetValue("NATIVE_NAME",
                   NativeName(Name(struct_def), &struct_def, opts_));
    code_.SetValue("INIT_LIST", initializer_list);
    code_.SetValue("VEC_COPY", vector_copies);
    code_.SetValue("SWAPS", swaps);

    code_ +=
        "inline {{NATIVE_NAME}}::{{NATIVE_NAME}}(const {{NATIVE_NAME}} &o)"
        "{{INIT_LIST}} {";
    code_ += "{{VEC_COPY}}}\n";
    code_ +=
        "inline {{NATIVE_NAME}} &{{NATIVE_NAME}}::operator="
        "({{NATIVE_NAME}} o) FLATBUFFERS_NOEXCEPT {";
    code_ += "{{SWAPS}}";
    code_ += "  return *this;\n}\n";
  }

  void GenCompareOperator(const StructDef &struct_def,
                          const std::string &accessSuffix = "") {
    std::string compare_op;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      const auto accessor = Name(field) + accessSuffix;
      const auto lhs_accessor = "lhs." + accessor;
      const auto rhs_accessor = "rhs." + accessor;
      if (!field.deprecated &&  // Deprecated fields won't be accessible.
          field.value.type.base_type != BASE_TYPE_UTYPE &&
          (!IsVector(field.value.type) ||
           field.value.type.element != BASE_TYPE_UTYPE)) {
        if (!compare_op.empty()) { compare_op += " &&\n      "; }
        if (struct_def.fixed || field.native_inline ||
            field.value.type.base_type != BASE_TYPE_STRUCT) {
          // If the field is a vector of tables, the table need to be compared
          // by value, instead of by the default unique_ptr == operator which
          // compares by address.
          if (IsVectorOfPointers(field)) {
            const auto type =
                GenTypeNative(field.value.type.VectorType(), true, field);
            const auto equal_length =
                lhs_accessor + ".size() == " + rhs_accessor + ".size()";
            const auto elements_equal =
                "std::equal(" + lhs_accessor + ".cbegin(), " + lhs_accessor +
                ".cend(), " + rhs_accessor + ".cbegin(), [](" + type +
                " const &a, " + type +
                " const &b) { return (a == b) || (a && b && *a == *b); })";

            compare_op += "(" + equal_length + " && " + elements_equal + ")";
          } else if (field.value.type.base_type == BASE_TYPE_ARRAY) {
            compare_op += "(*" + lhs_accessor + " == *" + rhs_accessor + ")";
          } else {
            compare_op += "(" + lhs_accessor + " == " + rhs_accessor + ")";
          }
        } else {
          // Deep compare of std::unique_ptr. Null is not equal to empty.
          std::string both_null =
              "(" + lhs_accessor + " == " + rhs_accessor + ")";
          std::string not_null_and_equal = "(lhs." + accessor + " && rhs." +
                                           accessor + " && *lhs." + accessor +
                                           " == *rhs." + accessor + ")";
          compare_op += "(" + both_null + " || " + not_null_and_equal + ")";
        }
      }
    }

    std::string cmp_lhs;
    std::string cmp_rhs;
    if (compare_op.empty()) {
      cmp_lhs = "";
      cmp_rhs = "";
      compare_op = "  return true;";
    } else {
      cmp_lhs = "lhs";
      cmp_rhs = "rhs";
      compare_op = "  return\n      " + compare_op + ";";
    }

    code_.SetValue("CMP_OP", compare_op);
    code_.SetValue("CMP_LHS", cmp_lhs);
    code_.SetValue("CMP_RHS", cmp_rhs);
    code_ += "";
    code_ +=
        "inline bool operator==(const {{NATIVE_NAME}} &{{CMP_LHS}}, const "
        "{{NATIVE_NAME}} &{{CMP_RHS}}) {";
    code_ += "{{CMP_OP}}";
    code_ += "}";

    code_ += "";
    code_ +=
        "inline bool operator!=(const {{NATIVE_NAME}} &lhs, const "
        "{{NATIVE_NAME}} &rhs) {";
    code_ += "    return !(lhs == rhs);";
    code_ += "}";
    code_ += "";
  }

  void GenOperatorNewDelete(const StructDef &struct_def) {
    if (auto native_custom_alloc =
            struct_def.attributes.Lookup("native_custom_alloc")) {
      code_ += "  inline void *operator new (std::size_t count) {";
      code_ += "    return " + native_custom_alloc->constant +
               "<{{NATIVE_NAME}}>().allocate(count / sizeof({{NATIVE_NAME}}));";
      code_ += "  }";
      code_ += "  inline void operator delete (void *ptr) {";
      code_ += "    return " + native_custom_alloc->constant +
               "<{{NATIVE_NAME}}>().deallocate(static_cast<{{NATIVE_NAME}}*>("
               "ptr),1);";
      code_ += "  }";
    }
  }

  void GenNativeTable(const StructDef &struct_def) {
    const auto native_name = NativeName(Name(struct_def), &struct_def, opts_);
    code_.SetValue("STRUCT_NAME", Name(struct_def));
    code_.SetValue("NATIVE_NAME", native_name);

    // Generate a C++ object that can hold an unpacked version of this table.
    code_ += "struct {{NATIVE_NAME}} : public ::flatbuffers::NativeTable {";
    code_ += "  typedef {{STRUCT_NAME}} TableType;";
    GenFullyQualifiedNameGetter(struct_def, native_name);
    for (const auto field : struct_def.fields.vec) { GenMember(*field); }
    GenOperatorNewDelete(struct_def);
    GenDefaultConstructor(struct_def);
    GenCopyMoveCtorAndAssigOpDecls(struct_def);
    code_ += "};";
    code_ += "";
  }

  // Adds a typedef to the binary schema type so one could get the bfbs based
  // on the type at runtime.
  void GenBinarySchemaTypeDef(const StructDef *struct_def) {
    if (struct_def && opts_.binary_schema_gen_embed) {
      code_ += "  typedef " + WrapInNameSpace(*struct_def) +
               "BinarySchema BinarySchema;";
    }
  }

  void GenNativeTablePost(const StructDef &struct_def) {
    if (opts_.gen_compare) {
      const auto native_name = NativeName(Name(struct_def), &struct_def, opts_);
      code_.SetValue("STRUCT_NAME", Name(struct_def));
      code_.SetValue("NATIVE_NAME", native_name);
      GenCompareOperator(struct_def);
      code_ += "";
    }
  }

  // Generate the code to call the appropriate Verify function(s) for a field.
  void GenVerifyCall(const FieldDef &field, const char *prefix) {
    code_.SetValue("PRE", prefix);
    code_.SetValue("NAME", Name(field));
    code_.SetValue("REQUIRED", field.IsRequired() ? "Required" : "");
    code_.SetValue("SIZE", GenTypeSize(field.value.type));
    code_.SetValue("OFFSET", GenFieldOffsetName(field));
    if (IsScalar(field.value.type.base_type) || IsStruct(field.value.type)) {
      code_.SetValue("ALIGN", NumToString(InlineAlignment(field.value.type)));
      code_ +=
          "{{PRE}}VerifyField{{REQUIRED}}<{{SIZE}}>(verifier, "
          "{{OFFSET}}, {{ALIGN}})\\";
    } else {
      code_.SetValue("OFFSET_SIZE", field.offset64 ? "64" : "");
      code_ +=
          "{{PRE}}VerifyOffset{{OFFSET_SIZE}}{{REQUIRED}}(verifier, "
          "{{OFFSET}})\\";
    }

    switch (field.value.type.base_type) {
      case BASE_TYPE_UNION: {
        code_.SetValue("ENUM_NAME", field.value.type.enum_def->name);
        code_.SetValue("SUFFIX", UnionTypeFieldSuffix());
        code_ +=
            "{{PRE}}Verify{{ENUM_NAME}}(verifier, {{NAME}}(), "
            "{{NAME}}{{SUFFIX}}())\\";
        break;
      }
      case BASE_TYPE_STRUCT: {
        if (!field.value.type.struct_def->fixed) {
          code_ += "{{PRE}}verifier.VerifyTable({{NAME}}())\\";
        }
        break;
      }
      case BASE_TYPE_STRING: {
        code_ += "{{PRE}}verifier.VerifyString({{NAME}}())\\";
        break;
      }
      case BASE_TYPE_VECTOR64:
      case BASE_TYPE_VECTOR: {
        code_ += "{{PRE}}verifier.VerifyVector({{NAME}}())\\";

        switch (field.value.type.element) {
          case BASE_TYPE_STRING: {
            code_ += "{{PRE}}verifier.VerifyVectorOfStrings({{NAME}}())\\";
            break;
          }
          case BASE_TYPE_STRUCT: {
            if (!field.value.type.struct_def->fixed) {
              code_ += "{{PRE}}verifier.VerifyVectorOfTables({{NAME}}())\\";
            }
            break;
          }
          case BASE_TYPE_UNION: {
            code_.SetValue("ENUM_NAME", field.value.type.enum_def->name);
            code_ +=
                "{{PRE}}Verify{{ENUM_NAME}}Vector(verifier, {{NAME}}(), "
                "{{NAME}}_type())\\";
            break;
          }
          default: break;
        }

        auto nfn = GetNestedFlatBufferName(field);
        if (!nfn.empty()) {
          code_.SetValue("CPP_NAME", nfn);
          // FIXME: file_identifier.
          code_ +=
              "{{PRE}}verifier.VerifyNestedFlatBuffer<{{CPP_NAME}}>"
              "({{NAME}}(), nullptr)\\";
        } else if (field.flexbuffer) {
          code_ +=
              "{{PRE}}flexbuffers::VerifyNestedFlexBuffer"
              "({{NAME}}(), verifier)\\";
        }
        break;
      }
      default: {
        break;
      }
    }
  }

  void GenComparatorForStruct(const StructDef &struct_def, size_t space_size,
                              const std::string lhs_struct_literal,
                              const std::string rhs_struct_literal) {
    code_.SetValue("LHS_PREFIX", lhs_struct_literal);
    code_.SetValue("RHS_PREFIX", rhs_struct_literal);
    std::string space(space_size, ' ');
    for (const auto &curr_field : struct_def.fields.vec) {
      const auto curr_field_name = Name(*curr_field);
      code_.SetValue("CURR_FIELD_NAME", curr_field_name);
      code_.SetValue("LHS", lhs_struct_literal + "_" + curr_field_name);
      code_.SetValue("RHS", rhs_struct_literal + "_" + curr_field_name);
      const bool is_scalar = IsScalar(curr_field->value.type.base_type);
      const bool is_array = IsArray(curr_field->value.type);
      const bool is_struct = IsStruct(curr_field->value.type);

      // If encouter a key field, call KeyCompareWithValue to compare this
      // field.
      if (curr_field->key) {
        code_ += space +
                 "const auto {{RHS}} = {{RHS_PREFIX}}.{{CURR_FIELD_NAME}}();";
        code_ += space +
                 "const auto {{CURR_FIELD_NAME}}_compare_result = "
                 "{{LHS_PREFIX}}.KeyCompareWithValue({{RHS}});";

        code_ += space + "if ({{CURR_FIELD_NAME}}_compare_result != 0)";
        code_ += space + "  return {{CURR_FIELD_NAME}}_compare_result;";
        continue;
      }

      code_ +=
          space + "const auto {{LHS}} = {{LHS_PREFIX}}.{{CURR_FIELD_NAME}}();";
      code_ +=
          space + "const auto {{RHS}} = {{RHS_PREFIX}}.{{CURR_FIELD_NAME}}();";
      if (is_scalar) {
        code_ += space + "if ({{LHS}} != {{RHS}})";
        code_ += space +
                 "  return static_cast<int>({{LHS}} > {{RHS}}) - "
                 "static_cast<int>({{LHS}} < {{RHS}});";
      } else if (is_array) {
        const auto &elem_type = curr_field->value.type.VectorType();
        code_ +=
            space +
            "for (::flatbuffers::uoffset_t i = 0; i < {{LHS}}->size(); i++) {";
        code_ += space + "  const auto {{LHS}}_elem = {{LHS}}->Get(i);";
        code_ += space + "  const auto {{RHS}}_elem = {{RHS}}->Get(i);";
        if (IsScalar(elem_type.base_type)) {
          code_ += space + "  if ({{LHS}}_elem != {{RHS}}_elem)";
          code_ += space +
                   "    return static_cast<int>({{LHS}}_elem > {{RHS}}_elem) - "
                   "static_cast<int>({{LHS}}_elem < {{RHS}}_elem);";
          code_ += space + "}";

        } else if (IsStruct(elem_type)) {
          if (curr_field->key) {
            code_ += space +
                     "const auto {{CURR_FIELD_NAME}}_compare_result = "
                     "{{LHS_PREFIX}}.KeyCompareWithValue({{RHS}});";
            code_ += space + "if ({{CURR_FIELD_NAME}}_compare_result != 0)";
            code_ += space + "  return {{CURR_FIELD_NAME}}_compare_result;";
            continue;
          }
          GenComparatorForStruct(
              *curr_field->value.type.struct_def, space_size + 2,
              code_.GetValue("LHS") + "_elem", code_.GetValue("RHS") + "_elem");

          code_ += space + "}";
        }

      } else if (is_struct) {
        GenComparatorForStruct(*curr_field->value.type.struct_def, space_size,
                               code_.GetValue("LHS"), code_.GetValue("RHS"));
      }
    }
  }

  // Generate CompareWithValue method for a key field.
  void GenKeyFieldMethods(const FieldDef &field) {
    FLATBUFFERS_ASSERT(field.key);
    const bool is_string = IsString(field.value.type);
    const bool is_array = IsArray(field.value.type);
    const bool is_struct = IsStruct(field.value.type);
    // Generate KeyCompareLessThan function
    code_ +=
        "  bool KeyCompareLessThan(const {{STRUCT_NAME}} * const o) const {";
    if (is_string) {
      // use operator< of ::flatbuffers::String
      code_ += "    return *{{FIELD_NAME}}() < *o->{{FIELD_NAME}}();";
    } else if (is_array || is_struct) {
      code_ += "    return KeyCompareWithValue(o->{{FIELD_NAME}}()) < 0;";
    } else {
      code_ += "    return {{FIELD_NAME}}() < o->{{FIELD_NAME}}();";
    }
    code_ += "  }";

    // Generate KeyCompareWithValue function
    if (is_string) {
      // Compares key against a null-terminated char array.
      code_ += "  int KeyCompareWithValue(const char *_{{FIELD_NAME}}) const {";
      code_ += "    return strcmp({{FIELD_NAME}}()->c_str(), _{{FIELD_NAME}});";
      code_ += "  }";
      // Compares key against any string-like object (e.g. std::string_view or
      // std::string) that implements operator< comparison with const char*.
      code_ += "  template<typename StringType>";
      code_ +=
          "  int KeyCompareWithValue(const StringType& _{{FIELD_NAME}}) const "
          "{";
      code_ +=
          "    if ({{FIELD_NAME}}()->c_str() < _{{FIELD_NAME}}) return -1;";
      code_ += "    if (_{{FIELD_NAME}} < {{FIELD_NAME}}()->c_str()) return 1;";
      code_ += "    return 0;";
    } else if (is_array) {
      const auto &elem_type = field.value.type.VectorType();
      std::string input_type = "::flatbuffers::Array<" +
                               GenTypeGet(elem_type, "", "", "", false) + ", " +
                               NumToString(elem_type.fixed_length) + ">";
      code_.SetValue("INPUT_TYPE", input_type);
      code_ +=
          "  int KeyCompareWithValue(const {{INPUT_TYPE}} *_{{FIELD_NAME}}"
          ") const {";
      code_ +=
          "    const {{INPUT_TYPE}} *curr_{{FIELD_NAME}} = {{FIELD_NAME}}();";
      code_ +=
          "    for (::flatbuffers::uoffset_t i = 0; i < "
          "curr_{{FIELD_NAME}}->size(); i++) {";

      if (IsScalar(elem_type.base_type)) {
        code_ += "      const auto lhs = curr_{{FIELD_NAME}}->Get(i);";
        code_ += "      const auto rhs = _{{FIELD_NAME}}->Get(i);";
        code_ += "      if (lhs != rhs)";
        code_ +=
            "        return static_cast<int>(lhs > rhs)"
            " - static_cast<int>(lhs < rhs);";
      } else if (IsStruct(elem_type)) {
        code_ +=
            "      const auto &lhs_{{FIELD_NAME}} = "
            "*(curr_{{FIELD_NAME}}->Get(i));";
        code_ +=
            "      const auto &rhs_{{FIELD_NAME}} = "
            "*(_{{FIELD_NAME}}->Get(i));";
        GenComparatorForStruct(*elem_type.struct_def, 6,
                               "lhs_" + code_.GetValue("FIELD_NAME"),
                               "rhs_" + code_.GetValue("FIELD_NAME"));
      }
      code_ += "    }";
      code_ += "    return 0;";
    } else if (is_struct) {
      const auto *struct_def = field.value.type.struct_def;
      code_.SetValue("INPUT_TYPE",
                     GenTypeGet(field.value.type, "", "", "", false));
      code_ +=
          "  int KeyCompareWithValue(const {{INPUT_TYPE}} &_{{FIELD_NAME}}) "
          "const {";
      code_ += "    const auto &lhs_{{FIELD_NAME}} = {{FIELD_NAME}}();";
      code_ += "    const auto &rhs_{{FIELD_NAME}} = _{{FIELD_NAME}};";
      GenComparatorForStruct(*struct_def, 4,
                             "lhs_" + code_.GetValue("FIELD_NAME"),
                             "rhs_" + code_.GetValue("FIELD_NAME"));
      code_ += "    return 0;";

    } else {
      FLATBUFFERS_ASSERT(IsScalar(field.value.type.base_type));
      auto type = GenTypeBasic(field.value.type, false);
      if (opts_.scoped_enums && field.value.type.enum_def &&
          IsScalar(field.value.type.base_type)) {
        type = GenTypeGet(field.value.type, " ", "const ", " *", true);
      }
      // Returns {field<val: -1, field==val: 0, field>val: +1}.
      code_.SetValue("KEY_TYPE", type);
      code_ +=
          "  int KeyCompareWithValue({{KEY_TYPE}} _{{FIELD_NAME}}) const {";
      code_ +=
          "    return static_cast<int>({{FIELD_NAME}}() > _{{FIELD_NAME}}) - "
          "static_cast<int>({{FIELD_NAME}}() < _{{FIELD_NAME}});";
    }
    code_ += "  }";
  }

  void GenTableUnionAsGetters(const FieldDef &field) {
    const auto &type = field.value.type;
    auto u = type.enum_def;

    if (!type.enum_def->uses_multiple_type_instances)
      code_ +=
          "  template<typename T> "
          "const T *{{NULLABLE_EXT}}{{FIELD_NAME}}_as() const;";

    for (auto u_it = u->Vals().begin(); u_it != u->Vals().end(); ++u_it) {
      auto &ev = **u_it;
      if (ev.union_type.base_type == BASE_TYPE_NONE) { continue; }
      auto full_struct_name = GetUnionElement(ev, false, opts_);

      // @TODO: Mby make this decisions more universal? How?
      code_.SetValue("U_GET_TYPE",
                     EscapeKeyword(Name(field) + UnionTypeFieldSuffix()));
      code_.SetValue("U_ELEMENT_TYPE", WrapInNameSpace(u->defined_namespace,
                                                       GetEnumValUse(*u, ev)));
      code_.SetValue("U_FIELD_TYPE", "const " + full_struct_name + " *");
      code_.SetValue("U_FIELD_NAME", Name(field) + "_as_" + Name(ev));
      code_.SetValue("U_NULLABLE", NullableExtension());

      // `const Type *union_name_asType() const` accessor.
      code_ += "  {{U_FIELD_TYPE}}{{U_NULLABLE}}{{U_FIELD_NAME}}() const {";
      code_ +=
          "    return {{U_GET_TYPE}}() == {{U_ELEMENT_TYPE}} ? "
          "static_cast<{{U_FIELD_TYPE}}>({{FIELD_NAME}}()) "
          ": nullptr;";
      code_ += "  }";
    }
  }

  void GenTableFieldGetter(const FieldDef &field) {
    const auto &type = field.value.type;
    const auto offset_str = GenFieldOffsetName(field);

    GenComment(field.doc_comment, "  ");
    // Call a different accessor for pointers, that indirects.
    if (!field.IsScalarOptional()) {
      const bool is_scalar = IsScalar(type.base_type);
      std::string accessor;
      std::string offset_size = "";
      if (is_scalar) {
        accessor = "GetField<";
      } else if (IsStruct(type)) {
        accessor = "GetStruct<";
      } else {
        if (field.offset64) {
          accessor = "GetPointer64<";
        } else {
          accessor = "GetPointer<";
        }
      }
      auto offset_type = GenTypeGet(type, "", "const ", " *", false);
      auto call = accessor + offset_type + ">(" + offset_str;
      // Default value as second arg for non-pointer types.
      if (is_scalar) { call += ", " + GenDefaultConstant(field); }
      call += ")";

      std::string afterptr = " *" + NullableExtension();
      code_.SetValue("FIELD_TYPE",
                     GenTypeGet(type, " ", "const ", afterptr.c_str(), true));
      code_.SetValue("FIELD_VALUE", GenUnderlyingCast(field, true, call));
      code_.SetValue("NULLABLE_EXT", NullableExtension());
      code_ += "  {{FIELD_TYPE}}{{FIELD_NAME}}() const {";
      code_ += "    return {{FIELD_VALUE}};";
      code_ += "  }";
    } else {
      auto wire_type = GenTypeBasic(type, false);
      auto face_type = GenTypeBasic(type, true);
      auto opt_value = "GetOptional<" + wire_type + ", " + face_type + ">(" +
                       offset_str + ")";
      code_.SetValue("FIELD_TYPE", GenOptionalDecl(type));
      code_ += "  {{FIELD_TYPE}} {{FIELD_NAME}}() const {";
      code_ += "    return " + opt_value + ";";
      code_ += "  }";
    }

    if (type.base_type == BASE_TYPE_UNION) { GenTableUnionAsGetters(field); }
  }

  void GenTableFieldType(const FieldDef &field) {
    const auto &type = field.value.type;
    const auto offset_str = GenFieldOffsetName(field);
    if (!field.IsScalarOptional()) {
      std::string afterptr = " *" + NullableExtension();
      code_.SetValue("FIELD_TYPE",
                     GenTypeGet(type, "", "const ", afterptr.c_str(), true));
      code_ += "    {{FIELD_TYPE}}\\";
    } else {
      code_.SetValue("FIELD_TYPE", GenOptionalDecl(type));
      code_ += "    {{FIELD_TYPE}}\\";
    }
  }

  void GenStructFieldType(const FieldDef &field) {
    const auto is_array = IsArray(field.value.type);
    std::string field_type =
        GenTypeGet(field.value.type, "", is_array ? "" : "const ",
                   is_array ? "" : " &", true);
    code_.SetValue("FIELD_TYPE", field_type);
    code_ += "    {{FIELD_TYPE}}\\";
  }

  void GenFieldTypeHelper(const StructDef &struct_def) {
    if (struct_def.fields.vec.empty()) { return; }
    code_ += "  template<size_t Index>";
    code_ += "  using FieldType = \\";
    code_ += "decltype(std::declval<type>().get_field<Index>());";
  }

  void GenIndexBasedFieldGetter(const StructDef &struct_def) {
    if (struct_def.fields.vec.empty()) { return; }
    code_ += "  template<size_t Index>";
    code_ += "  auto get_field() const {";

    size_t index = 0;
    bool need_else = false;
    // Generate one index-based getter for each field.
    for (const auto &field : struct_def.fields.vec) {
      if (field->deprecated) {
        // Deprecated fields won't be accessible.
        continue;
      }
      code_.SetValue("FIELD_NAME", Name(*field));
      code_.SetValue("FIELD_INDEX",
                     std::to_string(static_cast<long long>(index++)));
      if (need_else) {
        code_ += "    else \\";
      } else {
        code_ += "         \\";
      }
      need_else = true;
      code_ += "if constexpr (Index == {{FIELD_INDEX}}) \\";
      code_ += "return {{FIELD_NAME}}();";
    }
    code_ += "    else static_assert(Index != Index, \"Invalid Field Index\");";
    code_ += "  }";
  }

  // Sample for Vec3:
  //
  //   static constexpr std::array<const char *, 3> field_names = {
  //     "x",
  //     "y",
  //     "z"
  //   };
  //
  void GenFieldNames(const StructDef &struct_def) {
    code_ += "  static constexpr std::array<\\";
    code_ += "const char *, fields_number> field_names = {\\";
    if (struct_def.fields.vec.empty()) {
      code_ += "};";
      return;
    }
    code_ += "";
    // Generate the field_names elements.
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) {
        // Deprecated fields won't be accessible.
        continue;
      }
      code_.SetValue("FIELD_NAME", Name(field));
      code_ += R"(    "{{FIELD_NAME}}"\)";
      if (it + 1 != struct_def.fields.vec.end()) { code_ += ","; }
    }
    code_ += "\n  };";
  }

  void GenFieldsNumber(const StructDef &struct_def) {
    const auto non_deprecated_field_count = std::count_if(
        struct_def.fields.vec.begin(), struct_def.fields.vec.end(),
        [](const FieldDef *field) { return !field->deprecated; });
    code_.SetValue(
        "FIELD_COUNT",
        std::to_string(static_cast<long long>(non_deprecated_field_count)));
    code_ += "  static constexpr size_t fields_number = {{FIELD_COUNT}};";
  }

  void GenTraitsStruct(const StructDef &struct_def) {
    code_.SetValue(
        "FULLY_QUALIFIED_NAME",
        struct_def.defined_namespace->GetFullyQualifiedName(Name(struct_def)));
    code_ += "struct {{STRUCT_NAME}}::Traits {";
    code_ += "  using type = {{STRUCT_NAME}};";
    if (!struct_def.fixed) {
      // We have a table and not a struct.
      code_ += "  static auto constexpr Create = Create{{STRUCT_NAME}};";
    }
    if (opts_.cpp_static_reflection) {
      code_ += "  static constexpr auto name = \"{{STRUCT_NAME}}\";";
      code_ +=
          "  static constexpr auto fully_qualified_name = "
          "\"{{FULLY_QUALIFIED_NAME}}\";";
      GenFieldsNumber(struct_def);
      GenFieldNames(struct_def);
      GenFieldTypeHelper(struct_def);
    }
    code_ += "};";
    code_ += "";
  }

  void GenTableFieldSetter(const FieldDef &field) {
    const auto &type = field.value.type;
    const bool is_scalar = IsScalar(type.base_type);
    if (is_scalar && IsUnion(type))
      return;  // changing of a union's type is forbidden

    auto offset_str = GenFieldOffsetName(field);
    if (is_scalar) {
      const auto wire_type = GenTypeWire(type, "", false, field.offset64);
      code_.SetValue("SET_FN", "SetField<" + wire_type + ">");
      code_.SetValue("OFFSET_NAME", offset_str);
      code_.SetValue("FIELD_TYPE", GenTypeBasic(type, true));
      code_.SetValue("FIELD_VALUE",
                     GenUnderlyingCast(field, false, "_" + Name(field)));

      code_ += "  bool mutate_{{FIELD_NAME}}({{FIELD_TYPE}} _{{FIELD_NAME}}\\";
      if (!field.IsScalarOptional()) {
        code_.SetValue("DEFAULT_VALUE", GenDefaultConstant(field));
        code_.SetValue(
            "INTERFACE_DEFAULT_VALUE",
            GenUnderlyingCast(field, true, GenDefaultConstant(field)));

        // GenUnderlyingCast for a bool field generates 0 != 0
        // So the type has to be checked and the appropriate default chosen
        if (IsBool(field.value.type.base_type)) {
          code_ += " = {{DEFAULT_VALUE}}) {";
        } else {
          code_ += " = {{INTERFACE_DEFAULT_VALUE}}) {";
        }
        code_ +=
            "    return {{SET_FN}}({{OFFSET_NAME}}, {{FIELD_VALUE}}, "
            "{{DEFAULT_VALUE}});";
      } else {
        code_ += ") {";
        code_ += "    return {{SET_FN}}({{OFFSET_NAME}}, {{FIELD_VALUE}});";
      }
      code_ += "  }";
    } else {
      auto postptr = " *" + NullableExtension();
      auto wire_type = GenTypeGet(type, " ", "", postptr.c_str(), true);
      const std::string accessor = [&]() {
        if (IsStruct(type)) { return "GetStruct<"; }
        if (field.offset64) { return "GetPointer64<"; }
        return "GetPointer<";
      }();
      auto underlying = accessor + wire_type + ">(" + offset_str + ")";
      code_.SetValue("FIELD_TYPE", wire_type);
      code_.SetValue("FIELD_VALUE", GenUnderlyingCast(field, true, underlying));

      code_ += "  {{FIELD_TYPE}}mutable_{{FIELD_NAME}}() {";
      code_ += "    return {{FIELD_VALUE}};";
      code_ += "  }";
    }
  }

  std::string GetNestedFlatBufferName(const FieldDef &field) {
    auto nested = field.attributes.Lookup("nested_flatbuffer");
    if (!nested) return "";
    std::string qualified_name = nested->constant;
    auto nested_root = parser_.LookupStruct(nested->constant);
    if (nested_root == nullptr) {
      qualified_name =
          parser_.current_namespace_->GetFullyQualifiedName(nested->constant);
      nested_root = parser_.LookupStruct(qualified_name);
    }
    FLATBUFFERS_ASSERT(nested_root);  // Guaranteed to exist by parser.
    (void)nested_root;
    return TranslateNameSpace(qualified_name);
  }

  // Generate an accessor struct, builder structs & function for a table.
  void GenTable(const StructDef &struct_def) {
    if (opts_.generate_object_based_api) { GenNativeTable(struct_def); }

    // Generate an accessor struct, with methods of the form:
    // type name() const { return GetField<type>(offset, defaultval); }
    GenComment(struct_def.doc_comment);

    code_.SetValue("STRUCT_NAME", Name(struct_def));
    code_ +=
        "struct {{STRUCT_NAME}} FLATBUFFERS_FINAL_CLASS"
        " : private ::flatbuffers::Table {";
    if (opts_.generate_object_based_api) {
      code_ += "  typedef {{NATIVE_NAME}} NativeTableType;";
    }
    code_ += "  typedef {{STRUCT_NAME}}Builder Builder;";
    GenBinarySchemaTypeDef(parser_.root_struct_def_);

    if (opts_.g_cpp_std >= cpp::CPP_STD_17) { code_ += "  struct Traits;"; }
    if (opts_.mini_reflect != IDLOptions::kNone) {
      code_ +=
          "  static const ::flatbuffers::TypeTable *MiniReflectTypeTable() {";
      code_ += "    return {{STRUCT_NAME}}TypeTable();";
      code_ += "  }";
    }

    GenFullyQualifiedNameGetter(struct_def, Name(struct_def));

    // Generate field id constants.
    if (!struct_def.fields.vec.empty()) {
      // We need to add a trailing comma to all elements except the last one as
      // older versions of gcc complain about this.
      code_.SetValue("SEP", "");
      code_ +=
          "  enum FlatBuffersVTableOffset FLATBUFFERS_VTABLE_UNDERLYING_TYPE {";
      for (const auto &field : struct_def.fields.vec) {
        if (field->deprecated) {
          // Deprecated fields won't be accessible.
          continue;
        }

        code_.SetValue("OFFSET_NAME", GenFieldOffsetName(*field));
        code_.SetValue("OFFSET_VALUE", NumToString(field->value.offset));
        code_ += "{{SEP}}    {{OFFSET_NAME}} = {{OFFSET_VALUE}}\\";
        code_.SetValue("SEP", ",\n");
      }
      code_ += "";
      code_ += "  };";
    }

    // Generate the accessors.
    for (const auto &field : struct_def.fields.vec) {
      if (field->deprecated) {
        // Deprecated fields won't be accessible.
        continue;
      }

      code_.SetValue("FIELD_NAME", Name(*field));
      GenTableFieldGetter(*field);
      if (opts_.mutable_buffer) { GenTableFieldSetter(*field); }

      auto nfn = GetNestedFlatBufferName(*field);
      if (!nfn.empty()) {
        code_.SetValue("CPP_NAME", nfn);
        code_ += "  const {{CPP_NAME}} *{{FIELD_NAME}}_nested_root() const {";
        code_ += "    const auto _f = {{FIELD_NAME}}();";
        code_ +=
            "    return _f ? ::flatbuffers::GetRoot<{{CPP_NAME}}>(_f->Data())";
        code_ += "              : nullptr;";
        code_ += "  }";
      }

      if (field->flexbuffer) {
        code_ +=
            "  flexbuffers::Reference {{FIELD_NAME}}_flexbuffer_root()"
            " const {";
        // Both Data() and size() are const-methods, therefore call order
        // doesn't matter.
        code_ += "    const auto _f = {{FIELD_NAME}}();";
        code_ += "    return _f ? flexbuffers::GetRoot(_f->Data(), _f->size())";
        code_ += "              : flexbuffers::Reference();";
        code_ += "  }";
      }

      // Generate a comparison function for this field if it is a key.
      if (field->key) { GenKeyFieldMethods(*field); }
    }

    if (opts_.cpp_static_reflection) { GenIndexBasedFieldGetter(struct_def); }

    // Generate a verifier function that can check a buffer from an untrusted
    // source will never cause reads outside the buffer.
    code_ += "  bool Verify(::flatbuffers::Verifier &verifier) const {";
    code_ += "    return VerifyTableStart(verifier)\\";
    for (const auto &field : struct_def.fields.vec) {
      if (field->deprecated) { continue; }
      GenVerifyCall(*field, " &&\n           ");
    }

    code_ += " &&\n           verifier.EndTable();";
    code_ += "  }";

    if (opts_.generate_object_based_api) {
      // Generate the UnPack() pre declaration.
      code_ += "  " + TableUnPackSignature(struct_def, true, opts_) + ";";
      code_ += "  " + TableUnPackToSignature(struct_def, true, opts_) + ";";
      code_ += "  " + TablePackSignature(struct_def, true, opts_) + ";";
    }

    code_ += "};";  // End of table.
    code_ += "";

    // Explicit specializations for union accessors
    for (const auto &field : struct_def.fields.vec) {
      if (field->deprecated || field->value.type.base_type != BASE_TYPE_UNION) {
        continue;
      }

      auto u = field->value.type.enum_def;
      if (u->uses_multiple_type_instances) continue;

      code_.SetValue("FIELD_NAME", Name(*field));

      for (auto u_it = u->Vals().begin(); u_it != u->Vals().end(); ++u_it) {
        auto &ev = **u_it;
        if (ev.union_type.base_type == BASE_TYPE_NONE) { continue; }

        auto full_struct_name = GetUnionElement(ev, false, opts_);

        code_.SetValue(
            "U_ELEMENT_TYPE",
            WrapInNameSpace(u->defined_namespace, GetEnumValUse(*u, ev)));
        code_.SetValue("U_FIELD_TYPE", "const " + full_struct_name + " *");
        code_.SetValue("U_ELEMENT_NAME", full_struct_name);
        code_.SetValue("U_FIELD_NAME", Name(*field) + "_as_" + Name(ev));

        // `template<> const T *union_name_as<T>() const` accessor.
        code_ +=
            "template<> "
            "inline {{U_FIELD_TYPE}}{{STRUCT_NAME}}::{{FIELD_NAME}}_as"
            "<{{U_ELEMENT_NAME}}>() const {";
        code_ += "  return {{U_FIELD_NAME}}();";
        code_ += "}";
        code_ += "";
      }
    }

    GenBuilders(struct_def);

    if (opts_.generate_object_based_api) {
      // Generate a pre-declaration for a CreateX method that works with an
      // unpacked C++ object.
      code_ += TableCreateSignature(struct_def, true, opts_) + ";";
      code_ += "";
    }
  }

  // Generate code to force vector alignment. Return empty string for vector
  // that doesn't need alignment code.
  std::string GenVectorForceAlign(const FieldDef &field,
                                  const std::string &field_size) {
    FLATBUFFERS_ASSERT(IsVector(field.value.type));
    // Get the value of the force_align attribute.
    const auto *force_align = field.attributes.Lookup("force_align");
    const int align = force_align ? atoi(force_align->constant.c_str()) : 1;
    // Generate code to do force_align for the vector.
    if (align > 1) {
      const auto vtype = field.value.type.VectorType();
      const std::string &type =
          IsStruct(vtype) ? WrapInNameSpace(*vtype.struct_def)
                          : GenTypeWire(vtype, "", false, field.offset64);
      return std::string("_fbb.ForceVectorAlignment") +
             (field.offset64 ? "64" : "") + "(" + field_size + ", sizeof(" +
             type + "), " + std::to_string(static_cast<long long>(align)) +
             ");";
    }
    return "";
  }

  void GenBuilders(const StructDef &struct_def) {
    code_.SetValue("STRUCT_NAME", Name(struct_def));

    // Generate a builder struct:
    code_ += "struct {{STRUCT_NAME}}Builder {";
    code_ += "  typedef {{STRUCT_NAME}} Table;";
    code_ += "  " + GetBuilder() + " &fbb_;";
    code_ += "  ::flatbuffers::uoffset_t start_;";

    bool has_string_or_vector_fields = false;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) continue;
      const bool is_scalar = IsScalar(field.value.type.base_type);
      const bool is_default_scalar = is_scalar && !field.IsScalarOptional();
      const bool is_string = IsString(field.value.type);
      const bool is_vector = IsVector(field.value.type);
      if (is_string || is_vector) { has_string_or_vector_fields = true; }

      std::string offset = GenFieldOffsetName(field);
      std::string name = GenUnderlyingCast(field, false, Name(field));
      std::string value = is_default_scalar ? GenDefaultConstant(field) : "";

      // Generate accessor functions of the form:
      // void add_name(type name) {
      //   fbb_.AddElement<type>(offset, name, default);
      // }
      code_.SetValue("FIELD_NAME", Name(field));
      code_.SetValue("FIELD_TYPE",
                     GenTypeWire(field.value.type, " ", true, field.offset64));
      code_.SetValue("ADD_OFFSET", Name(struct_def) + "::" + offset);
      code_.SetValue("ADD_NAME", name);
      code_.SetValue("ADD_VALUE", value);
      if (is_scalar) {
        const auto type =
            GenTypeWire(field.value.type, "", false, field.offset64);
        code_.SetValue("ADD_FN", "AddElement<" + type + ">");
      } else if (IsStruct(field.value.type)) {
        code_.SetValue("ADD_FN", "AddStruct");
      } else {
        code_.SetValue("ADD_FN", "AddOffset");
      }

      code_ += "  void add_{{FIELD_NAME}}({{FIELD_TYPE}}{{FIELD_NAME}}) {";
      code_ += "    fbb_.{{ADD_FN}}(\\";
      if (is_default_scalar) {
        code_ += "{{ADD_OFFSET}}, {{ADD_NAME}}, {{ADD_VALUE}});";
      } else {
        code_ += "{{ADD_OFFSET}}, {{ADD_NAME}});";
      }
      code_ += "  }";
    }

    // Builder constructor
    code_ += "  explicit {{STRUCT_NAME}}Builder(" + GetBuilder() +
             " "
             "&_fbb)";
    code_ += "        : fbb_(_fbb) {";
    code_ += "    start_ = fbb_.StartTable();";
    code_ += "  }";

    // Finish() function.
    code_ += "  ::flatbuffers::Offset<{{STRUCT_NAME}}> Finish() {";
    code_ += "    const auto end = fbb_.EndTable(start_);";
    code_ += "    auto o = ::flatbuffers::Offset<{{STRUCT_NAME}}>(end);";

    for (const auto &field : struct_def.fields.vec) {
      if (!field->deprecated && field->IsRequired()) {
        code_.SetValue("FIELD_NAME", Name(*field));
        code_.SetValue("OFFSET_NAME", GenFieldOffsetName(*field));
        code_ += "    fbb_.Required(o, {{STRUCT_NAME}}::{{OFFSET_NAME}});";
      }
    }
    code_ += "    return o;";
    code_ += "  }";
    code_ += "};";
    code_ += "";

    // Generate a convenient CreateX function that uses the above builder
    // to create a table in one go.
    code_ +=
        "inline ::flatbuffers::Offset<{{STRUCT_NAME}}> "
        "Create{{STRUCT_NAME}}(";
    code_ += "    " + GetBuilder() + " &_fbb\\";
    for (const auto &field : struct_def.fields.vec) {
      if (!field->deprecated) { GenParam(*field, false, ",\n    "); }
    }
    code_ += ") {";

    code_ += "  {{STRUCT_NAME}}Builder builder_(_fbb);";
    for (size_t size = struct_def.sortbysize ? sizeof(largest_scalar_t) : 1;
         size; size /= 2) {
      for (auto it = struct_def.fields.vec.rbegin();
           it != struct_def.fields.vec.rend(); ++it) {
        const auto &field = **it;
        if (!field.deprecated && (!struct_def.sortbysize ||
                                  size == SizeOf(field.value.type.base_type))) {
          code_.SetValue("FIELD_NAME", Name(field));
          if (field.IsScalarOptional()) {
            code_ +=
                "  if({{FIELD_NAME}}) { "
                "builder_.add_{{FIELD_NAME}}(*{{FIELD_NAME}}); }";
          } else {
            code_ += "  builder_.add_{{FIELD_NAME}}({{FIELD_NAME}});";
          }
        }
      }
    }
    code_ += "  return builder_.Finish();";
    code_ += "}";
    code_ += "";

    // Definition for type traits for this table type. This allows querying var-
    // ious compile-time traits of the table.
    if (opts_.g_cpp_std >= cpp::CPP_STD_17) { GenTraitsStruct(struct_def); }

    // Generate a CreateXDirect function with vector types as parameters
    if (opts_.cpp_direct_copy && has_string_or_vector_fields) {
      code_ +=
          "inline ::flatbuffers::Offset<{{STRUCT_NAME}}> "
          "Create{{STRUCT_NAME}}Direct(";
      code_ += "    " + GetBuilder() + " &_fbb\\";
      for (const auto &field : struct_def.fields.vec) {
        if (!field->deprecated) { GenParam(*field, true, ",\n    "); }
      }
      // Need to call "Create" with the struct namespace.
      const auto qualified_create_name =
          struct_def.defined_namespace->GetFullyQualifiedName("Create");
      code_.SetValue("CREATE_NAME", TranslateNameSpace(qualified_create_name));
      code_ += ") {";
      // Offset64 bit fields need to be added to the buffer first, so here we
      // loop over the fields in order of their offset size, followed by their
      // definition order. Otherwise the emitted code might add a Offset
      // followed by an Offset64 which would trigger an assertion.

      // TODO(derekbailey): maybe optimize for the case where there is no
      // 64offsets in the whole schema?
      ForAllFieldsOrderedByOffset(struct_def, [&](const FieldDef *field) {
        if (field->deprecated) { return; }
        code_.SetValue("FIELD_NAME", Name(*field));
        if (IsString(field->value.type)) {
          if (!field->shared) {
            code_.SetValue(
                "CREATE_STRING",
                "CreateString" + std::string(field->offset64
                                                 ? "<::flatbuffers::Offset64>"
                                                 : ""));
          } else {
            code_.SetValue("CREATE_STRING", "CreateSharedString");
          }
          code_ +=
              "  auto {{FIELD_NAME}}__ = {{FIELD_NAME}} ? "
              "_fbb.{{CREATE_STRING}}({{FIELD_NAME}}) : 0;";
        } else if (IsVector(field->value.type)) {
          const std::string force_align_code =
              GenVectorForceAlign(*field, Name(*field) + "->size()");
          if (!force_align_code.empty()) {
            code_ += "  if ({{FIELD_NAME}}) { " + force_align_code + " }";
          }
          code_ += "  auto {{FIELD_NAME}}__ = {{FIELD_NAME}} ? \\";
          const auto vtype = field->value.type.VectorType();
          const auto has_key = TypeHasKey(vtype);
          if (IsStruct(vtype)) {
            const std::string type = WrapInNameSpace(*vtype.struct_def);
            if (has_key) {
              code_ += "_fbb.CreateVectorOfSortedStructs<" + type + ">\\";
            } else {
              // If the field uses 64-bit addressing, create a 64-bit vector.
              if (field->value.type.base_type == BASE_TYPE_VECTOR64) {
                code_ += "_fbb.CreateVectorOfStructs64\\";
              } else {
                code_ += "_fbb.CreateVectorOfStructs\\";
                if (field->offset64) {
                  // This is normal 32-bit vector, with 64-bit addressing.
                  code_ += "64<::flatbuffers::Vector>\\";
                } else {
                  code_ += "<" + type + ">\\";
                }
              }
            }
          } else if (has_key) {
            const auto type = WrapInNameSpace(*vtype.struct_def);
            code_ += "_fbb.CreateVectorOfSortedTables<" + type + ">\\";
          } else {
            const auto type = GenTypeWire(
                vtype, "", VectorElementUserFacing(vtype), field->offset64);

            if (field->value.type.base_type == BASE_TYPE_VECTOR64) {
              code_ += "_fbb.CreateVector64\\";
            } else {
              // If the field uses 64-bit addressing, create a 64-bit vector.
              code_.SetValue("64OFFSET", field->offset64 ? "64" : "");
              code_.SetValue("TYPE",
                             field->offset64 ? "::flatbuffers::Vector" : type);

              code_ += "_fbb.CreateVector{{64OFFSET}}<{{TYPE}}>\\";
            }
          }
          code_ += has_key ? "({{FIELD_NAME}}) : 0;" : "(*{{FIELD_NAME}}) : 0;";
        }
      });
      code_ += "  return {{CREATE_NAME}}{{STRUCT_NAME}}(";
      code_ += "      _fbb\\";
      for (const auto &field : struct_def.fields.vec) {
        if (field->deprecated) { continue; }
        code_.SetValue("FIELD_NAME", Name(*field));
        code_ += ",\n      {{FIELD_NAME}}\\";
        if (IsString(field->value.type) || IsVector(field->value.type)) {
          code_ += "__\\";
        }
      }
      code_ += ");";
      code_ += "}";
      code_ += "";
    }
  }

  std::string GenUnionUnpackVal(const FieldDef &afield,
                                const char *vec_elem_access,
                                const char *vec_type_access) {
    auto type_name = WrapInNameSpace(*afield.value.type.enum_def);
    return type_name + "Union::UnPack(" + "_e" + vec_elem_access + ", " +
           EscapeKeyword(afield.name + UnionTypeFieldSuffix()) + "()" +
           vec_type_access + ", _resolver)";
  }

  std::string GenUnpackVal(const Type &type, const std::string &val,
                           bool invector, const FieldDef &afield) {
    switch (type.base_type) {
      case BASE_TYPE_STRING: {
        if (FlexibleStringConstructor(&afield)) {
          return NativeString(&afield) + "(" + val + "->c_str(), " + val +
                 "->size())";
        } else {
          return val + "->str()";
        }
      }
      case BASE_TYPE_STRUCT: {
        if (IsStruct(type)) {
          const auto &struct_attrs = type.struct_def->attributes;
          const auto native_type = struct_attrs.Lookup("native_type");
          if (native_type) {
            std::string unpack_call = "::flatbuffers::UnPack";
            const auto pack_name = struct_attrs.Lookup("native_type_pack_name");
            if (pack_name) { unpack_call += pack_name->constant; }
            unpack_call += "(*" + val + ")";
            return unpack_call;
          } else if (invector || afield.native_inline) {
            return "*" + val;
          } else {
            const auto name = WrapInNameSpace(*type.struct_def);
            const auto ptype = GenTypeNativePtr(name, &afield, true);
            return ptype + "(new " + name + "(*" + val + "))";
          }
        } else {
          std::string ptype = afield.native_inline ? "*" : "";
          ptype += GenTypeNativePtr(
              WrapNativeNameInNameSpace(*type.struct_def, opts_), &afield,
              true);
          return ptype + "(" + val + "->UnPack(_resolver))";
        }
      }
      case BASE_TYPE_UNION: {
        return GenUnionUnpackVal(
            afield, invector ? "->Get(_i)" : "",
            invector ? ("->GetEnum<" + type.enum_def->name + ">(_i)").c_str()
                     : "");
      }
      default: {
        return val;
        break;
      }
    }
  }

  std::string GenUnpackFieldStatement(const FieldDef &field,
                                      const FieldDef *union_field) {
    std::string code;
    switch (field.value.type.base_type) {
      case BASE_TYPE_VECTOR64:
      case BASE_TYPE_VECTOR: {
        auto name = Name(field);
        if (field.value.type.element == BASE_TYPE_UTYPE) {
          name = StripUnionType(Name(field));
        }
        const std::string vector_field = "_o->" + name;
        code += "{ " + vector_field + ".resize(_e->size()); ";
        if (!field.value.type.enum_def && !IsBool(field.value.type.element) &&
            IsOneByte(field.value.type.element)) {
          // For vectors of bytes, std::copy is used to improve performance.
          // This doesn't work for:
          //  - enum types because they have to be explicitly static_cast.
          //  - vectors of bool, since they are a template specialization.
          //  - multiple-byte types due to endianness.
          code +=
              "std::copy(_e->begin(), _e->end(), _o->" + name + ".begin()); }";
        } else {
          std::string indexing;
          if (field.value.type.enum_def) {
            indexing += "static_cast<" +
                        WrapInNameSpace(*field.value.type.enum_def) + ">(";
          }
          indexing += "_e->Get(_i)";
          if (field.value.type.enum_def) { indexing += ")"; }
          if (field.value.type.element == BASE_TYPE_BOOL) {
            indexing += " != 0";
          }
          // Generate code that pushes data from _e to _o in the form:
          //   for (uoffset_t i = 0; i < _e->size(); ++i) {
          //     _o->field.push_back(_e->Get(_i));
          //   }
          auto access =
              field.value.type.element == BASE_TYPE_UTYPE
                  ? ".type"
                  : (field.value.type.element == BASE_TYPE_UNION ? ".value"
                                                                 : "");
          if (field.value.type.base_type == BASE_TYPE_VECTOR64) {
            code += "for (::flatbuffers::uoffset64_t _i = 0;";
          } else {
            code += "for (::flatbuffers::uoffset_t _i = 0;";
          }
          code += " _i < _e->size(); _i++) { ";
          auto cpp_type = field.attributes.Lookup("cpp_type");
          if (cpp_type) {
            // Generate code that resolves the cpp pointer type, of the form:
            //  if (resolver)
            //    (*resolver)(&_o->field, (hash_value_t)(_e));
            //  else
            //    _o->field = nullptr;
            code += "/*vector resolver, " + PtrType(&field) + "*/ ";
            code += "if (_resolver) ";
            code += "(*_resolver)";
            code += "(reinterpret_cast<void **>(&_o->" + name + "[_i]" +
                    access + "), ";
            code +=
                "static_cast<::flatbuffers::hash_value_t>(" + indexing + "));";
            if (PtrType(&field) == "naked") {
              code += " else ";
              code += "_o->" + name + "[_i]" + access + " = nullptr";
            } else {
              // code += " else ";
              // code += "_o->" + name + "[_i]" + access + " = " +
              // GenTypeNativePtr(cpp_type->constant, &field, true) + "();";
              code += "/* else do nothing */";
            }
          } else {
            const bool is_pointer = IsVectorOfPointers(field);
            if (is_pointer) {
              code += "if(_o->" + name + "[_i]" + ") { ";
              code += indexing + "->UnPackTo(_o->" + name +
                      "[_i].get(), _resolver);";
              code += " } else { ";
            }
            code += "_o->" + name + "[_i]" + access + " = ";
            code += GenUnpackVal(field.value.type.VectorType(), indexing, true,
                                 field);
            if (is_pointer) { code += "; }"; }
          }
          code += "; } } else { " + vector_field + ".resize(0); }";
        }
        break;
      }
      case BASE_TYPE_UTYPE: {
        FLATBUFFERS_ASSERT(union_field->value.type.base_type ==
                           BASE_TYPE_UNION);
        // Generate code that sets the union type, of the form:
        //   _o->field.type = _e;
        code += "_o->" + union_field->name + ".type = _e;";
        break;
      }
      case BASE_TYPE_UNION: {
        // Generate code that sets the union value, of the form:
        //   _o->field.value = Union::Unpack(_e, field_type(), resolver);
        code += "_o->" + Name(field) + ".value = ";
        code += GenUnionUnpackVal(field, "", "");
        code += ";";
        break;
      }
      default: {
        auto cpp_type = field.attributes.Lookup("cpp_type");
        if (cpp_type) {
          // Generate code that resolves the cpp pointer type, of the form:
          //  if (resolver)
          //    (*resolver)(&_o->field, (hash_value_t)(_e));
          //  else
          //    _o->field = nullptr;
          code += "/*scalar resolver, " + PtrType(&field) + "*/ ";
          code += "if (_resolver) ";
          code += "(*_resolver)";
          code += "(reinterpret_cast<void **>(&_o->" + Name(field) + "), ";
          code += "static_cast<::flatbuffers::hash_value_t>(_e));";
          if (PtrType(&field) == "naked") {
            code += " else ";
            code += "_o->" + Name(field) + " = nullptr;";
          } else {
            // code += " else ";
            // code += "_o->" + Name(field) + " = " +
            // GenTypeNativePtr(cpp_type->constant, &field, true) + "();";
            code += "/* else do nothing */;";
          }
        } else {
          // Generate code for assigning the value, of the form:
          //  _o->field = value;
          const bool is_pointer = IsPointer(field);

          const std::string out_field = "_o->" + Name(field);

          if (is_pointer) {
            code += "{ if(" + out_field + ") { ";
            code += "_e->UnPackTo(" + out_field + ".get(), _resolver);";
            code += " } else { ";
          }
          code += out_field + " = ";
          code += GenUnpackVal(field.value.type, "_e", false, field) + ";";
          if (is_pointer) {
            code += " } } else if (" + out_field + ") { " + out_field +
                    ".reset(); }";
          }
        }
        break;
      }
    }
    return code;
  }

  std::string GenCreateParam(const FieldDef &field) {
    std::string value = "_o->";
    if (field.value.type.base_type == BASE_TYPE_UTYPE) {
      value += StripUnionType(Name(field));
      value += ".type";
    } else {
      value += Name(field);
    }
    if (!IsVector(field.value.type) && field.attributes.Lookup("cpp_type")) {
      auto type = GenTypeBasic(field.value.type, false);
      value =
          "_rehasher ? "
          "static_cast<" +
          type + ">((*_rehasher)(" + value + GenPtrGet(field) + ")) : 0";
    }

    std::string code;
    switch (field.value.type.base_type) {
      // String fields are of the form:
      //   _fbb.CreateString(_o->field)
      // or
      //   _fbb.CreateSharedString(_o->field)
      case BASE_TYPE_STRING: {
        if (!field.shared) {
          code +=
              "_fbb.CreateString" +
              std::string(field.offset64 ? "<::flatbuffers::Offset64>" : "") +
              "(";
        } else {
          code += "_fbb.CreateSharedString(";
        }
        code += value;
        code.push_back(')');

        // For optional fields, check to see if there actually is any data
        // in _o->field before attempting to access it. If there isn't,
        // depending on set_empty_strings_to_null either set it to 0 or an empty
        // string.
        if (!field.IsRequired()) {
          auto empty_value = opts_.set_empty_strings_to_null
                                 ? "0"
                                 : "_fbb.CreateSharedString(\"\")";
          code = value + ".empty() ? " + empty_value + " : " + code;
        }
        break;
      }
        // Vector fields come in several flavours, of the forms:
        //   _fbb.CreateVector(_o->field);
        //   _fbb.CreateVector((const utype*)_o->field.data(),
        //   _o->field.size()); _fbb.CreateVectorOfStrings(_o->field)
        //   _fbb.CreateVectorOfStructs(_o->field)
        //   _fbb.CreateVector<Offset<T>>(_o->field.size() [&](size_t i) {
        //     return CreateT(_fbb, _o->Get(i), rehasher);
        //   });
      case BASE_TYPE_VECTOR64:
      case BASE_TYPE_VECTOR: {
        auto vector_type = field.value.type.VectorType();
        switch (vector_type.base_type) {
          case BASE_TYPE_STRING: {
            if (NativeString(&field) == "std::string") {
              code += "_fbb.CreateVectorOfStrings(" + value + ")";
            } else {
              // Use by-function serialization to emulate
              // CreateVectorOfStrings(); this works also with non-std strings.
              code +=
                  "_fbb.CreateVector<::flatbuffers::Offset<::flatbuffers::"
                  "String>>"
                  " ";
              code += "(" + value + ".size(), ";
              code += "[](size_t i, _VectorArgs *__va) { ";
              code +=
                  "return __va->__fbb->CreateString(__va->_" + value + "[i]);";
              code += " }, &_va )";
            }
            break;
          }
          case BASE_TYPE_STRUCT: {
            if (IsStruct(vector_type)) {
              const auto &struct_attrs =
                  field.value.type.struct_def->attributes;
              const auto native_type = struct_attrs.Lookup("native_type");
              if (native_type) {
                code += "_fbb.CreateVectorOfNativeStructs<";
                code += WrapInNameSpace(*vector_type.struct_def) + ", " +
                        native_type->constant + ">";
                code += "(" + value;
                const auto pack_name =
                    struct_attrs.Lookup("native_type_pack_name");
                if (pack_name) {
                  code += ", ::flatbuffers::Pack" + pack_name->constant;
                }
                code += ")";
              } else {
                // If the field uses 64-bit addressing, create a 64-bit vector.
                if (field.value.type.base_type == BASE_TYPE_VECTOR64) {
                  code += "_fbb.CreateVectorOfStructs64";
                } else {
                  code += "_fbb.CreateVectorOfStructs";
                  if (field.offset64) {
                    // This is normal 32-bit vector, with 64-bit addressing.
                    code += "64<::flatbuffers::Vector>";
                  }
                }
                code += "(" + value + ")";
              }
            } else {
              code += "_fbb.CreateVector<::flatbuffers::Offset<";
              code += WrapInNameSpace(*vector_type.struct_def) + ">> ";
              code += "(" + value + ".size(), ";
              code += "[](size_t i, _VectorArgs *__va) { ";
              code += "return Create" + vector_type.struct_def->name;
              code += "(*__va->__fbb, ";
              if (field.native_inline) {
                code += "&(__va->_" + value + "[i])";
              } else {
                code += "__va->_" + value + "[i]" + GenPtrGet(field);
              }
              code += ", __va->__rehasher); }, &_va )";
            }
            break;
          }
          case BASE_TYPE_BOOL: {
            code += "_fbb.CreateVector(" + value + ")";
            break;
          }
          case BASE_TYPE_UNION: {
            code +=
                "_fbb.CreateVector<::flatbuffers::"
                "Offset<void>>(" +
                value +
                ".size(), [](size_t i, _VectorArgs *__va) { "
                "return __va->_" +
                value + "[i].Pack(*__va->__fbb, __va->__rehasher); }, &_va)";
            break;
          }
          case BASE_TYPE_UTYPE: {
            value = StripUnionType(value);
            auto underlying_type = GenTypeBasic(vector_type, false);
            const std::string &type = opts_.scoped_enums
                                          ? Name(*field.value.type.enum_def)
                                          : underlying_type;
            auto enum_value = "__va->_" + value + "[i].type";
            if (!opts_.scoped_enums)
              enum_value =
                  "static_cast<" + underlying_type + ">(" + enum_value + ")";

            code += "_fbb.CreateVector<" + type + ">(" + value +
                    ".size(), [](size_t i, _VectorArgs *__va) { return " +
                    enum_value + "; }, &_va)";
            break;
          }
          default: {
            if (field.value.type.enum_def &&
                !VectorElementUserFacing(vector_type)) {
              // For enumerations, we need to get access to the array data for
              // the underlying storage type (eg. uint8_t).
              const auto basetype = GenTypeBasic(
                  field.value.type.enum_def->underlying_type, false);
              code += "_fbb.CreateVectorScalarCast<" + basetype +
                      ">(::flatbuffers::data(" + value + "), " + value +
                      ".size())";
            } else if (field.attributes.Lookup("cpp_type")) {
              auto type = GenTypeBasic(vector_type, false);
              code += "_fbb.CreateVector<" + type + ">(" + value + ".size(), ";
              code += "[](size_t i, _VectorArgs *__va) { ";
              code += "return __va->__rehasher ? ";
              code += "static_cast<" + type + ">((*__va->__rehasher)";
              code += "(__va->_" + value + "[i]" + GenPtrGet(field) + ")) : 0";
              code += "; }, &_va )";
            } else {
              // If the field uses 64-bit addressing, create a 64-bit vector.
              if (field.value.type.base_type == BASE_TYPE_VECTOR64) {
                code += "_fbb.CreateVector64(" + value + ")";
              } else {
                code += "_fbb.CreateVector";
                if (field.offset64) {
                  // This is normal 32-bit vector, with 64-bit addressing.
                  code += "64<::flatbuffers::Vector>";
                }
                code += "(" + value + ")";
              }
            }
            break;
          }
        }

        // If set_empty_vectors_to_null option is enabled, for optional fields,
        // check to see if there actually is any data in _o->field before
        // attempting to access it.
        if (field.attributes.Lookup("nested_flatbuffer") ||
            (opts_.set_empty_vectors_to_null && !field.IsRequired())) {
          code = value + ".size() ? " + code + " : 0";
        }
        break;
      }
      case BASE_TYPE_UNION: {
        // _o->field.Pack(_fbb);
        code += value + ".Pack(_fbb)";
        break;
      }
      case BASE_TYPE_STRUCT: {
        if (IsStruct(field.value.type)) {
          const auto &struct_attribs = field.value.type.struct_def->attributes;
          const auto native_type = struct_attribs.Lookup("native_type");
          if (native_type) {
            code += "::flatbuffers::Pack";
            const auto pack_name =
                struct_attribs.Lookup("native_type_pack_name");
            if (pack_name) { code += pack_name->constant; }
            code += "(" + value + ")";
          } else if (field.native_inline) {
            code += "&" + value;
          } else {
            code += value + " ? " + value + GenPtrGet(field) + " : nullptr";
          }
        } else {
          // _o->field ? CreateT(_fbb, _o->field.get(), _rehasher);
          const std::string &type = field.value.type.struct_def->name;
          code += value + " ? Create" + type;
          code += "(_fbb, " + value;
          if (!field.native_inline) code += GenPtrGet(field);
          code += ", _rehasher) : 0";
        }
        break;
      }
      default: {
        code += value;
        break;
      }
    }
    return code;
  }

  // Generate code for tables that needs to come after the regular definition.
  void GenTablePost(const StructDef &struct_def) {
    if (opts_.generate_object_based_api) { GenNativeTablePost(struct_def); }

    code_.SetValue("STRUCT_NAME", Name(struct_def));
    code_.SetValue("NATIVE_NAME",
                   NativeName(Name(struct_def), &struct_def, opts_));

    if (opts_.generate_object_based_api) {
      // Generate the >= C++11 copy ctor and assignment operator definitions.
      GenCopyCtorAssignOpDefs(struct_def);

      // Generate the X::UnPack() method.
      code_ +=
          "inline " + TableUnPackSignature(struct_def, false, opts_) + " {";

      if (opts_.g_cpp_std == cpp::CPP_STD_X0) {
        auto native_name = WrapNativeNameInNameSpace(struct_def, parser_.opts);
        code_.SetValue("POINTER_TYPE",
                       GenTypeNativePtr(native_name, nullptr, false));
        code_ +=
            "  {{POINTER_TYPE}} _o = {{POINTER_TYPE}}(new {{NATIVE_NAME}}());";
      } else if (opts_.g_cpp_std == cpp::CPP_STD_11) {
        code_ +=
            "  auto _o = std::unique_ptr<{{NATIVE_NAME}}>(new "
            "{{NATIVE_NAME}}());";
      } else {
        code_ += "  auto _o = std::make_unique<{{NATIVE_NAME}}>();";
      }
      code_ += "  UnPackTo(_o.get(), _resolver);";
      code_ += "  return _o.release();";
      code_ += "}";
      code_ += "";
      code_ +=
          "inline " + TableUnPackToSignature(struct_def, false, opts_) + " {";
      code_ += "  (void)_o;";
      code_ += "  (void)_resolver;";

      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        const auto &field = **it;
        if (field.deprecated) { continue; }

        // Assign a value from |this| to |_o|.   Values from |this| are stored
        // in a variable |_e| by calling this->field_type().  The value is then
        // assigned to |_o| using the GenUnpackFieldStatement.
        const bool is_union = field.value.type.base_type == BASE_TYPE_UTYPE;
        const auto statement =
            GenUnpackFieldStatement(field, is_union ? *(it + 1) : nullptr);

        code_.SetValue("FIELD_NAME", Name(field));
        auto prefix = "  { auto _e = {{FIELD_NAME}}(); ";
        auto check = IsScalar(field.value.type.base_type) ? "" : "if (_e) ";
        auto postfix = " }";
        code_ += std::string(prefix) + check + statement + postfix;
      }
      code_ += "}";
      code_ += "";

      // Generate the X::Pack member function that simply calls the global
      // CreateX function.
      code_ += "inline " + TablePackSignature(struct_def, false, opts_) + " {";
      code_ += "  return Create{{STRUCT_NAME}}(_fbb, _o, _rehasher);";
      code_ += "}";
      code_ += "";

      // Generate a CreateX method that works with an unpacked C++ object.
      code_ +=
          "inline " + TableCreateSignature(struct_def, false, opts_) + " {";
      code_ += "  (void)_rehasher;";
      code_ += "  (void)_o;";

      code_ +=
          "  struct _VectorArgs "
          "{ " +
          GetBuilder() +
          " *__fbb; "
          "const " +
          NativeName(Name(struct_def), &struct_def, opts_) +
          "* __o; "
          "const ::flatbuffers::rehasher_function_t *__rehasher; } _va = { "
          "&_fbb, _o, _rehasher}; (void)_va;";

      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) { continue; }
        if (IsVector(field.value.type)) {
          const std::string force_align_code =
              GenVectorForceAlign(field, "_o->" + Name(field) + ".size()");
          if (!force_align_code.empty()) { code_ += "  " + force_align_code; }
        }
        code_ += "  auto _" + Name(field) + " = " + GenCreateParam(field) + ";";
      }
      // Need to call "Create" with the struct namespace.
      const auto qualified_create_name =
          struct_def.defined_namespace->GetFullyQualifiedName("Create");
      code_.SetValue("CREATE_NAME", TranslateNameSpace(qualified_create_name));

      code_ += "  return {{CREATE_NAME}}{{STRUCT_NAME}}(";
      code_ += "      _fbb\\";
      for (const auto &field : struct_def.fields.vec) {
        if (field->deprecated) { continue; }

        bool pass_by_address = false;
        if (field->value.type.base_type == BASE_TYPE_STRUCT) {
          if (IsStruct(field->value.type)) {
            auto native_type =
                field->value.type.struct_def->attributes.Lookup("native_type");
            if (native_type) { pass_by_address = true; }
          }
        }

        // Call the CreateX function using values from |_o|.
        if (pass_by_address) {
          code_ += ",\n      &_" + Name(*field) + "\\";
        } else {
          code_ += ",\n      _" + Name(*field) + "\\";
        }
      }
      code_ += ");";
      code_ += "}";
      code_ += "";
    }
  }

  static void GenPadding(
      const FieldDef &field, std::string *code_ptr, int *id,
      const std::function<void(int bits, std::string *code_ptr, int *id)> &f) {
    if (field.padding) {
      for (int i = 0; i < 4; i++) {
        if (static_cast<int>(field.padding) & (1 << i)) {
          f((1 << i) * 8, code_ptr, id);
        }
      }
      FLATBUFFERS_ASSERT(!(field.padding & ~0xF));
    }
  }

  static void PaddingDefinition(int bits, std::string *code_ptr, int *id) {
    *code_ptr += "  int" + NumToString(bits) + "_t padding" +
                 NumToString((*id)++) + "__;";
  }

  static void PaddingInitializer(int bits, std::string *code_ptr, int *id) {
    (void)bits;
    if (!code_ptr->empty()) *code_ptr += ",\n        ";
    *code_ptr += "padding" + NumToString((*id)++) + "__(0)";
  }

  static void PaddingNoop(int bits, std::string *code_ptr, int *id) {
    (void)bits;
    if (!code_ptr->empty()) *code_ptr += '\n';
    *code_ptr += "    (void)padding" + NumToString((*id)++) + "__;";
  }

  void GenStructDefaultConstructor(const StructDef &struct_def) {
    std::string init_list;
    std::string body;
    bool first_in_init_list = true;
    int padding_initializer_id = 0;
    int padding_body_id = 0;
    for (const auto &field : struct_def.fields.vec) {
      const auto field_name = Name(*field) + "_";

      if (first_in_init_list) {
        first_in_init_list = false;
      } else {
        init_list += ",";
        init_list += "\n        ";
      }

      init_list += field_name;
      if (IsStruct(field->value.type) || IsArray(field->value.type)) {
        // this is either default initialization of struct
        // or
        // implicit initialization of array
        // for each object in array it:
        // * sets it as zeros for POD types (integral, floating point, etc)
        // * calls default constructor for classes/structs
        init_list += "()";
      } else {
        init_list += "(0)";
      }
      if (field->padding) {
        GenPadding(*field, &init_list, &padding_initializer_id,
                   PaddingInitializer);
        GenPadding(*field, &body, &padding_body_id, PaddingNoop);
      }
    }

    if (init_list.empty()) {
      code_ += "  {{STRUCT_NAME}}()";
      code_ += "  {}";
    } else {
      code_.SetValue("INIT_LIST", init_list);
      code_ += "  {{STRUCT_NAME}}()";
      code_ += "      : {{INIT_LIST}} {";
      if (!body.empty()) { code_ += body; }
      code_ += "  }";
    }
  }

  void GenStructConstructor(const StructDef &struct_def,
                            GenArrayArgMode array_mode) {
    std::string arg_list;
    std::string init_list;
    int padding_id = 0;
    auto first = struct_def.fields.vec.begin();
    // skip arrays if generate ctor without array assignment
    const auto init_arrays = (array_mode != kArrayArgModeNone);
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      const auto &type = field.value.type;
      const auto is_array = IsArray(type);
      const auto arg_name = "_" + Name(field);
      if (!is_array || init_arrays) {
        if (it != first && !arg_list.empty()) { arg_list += ", "; }
        arg_list += !is_array ? GenTypeGet(type, " ", "const ", " &", true)
                              : GenTypeSpan(type, true, type.fixed_length);
        arg_list += arg_name;
      }
      // skip an array with initialization from span
      if (false == (is_array && init_arrays)) {
        if (it != first && !init_list.empty()) { init_list += ",\n        "; }
        init_list += Name(field) + "_";
        if (IsScalar(type.base_type)) {
          auto scalar_type = GenUnderlyingCast(field, false, arg_name);
          init_list += "(::flatbuffers::EndianScalar(" + scalar_type + "))";
        } else {
          FLATBUFFERS_ASSERT((is_array && !init_arrays) || IsStruct(type));
          if (!is_array)
            init_list += "(" + arg_name + ")";
          else
            init_list += "()";
        }
      }
      if (field.padding)
        GenPadding(field, &init_list, &padding_id, PaddingInitializer);
    }

    if (!arg_list.empty()) {
      code_.SetValue("ARG_LIST", arg_list);
      code_.SetValue("INIT_LIST", init_list);
      if (!init_list.empty()) {
        code_ += "  {{STRUCT_NAME}}({{ARG_LIST}})";
        code_ += "      : {{INIT_LIST}} {";
      } else {
        code_ += "  {{STRUCT_NAME}}({{ARG_LIST}}) {";
      }
      padding_id = 0;
      for (const auto &field : struct_def.fields.vec) {
        const auto &type = field->value.type;
        if (IsArray(type) && init_arrays) {
          const auto &element_type = type.VectorType();
          const auto is_enum = IsEnum(element_type);
          FLATBUFFERS_ASSERT(
              (IsScalar(element_type.base_type) || IsStruct(element_type)) &&
              "invalid declaration");
          const auto face_type = GenTypeGet(type, " ", "", "", is_enum);
          std::string get_array =
              is_enum ? "CastToArrayOfEnum<" + face_type + ">" : "CastToArray";
          const auto field_name = Name(*field) + "_";
          const auto arg_name = "_" + Name(*field);
          code_ += "    ::flatbuffers::" + get_array + "(" + field_name +
                   ").CopyFromSpan(" + arg_name + ");";
        }
        if (field->padding) {
          std::string padding;
          GenPadding(*field, &padding, &padding_id, PaddingNoop);
          code_ += padding;
        }
      }
      code_ += "  }";
    }
  }

  void GenArrayAccessor(const Type &type, bool mutable_accessor) {
    FLATBUFFERS_ASSERT(IsArray(type));
    const auto is_enum = IsEnum(type.VectorType());
    // The Array<bool,N> is a tricky case, like std::vector<bool>.
    // It requires a specialization of Array class.
    // Generate Array<uint8_t> for Array<bool>.
    const auto face_type = GenTypeGet(type, " ", "", "", is_enum);
    std::string ret_type = "::flatbuffers::Array<" + face_type + ", " +
                           NumToString(type.fixed_length) + ">";
    if (mutable_accessor)
      code_ += "  " + ret_type + " *mutable_{{FIELD_NAME}}() {";
    else
      code_ += "  const " + ret_type + " *{{FIELD_NAME}}() const {";

    std::string get_array =
        is_enum ? "CastToArrayOfEnum<" + face_type + ">" : "CastToArray";
    code_ += "    return &::flatbuffers::" + get_array + "({{FIELD_VALUE}});";
    code_ += "  }";
  }

  // Generate an accessor struct with constructor for a flatbuffers struct.
  void GenStruct(const StructDef &struct_def) {
    // Generate an accessor struct, with private variables of the form:
    // type name_;
    // Generates manual padding and alignment.
    // Variables are private because they contain little endian data on all
    // platforms.
    GenComment(struct_def.doc_comment);
    code_.SetValue("ALIGN", NumToString(struct_def.minalign));
    code_.SetValue("STRUCT_NAME", Name(struct_def));

    code_ +=
        "FLATBUFFERS_MANUALLY_ALIGNED_STRUCT({{ALIGN}}) "
        "{{STRUCT_NAME}} FLATBUFFERS_FINAL_CLASS {";
    code_ += " private:";

    int padding_id = 0;
    for (const auto &field : struct_def.fields.vec) {
      const auto &field_type = field->value.type;
      code_.SetValue("FIELD_TYPE", GenTypeGet(field_type, " ", "", " ", false));
      code_.SetValue("FIELD_NAME", Name(*field));
      code_.SetValue("ARRAY",
                     IsArray(field_type)
                         ? "[" + NumToString(field_type.fixed_length) + "]"
                         : "");
      code_ += ("  {{FIELD_TYPE}}{{FIELD_NAME}}_{{ARRAY}};");

      if (field->padding) {
        std::string padding;
        GenPadding(*field, &padding, &padding_id, PaddingDefinition);
        code_ += padding;
      }
    }

    // Generate GetFullyQualifiedName
    code_ += "";
    code_ += " public:";

    if (opts_.g_cpp_std >= cpp::CPP_STD_17) { code_ += "  struct Traits;"; }

    // Make TypeTable accessible via the generated struct.
    if (opts_.mini_reflect != IDLOptions::kNone) {
      code_ +=
          "  static const ::flatbuffers::TypeTable *MiniReflectTypeTable() {";
      code_ += "    return {{STRUCT_NAME}}TypeTable();";
      code_ += "  }";
    }

    GenFullyQualifiedNameGetter(struct_def, Name(struct_def));

    // Generate a default constructor.
    GenStructDefaultConstructor(struct_def);

    // Generate a constructor that takes all fields as arguments,
    // excluding arrays.
    GenStructConstructor(struct_def, kArrayArgModeNone);

    auto arrays_num = std::count_if(
        struct_def.fields.vec.begin(), struct_def.fields.vec.end(),
        [](const FieldDef *fd) { return IsArray(fd->value.type); });
    if (arrays_num > 0) {
      GenStructConstructor(struct_def, kArrayArgModeSpanStatic);
    }

    // Generate accessor methods of the form:
    // type name() const { return ::flatbuffers::EndianScalar(name_); }
    for (const auto &field : struct_def.fields.vec) {
      const auto &type = field->value.type;
      const auto is_scalar = IsScalar(type.base_type);
      const auto is_array = IsArray(type);

      const auto field_type = GenTypeGet(type, " ", is_array ? "" : "const ",
                                         is_array ? "" : " &", true);
      auto member = Name(*field) + "_";
      const std::string &value =
          is_scalar ? "::flatbuffers::EndianScalar(" + member + ")" : member;

      code_.SetValue("FIELD_NAME", Name(*field));
      code_.SetValue("FIELD_TYPE", field_type);
      code_.SetValue("FIELD_VALUE", GenUnderlyingCast(*field, true, value));

      GenComment(field->doc_comment, "  ");

      // Generate a const accessor function.
      if (is_array) {
        GenArrayAccessor(type, false);
      } else {
        code_ += "  {{FIELD_TYPE}}{{FIELD_NAME}}() const {";
        code_ += "    return {{FIELD_VALUE}};";
        code_ += "  }";
      }

      // Generate a mutable accessor function.
      if (opts_.mutable_buffer) {
        auto mut_field_type =
            GenTypeGet(type, " ", "", is_array ? "" : " &", true);
        code_.SetValue("FIELD_TYPE", mut_field_type);
        if (is_scalar) {
          code_.SetValue("ARG", GenTypeBasic(type, true));
          code_.SetValue("FIELD_VALUE",
                         GenUnderlyingCast(*field, false, "_" + Name(*field)));

          code_ += "  void mutate_{{FIELD_NAME}}({{ARG}} _{{FIELD_NAME}}) {";
          code_ +=
              "    ::flatbuffers::WriteScalar(&{{FIELD_NAME}}_, "
              "{{FIELD_VALUE}});";
          code_ += "  }";
        } else if (is_array) {
          GenArrayAccessor(type, true);
        } else {
          code_ += "  {{FIELD_TYPE}}mutable_{{FIELD_NAME}}() {";
          code_ += "    return {{FIELD_VALUE}};";
          code_ += "  }";
        }
      }

      // Generate a comparison function for this field if it is a key.
      if (field->key) { GenKeyFieldMethods(*field); }
    }
    code_.SetValue("NATIVE_NAME", Name(struct_def));
    GenOperatorNewDelete(struct_def);

    if (opts_.cpp_static_reflection) { GenIndexBasedFieldGetter(struct_def); }

    code_ += "};";

    code_.SetValue("STRUCT_BYTE_SIZE", NumToString(struct_def.bytesize));
    code_ += "FLATBUFFERS_STRUCT_END({{STRUCT_NAME}}, {{STRUCT_BYTE_SIZE}});";
    if (opts_.gen_compare) GenCompareOperator(struct_def, "()");
    code_ += "";

    // Definition for type traits for this table type. This allows querying var-
    // ious compile-time traits of the table.
    if (opts_.g_cpp_std >= cpp::CPP_STD_17) { GenTraitsStruct(struct_def); }
  }

  // Set up the correct namespace. Only open a namespace if the existing one is
  // different (closing/opening only what is necessary).
  //
  // The file must start and end with an empty (or null) namespace so that
  // namespaces are properly opened and closed.
  void SetNameSpace(const Namespace *ns) {
    if (cur_name_space_ == ns) { return; }

    // Compute the size of the longest common namespace prefix.
    // If cur_name_space is A::B::C::D and ns is A::B::E::F::G,
    // the common prefix is A::B:: and we have old_size = 4, new_size = 5
    // and common_prefix_size = 2
    size_t old_size = cur_name_space_ ? cur_name_space_->components.size() : 0;
    size_t new_size = ns ? ns->components.size() : 0;

    size_t common_prefix_size = 0;
    while (common_prefix_size < old_size && common_prefix_size < new_size &&
           ns->components[common_prefix_size] ==
               cur_name_space_->components[common_prefix_size]) {
      common_prefix_size++;
    }

    // Close cur_name_space in reverse order to reach the common prefix.
    // In the previous example, D then C are closed.
    for (size_t j = old_size; j > common_prefix_size; --j) {
      code_ += "}  // namespace " + cur_name_space_->components[j - 1];
    }
    if (old_size != common_prefix_size) { code_ += ""; }

    // open namespace parts to reach the ns namespace
    // in the previous example, E, then F, then G are opened
    for (auto j = common_prefix_size; j != new_size; ++j) {
      code_ += "namespace " + ns->components[j] + " {";
    }
    if (new_size != common_prefix_size) { code_ += ""; }

    cur_name_space_ = ns;
  }
};

}  // namespace cpp

static bool GenerateCPP(const Parser &parser, const std::string &path,
                        const std::string &file_name) {
  cpp::IDLOptionsCpp opts(parser.opts);
  // The '--cpp_std' argument could be extended (like ASAN):
  // Example: "flatc --cpp_std c++17:option1:option2".
  std::string cpp_std = !opts.cpp_std.empty() ? opts.cpp_std : "C++11";
  std::transform(cpp_std.begin(), cpp_std.end(), cpp_std.begin(), CharToUpper);
  if (cpp_std == "C++0X") {
    opts.g_cpp_std = cpp::CPP_STD_X0;
    opts.g_only_fixed_enums = false;
  } else if (cpp_std == "C++11") {
    // Use the standard C++11 code generator.
    opts.g_cpp_std = cpp::CPP_STD_11;
    opts.g_only_fixed_enums = true;
  } else if (cpp_std == "C++17") {
    opts.g_cpp_std = cpp::CPP_STD_17;
    // With c++17 generate strong enums only.
    opts.scoped_enums = true;
    // By default, prefixed_enums==true, reset it.
    opts.prefixed_enums = false;
  } else {
    LogCompilerError("Unknown value of the '--cpp-std' switch: " +
                     opts.cpp_std);
    return false;
  }
  // The opts.scoped_enums has priority.
  opts.g_only_fixed_enums |= opts.scoped_enums;

  if (opts.cpp_static_reflection && opts.g_cpp_std < cpp::CPP_STD_17) {
    LogCompilerError(
        "--cpp-static-reflection requires using --cpp-std at \"C++17\" or "
        "higher.");
    return false;
  }

  cpp::CppGenerator generator(parser, path, file_name, opts);
  return generator.generate();
}

static std::string CPPMakeRule(const Parser &parser, const std::string &path,
                               const std::string &file_name) {
  const auto filebase = StripPath(StripExtension(file_name));
  cpp::CppGenerator geneartor(parser, path, file_name, parser.opts);
  const auto included_files = parser.GetIncludedFilesRecursive(file_name);
  std::string make_rule =
      geneartor.GeneratedFileName(path, filebase, parser.opts) + ": ";
  for (const std::string &included_file : included_files) {
    make_rule += " " + included_file;
  }
  return make_rule;
}

namespace {

class CppCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateCPP(parser, path, filename)) { return Status::ERROR; }
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
    output = CPPMakeRule(parser, path, filename);
    return Status::OK;
  }

  Status GenerateGrpcCode(const Parser &parser, const std::string &path,
                          const std::string &filename) override {
    if (!GenerateCppGRPC(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateRootFile(const Parser &parser,
                          const std::string &path) override {
    (void)parser;
    (void)path;
    return Status::NOT_IMPLEMENTED;
  }

  bool IsSchemaOnly() const override { return true; }

  bool SupportsBfbsGeneration() const override { return false; }

  bool SupportsRootFileGeneration() const override { return false; }

  IDLOptions::Language Language() const override { return IDLOptions::kCpp; }

  std::string LanguageName() const override { return "C++"; }
};

}  // namespace

std::unique_ptr<CodeGenerator> NewCppCodeGenerator() {
  return std::unique_ptr<CppCodeGenerator>(new CppCodeGenerator());
}

}  // namespace flatbuffers
