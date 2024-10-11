/*
 * Copyright 2018 Google Inc. All rights reserved.
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

#include "idl_gen_rust.h"

#include <cmath>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

namespace flatbuffers {
namespace {

static Namer::Config RustDefaultConfig() {
  // Historical note: We've been using "keep" casing since the original
  // implementation, presumably because Flatbuffers schema style and Rust style
  // roughly align. We are not going to enforce proper casing since its an
  // unnecessary breaking change.
  return { /*types=*/Case::kKeep,
           /*constants=*/Case::kScreamingSnake,
           /*methods=*/Case::kSnake,
           /*functions=*/Case::kSnake,
           /*fields=*/Case::kKeep,
           /*variables=*/Case::kUnknown,  // Unused.
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/"::",
           /*escape_keywords=*/Namer::Config::Escape::BeforeConvertingCase,
           /*namespaces=*/Case::kSnake,
           /*namespace_seperator=*/"::",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kSnake,
           /*directories=*/Case::kSnake,
           /*output_path=*/"",
           /*filename_suffix=*/"_generated",
           /*filename_extension=*/".rs" };
}

static std::set<std::string> RustKeywords() {
  return {
    // https://doc.rust-lang.org/book/second-edition/appendix-01-keywords.html
    "as",
    "break",
    "const",
    "continue",
    "crate",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "Self",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    // future possible keywords
    "abstract",
    "alignof",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "offsetof",
    "override",
    "priv",
    "proc",
    "pure",
    "sizeof",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    // other rust terms we should not use
    "std",
    "usize",
    "isize",
    "u8",
    "i8",
    "u16",
    "i16",
    "u32",
    "i32",
    "u64",
    "i64",
    "u128",
    "i128",
    "f32",
    "f64",
    // Terms that we use ourselves
    "follow",
    "push",
    "size",
    "alignment",
    "to_little_endian",
    "from_little_endian",
    "ENUM_MAX",
    "ENUM_MIN",
    "ENUM_VALUES",
  };
}

// Encapsulate all logical field types in this enum. This allows us to write
// field logic based on type switches, instead of branches on the properties
// set on the Type.
// TODO(rw): for backwards compatibility, we can't use a strict `enum class`
//           declaration here. could we use the `-Wswitch-enum` warning to
//           achieve the same effect?
enum FullType {
  ftInteger = 0,
  ftFloat = 1,
  ftBool = 2,

  ftStruct = 3,
  ftTable = 4,

  ftEnumKey = 5,
  ftUnionKey = 6,

  ftUnionValue = 7,

  // TODO(rw): bytestring?
  ftString = 8,

  ftVectorOfInteger = 9,
  ftVectorOfFloat = 10,
  ftVectorOfBool = 11,
  ftVectorOfEnumKey = 12,
  ftVectorOfStruct = 13,
  ftVectorOfTable = 14,
  ftVectorOfString = 15,
  ftVectorOfUnionValue = 16,

  ftArrayOfBuiltin = 17,
  ftArrayOfEnum = 18,
  ftArrayOfStruct = 19,
};

// Convert a Type to a FullType (exhaustive).
static FullType GetFullType(const Type &type) {
  // N.B. The order of these conditionals matters for some types.

  if (IsString(type)) {
    return ftString;
  } else if (type.base_type == BASE_TYPE_STRUCT) {
    if (type.struct_def->fixed) {
      return ftStruct;
    } else {
      return ftTable;
    }
  } else if (IsVector(type)) {
    switch (GetFullType(type.VectorType())) {
      case ftInteger: {
        return ftVectorOfInteger;
      }
      case ftFloat: {
        return ftVectorOfFloat;
      }
      case ftBool: {
        return ftVectorOfBool;
      }
      case ftStruct: {
        return ftVectorOfStruct;
      }
      case ftTable: {
        return ftVectorOfTable;
      }
      case ftString: {
        return ftVectorOfString;
      }
      case ftEnumKey: {
        return ftVectorOfEnumKey;
      }
      case ftUnionKey:
      case ftUnionValue: {
        FLATBUFFERS_ASSERT(false && "vectors of unions are unsupported");
        break;
      }
      default: {
        FLATBUFFERS_ASSERT(false && "vector of vectors are unsupported");
      }
    }
  } else if (IsArray(type)) {
    switch (GetFullType(type.VectorType())) {
      case ftInteger:
      case ftFloat:
      case ftBool: {
        return ftArrayOfBuiltin;
      }
      case ftStruct: {
        return ftArrayOfStruct;
      }
      case ftEnumKey: {
        return ftArrayOfEnum;
      }
      default: {
        FLATBUFFERS_ASSERT(false && "Unsupported type for fixed array");
      }
    }
  } else if (type.enum_def != nullptr) {
    if (type.enum_def->is_union) {
      if (type.base_type == BASE_TYPE_UNION) {
        return ftUnionValue;
      } else if (IsInteger(type.base_type)) {
        return ftUnionKey;
      } else {
        FLATBUFFERS_ASSERT(false && "unknown union field type");
      }
    } else {
      return ftEnumKey;
    }
  } else if (IsScalar(type.base_type)) {
    if (IsBool(type.base_type)) {
      return ftBool;
    } else if (IsInteger(type.base_type)) {
      return ftInteger;
    } else if (IsFloat(type.base_type)) {
      return ftFloat;
    } else {
      FLATBUFFERS_ASSERT(false && "unknown number type");
    }
  }

  FLATBUFFERS_ASSERT(false && "completely unknown type");

  // this is only to satisfy the compiler's return analysis.
  return ftBool;
}

static bool IsBitFlagsEnum(const EnumDef &enum_def) {
  return enum_def.attributes.Lookup("bit_flags") != nullptr;
}

// TableArgs make required non-scalars "Option<_>".
// TODO(cneo): Rework how we do defaults and stuff.
static bool IsOptionalToBuilder(const FieldDef &field) {
  return field.IsOptional() || !IsScalar(field.value.type.base_type);
}
}  // namespace

static bool GenerateRustModuleRootFile(const Parser &parser,
                                       const std::string &output_dir) {
  if (!parser.opts.rust_module_root_file) {
    // Don't generate a root file when generating one file. This isn't an error
    // so return true.
    return true;
  }
  Namer namer(WithFlagOptions(RustDefaultConfig(), parser.opts, output_dir),
              RustKeywords());
  // We gather the symbols into a tree of namespaces (which are rust mods) and
  // generate a file that gathers them all.
  struct Module {
    std::map<std::string, Module> sub_modules;
    std::vector<std::string> generated_files;
    // Add a symbol into the tree.
    void Insert(const Namer &namer, const Definition *s) {
      const Definition &symbol = *s;
      Module *current_module = this;
      for (auto it = symbol.defined_namespace->components.begin();
           it != symbol.defined_namespace->components.end(); it++) {
        std::string ns_component = namer.Namespace(*it);
        current_module = &current_module->sub_modules[ns_component];
      }
      current_module->generated_files.push_back(
          namer.File(symbol.name, SkipFile::Extension));
    }
    // Recursively create the importer file.
    void GenerateImports(CodeWriter &code) {
      for (auto it = sub_modules.begin(); it != sub_modules.end(); it++) {
        code += "pub mod " + it->first + " {";
        code.IncrementIdentLevel();
        code += "use super::*;";
        it->second.GenerateImports(code);
        code.DecrementIdentLevel();
        code += "} // " + it->first;
      }
      for (auto it = generated_files.begin(); it != generated_files.end();
           it++) {
        code += "mod " + *it + ";";
        code += "pub use self::" + *it + "::*;";
      }
    }
  };
  Module root_module;
  for (auto it = parser.enums_.vec.begin(); it != parser.enums_.vec.end();
       it++) {
    root_module.Insert(namer, *it);
  }
  for (auto it = parser.structs_.vec.begin(); it != parser.structs_.vec.end();
       it++) {
    root_module.Insert(namer, *it);
  }
  CodeWriter code("  ");
  // TODO(caspern): Move generated warning out of BaseGenerator.
  code +=
      "// Automatically generated by the Flatbuffers compiler. "
      "Do not modify.";
  code += "// @generated";
  root_module.GenerateImports(code);
  const bool success =
      SaveFile((output_dir + "mod.rs").c_str(), code.ToString(), false);
  code.Clear();
  return success;
}

namespace rust {

class RustGenerator : public BaseGenerator {
 public:
  RustGenerator(const Parser &parser, const std::string &path,
                const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "", "::", "rs"),
        cur_name_space_(nullptr),
        namer_(WithFlagOptions(RustDefaultConfig(), parser.opts, path),
               RustKeywords()) {
    // TODO: Namer flag overrides should be in flatc or flatc_main.
    code_.SetPadding("  ");
  }

  bool generate() {
    if (!parser_.opts.rust_module_root_file) {
      return GenerateOneFile();
    } else {
      return GenerateIndividualFiles();
    }
  }

  template<typename T>
  bool GenerateSymbols(const SymbolTable<T> &symbols,
                       std::function<void(const T &)> gen_symbol) {
    for (auto it = symbols.vec.begin(); it != symbols.vec.end(); it++) {
      const T &symbol = **it;
      if (symbol.generated) continue;
      code_.Clear();
      code_ += "// " + std::string(FlatBuffersGeneratedWarning());
      code_ += "// @generated";
      code_ += "extern crate alloc;";
      code_ += "extern crate flatbuffers;";
      code_ += "use alloc::boxed::Box;";
      code_ += "use alloc::string::{String, ToString};";
      code_ += "use alloc::vec::Vec;";
      code_ += "use core::mem;";
      code_ += "use core::cmp::Ordering;";
      if (parser_.opts.rust_serialize) {
        code_ += "extern crate serde;";
        code_ +=
            "use self::serde::ser::{Serialize, Serializer, SerializeStruct};";
      }
      code_ += "use self::flatbuffers::{EndianScalar, Follow};";
      code_ += "use super::*;";
      cur_name_space_ = symbol.defined_namespace;
      gen_symbol(symbol);

      const std::string directories =
          namer_.Directories(*symbol.defined_namespace);
      EnsureDirExists(directories);
      const std::string file_path = directories + namer_.File(symbol);
      const bool save_success =
          SaveFile(file_path.c_str(), code_.ToString(), /*binary=*/false);
      if (!save_success) return false;
    }
    return true;
  }

  bool GenerateIndividualFiles() {
    code_.Clear();
    // Don't bother with imports. Use absolute paths everywhere.
    return GenerateSymbols<EnumDef>(
               parser_.enums_, [&](const EnumDef &e) { this->GenEnum(e); }) &&
           GenerateSymbols<StructDef>(
               parser_.structs_, [&](const StructDef &s) {
                 if (s.fixed) {
                   this->GenStruct(s);
                 } else {
                   this->GenTable(s);
                   if (this->parser_.opts.generate_object_based_api) {
                     this->GenTableObject(s);
                   }
                 }
                 if (this->parser_.root_struct_def_ == &s) {
                   this->GenRootTableFuncs(s);
                 }
               });
  }

  // Generates code organized by .fbs files. This is broken legacy behavior
  // that does not work with multiple fbs files with shared namespaces.
  // Iterate through all definitions we haven't generated code for (enums,
  // structs, and tables) and output them to a single file.
  bool GenerateOneFile() {
    code_.Clear();
    code_ += "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n";
    code_ += "// @generated";

    assert(!cur_name_space_);

    // Generate imports for the global scope in case no namespace is used
    // in the schema file.
    GenNamespaceImports(0);
    code_ += "";

    // Generate all code in their namespaces, once, because Rust does not
    // permit re-opening modules.
    //
    // TODO(rw): Use a set data structure to reduce namespace evaluations from
    //           O(n**2) to O(n).
    for (auto ns_it = parser_.namespaces_.begin();
         ns_it != parser_.namespaces_.end(); ++ns_it) {
      const auto &ns = *ns_it;

      // Generate code for all the enum declarations.
      for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
           ++it) {
        const auto &enum_def = **it;
        if (enum_def.defined_namespace == ns && !enum_def.generated) {
          SetNameSpace(enum_def.defined_namespace);
          GenEnum(enum_def);
        }
      }

      // Generate code for all structs.
      for (auto it = parser_.structs_.vec.begin();
           it != parser_.structs_.vec.end(); ++it) {
        const auto &struct_def = **it;
        if (struct_def.defined_namespace == ns && struct_def.fixed &&
            !struct_def.generated) {
          SetNameSpace(struct_def.defined_namespace);
          GenStruct(struct_def);
        }
      }

      // Generate code for all tables.
      for (auto it = parser_.structs_.vec.begin();
           it != parser_.structs_.vec.end(); ++it) {
        const auto &struct_def = **it;
        if (struct_def.defined_namespace == ns && !struct_def.fixed &&
            !struct_def.generated) {
          SetNameSpace(struct_def.defined_namespace);
          GenTable(struct_def);
          if (parser_.opts.generate_object_based_api) {
            GenTableObject(struct_def);
          }
        }
      }

      // Generate global helper functions.
      if (parser_.root_struct_def_) {
        auto &struct_def = *parser_.root_struct_def_;
        if (struct_def.defined_namespace != ns) { continue; }
        SetNameSpace(struct_def.defined_namespace);
        GenRootTableFuncs(struct_def);
      }
    }
    if (cur_name_space_) SetNameSpace(nullptr);

    const auto file_path = GeneratedFileName(path_, file_name_, parser_.opts);
    const auto final_code = code_.ToString();
    return SaveFile(file_path.c_str(), final_code, false);
  }

 private:
  CodeWriter code_;

  // This tracks the current namespace so we can insert namespace declarations.
  const Namespace *cur_name_space_;

  const Namespace *CurrentNameSpace() const { return cur_name_space_; }

  // Determine if a Type needs a lifetime template parameter when used in the
  // Rust builder args.
  bool TableBuilderTypeNeedsLifetime(const Type &type) const {
    switch (GetFullType(type)) {
      case ftInteger:
      case ftFloat:
      case ftBool:
      case ftEnumKey:
      case ftUnionKey:
      case ftUnionValue: {
        return false;
      }
      default: {
        return true;
      }
    }
  }

  // Determine if a table args rust type needs a lifetime template parameter.
  bool TableBuilderArgsNeedsLifetime(const StructDef &struct_def) const {
    FLATBUFFERS_ASSERT(!struct_def.fixed);

    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) { continue; }

      if (TableBuilderTypeNeedsLifetime(field.value.type)) { return true; }
    }

    return false;
  }

  std::string NamespacedNativeName(const EnumDef &def) {
    return WrapInNameSpace(def.defined_namespace, namer_.ObjectType(def));
  }
  std::string NamespacedNativeName(const StructDef &def) {
    return WrapInNameSpace(def.defined_namespace, namer_.ObjectType(def));
  }

  std::string WrapInNameSpace(const Definition &def) const {
    return WrapInNameSpace(def.defined_namespace,
                           namer_.EscapeKeyword(def.name));
  }
  std::string WrapInNameSpace(const Namespace *ns,
                              const std::string &name) const {
    if (CurrentNameSpace() == ns) return name;
    std::string prefix = GetRelativeNamespaceTraversal(CurrentNameSpace(), ns);
    return prefix + name;
  }

  // Determine the relative namespace traversal needed to reference one
  // namespace from another namespace. This is useful because it does not force
  // the user to have a particular file layout. (If we output absolute
  // namespace paths, that may require users to organize their Rust crates in a
  // particular way.)
  std::string GetRelativeNamespaceTraversal(const Namespace *src,
                                            const Namespace *dst) const {
    // calculate the path needed to reference dst from src.
    // example: f(A::B::C, A::B::C) -> (none)
    // example: f(A::B::C, A::B)    -> super::
    // example: f(A::B::C, A::B::D) -> super::D
    // example: f(A::B::C, A)       -> super::super::
    // example: f(A::B::C, D)       -> super::super::super::D
    // example: f(A::B::C, D::E)    -> super::super::super::D::E
    // example: f(A, D::E)          -> super::D::E
    // does not include leaf object (typically a struct type).

    std::stringstream stream;
    size_t common = 0;
    std::vector<std::string> s, d;
    if (src) s = src->components;
    if (dst) d = dst->components;
    while (common < s.size() && common < d.size() && s[common] == d[common])
      common++;
    // If src namespace is empty, this must be an absolute path.
    for (size_t i = common; i < s.size(); i++) stream << "super::";
    for (size_t i = common; i < d.size(); i++)
      stream << namer_.Namespace(d[i]) + "::";
    return stream.str();
  }

  // Generate a comment from the schema.
  void GenComment(const std::vector<std::string> &dc, const char *prefix = "") {
    for (auto it = dc.begin(); it != dc.end(); it++) {
      code_ += std::string(prefix) + "///" + *it;
    }
  }

  // Return a Rust type from the table in idl.h.
  std::string GetTypeBasic(const Type &type) const {
    switch (GetFullType(type)) {
      case ftInteger:
      case ftFloat:
      case ftBool:
      case ftEnumKey:
      case ftUnionKey: {
        break;
      }
      default: {
        FLATBUFFERS_ASSERT(false && "incorrect type given");
      }
    }

    // clang-format off
    static const char * const ctypename[] = {
    #define FLATBUFFERS_TD(ENUM, IDLTYPE, CTYPE, JTYPE, GTYPE, NTYPE, PTYPE, \
                           RTYPE, ...) \
      #RTYPE,
      FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
    #undef FLATBUFFERS_TD
    };
    // clang-format on

    if (type.enum_def) { return WrapInNameSpace(*type.enum_def); }
    return ctypename[type.base_type];
  }

  // Look up the native type for an enum. This will always be an integer like
  // u8, i32, etc.
  std::string GetEnumTypeForDecl(const Type &type) {
    const auto ft = GetFullType(type);
    if (!(ft == ftEnumKey || ft == ftUnionKey)) {
      FLATBUFFERS_ASSERT(false && "precondition failed in GetEnumTypeForDecl");
    }

    // clang-format off
    static const char *ctypename[] = {
    #define FLATBUFFERS_TD(ENUM, IDLTYPE, CTYPE, JTYPE, GTYPE, NTYPE, PTYPE, \
                           RTYPE, ...) \
      #RTYPE,
      FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
    #undef FLATBUFFERS_TD
    };
    // clang-format on

    // Enums can be bools, but their Rust representation must be a u8, as used
    // in the repr attribute (#[repr(bool)] is an invalid attribute).
    if (type.base_type == BASE_TYPE_BOOL) return "u8";
    return ctypename[type.base_type];
  }

  // Return a Rust type for any type (scalar, table, struct) specifically for
  // using a FlatBuffer.
  std::string GetTypeGet(const Type &type) const {
    switch (GetFullType(type)) {
      case ftInteger:
      case ftFloat:
      case ftBool:
      case ftEnumKey:
      case ftUnionKey: {
        return GetTypeBasic(type);
      }
      case ftArrayOfBuiltin:
      case ftArrayOfEnum:
      case ftArrayOfStruct: {
        return "[" + GetTypeGet(type.VectorType()) + "; " +
               NumToString(type.fixed_length) + "]";
      }
      case ftTable: {
        return WrapInNameSpace(type.struct_def->defined_namespace,
                               type.struct_def->name) +
               "<'a>";
      }
      default: {
        return WrapInNameSpace(type.struct_def->defined_namespace,
                               type.struct_def->name);
      }
    }
  }

  std::string GetEnumValue(const EnumDef &enum_def,
                           const EnumVal &enum_val) const {
    return namer_.EnumVariant(enum_def, enum_val);
  }

  // 1 suffix since old C++ can't figure out the overload.
  void ForAllEnumValues1(const EnumDef &enum_def,
                         std::function<void(const EnumVal &)> cb) {
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      const auto &ev = **it;
      code_.SetValue("VARIANT", namer_.Variant(ev));
      code_.SetValue("VALUE", enum_def.ToString(ev));
      code_.IncrementIdentLevel();
      cb(ev);
      code_.DecrementIdentLevel();
    }
  }
  void ForAllEnumValues(const EnumDef &enum_def, std::function<void()> cb) {
    std::function<void(const EnumVal &)> wrapped = [&](const EnumVal &unused) {
      (void)unused;
      cb();
    };
    ForAllEnumValues1(enum_def, wrapped);
  }
  // Generate an enum declaration,
  // an enum string lookup table,
  // an enum match function,
  // and an enum array of values
  void GenEnum(const EnumDef &enum_def) {
    const bool is_private = parser_.opts.no_leak_private_annotations &&
                            (enum_def.attributes.Lookup("private") != nullptr);
    code_.SetValue("ACCESS_TYPE", is_private ? "pub(crate)" : "pub");
    code_.SetValue("ENUM_TY", namer_.Type(enum_def));
    code_.SetValue("BASE_TYPE", GetEnumTypeForDecl(enum_def.underlying_type));
    code_.SetValue("ENUM_NAMESPACE", namer_.Namespace(enum_def.name));
    code_.SetValue("ENUM_CONSTANT", namer_.Constant(enum_def.name));
    const EnumVal *minv = enum_def.MinValue();
    const EnumVal *maxv = enum_def.MaxValue();
    FLATBUFFERS_ASSERT(minv && maxv);
    code_.SetValue("ENUM_MIN_BASE_VALUE", enum_def.ToString(*minv));
    code_.SetValue("ENUM_MAX_BASE_VALUE", enum_def.ToString(*maxv));

    if (IsBitFlagsEnum(enum_def)) {
      // Defer to the convenient and canonical bitflags crate. We declare it in
      // a module to #allow camel case constants in a smaller scope. This
      // matches Flatbuffers c-modeled enums where variants are associated
      // constants but in camel case.
      code_ += "#[allow(non_upper_case_globals)]";
      code_ += "mod bitflags_{{ENUM_NAMESPACE}} {";
      code_ += "  flatbuffers::bitflags::bitflags! {";
      GenComment(enum_def.doc_comment, "    ");
      code_ += "    #[derive(Default)]";
      code_ += "    {{ACCESS_TYPE}} struct {{ENUM_TY}}: {{BASE_TYPE}} {";
      ForAllEnumValues1(enum_def, [&](const EnumVal &ev) {
        this->GenComment(ev.doc_comment, "    ");
        code_ += "    const {{VARIANT}} = {{VALUE}};";
      });
      code_ += "    }";
      code_ += "  }";
      code_ += "}";
      code_ += "pub use self::bitflags_{{ENUM_NAMESPACE}}::{{ENUM_TY}};";
      code_ += "";

      code_.SetValue("INTO_BASE", "self.bits()");
    } else {
      // Normal, c-modelled enums.
      // Deprecated associated constants;
      const std::string deprecation_warning =
          "#[deprecated(since = \"2.0.0\", note = \"Use associated constants"
          " instead. This will no longer be generated in 2021.\")]";
      code_ += deprecation_warning;
      code_ +=
          "pub const ENUM_MIN_{{ENUM_CONSTANT}}: {{BASE_TYPE}}"
          " = {{ENUM_MIN_BASE_VALUE}};";
      code_ += deprecation_warning;
      code_ +=
          "pub const ENUM_MAX_{{ENUM_CONSTANT}}: {{BASE_TYPE}}"
          " = {{ENUM_MAX_BASE_VALUE}};";
      auto num_fields = NumToString(enum_def.size());
      code_ += deprecation_warning;
      code_ += "#[allow(non_camel_case_types)]";
      code_ += "pub const ENUM_VALUES_{{ENUM_CONSTANT}}: [{{ENUM_TY}}; " +
               num_fields + "] = [";
      ForAllEnumValues1(enum_def, [&](const EnumVal &ev) {
        code_ += namer_.EnumVariant(enum_def, ev) + ",";
      });
      code_ += "];";
      code_ += "";

      GenComment(enum_def.doc_comment);
      // Derive Default to be 0. flatc enforces this when the enum
      // is put into a struct, though this isn't documented behavior, it is
      // needed to derive defaults in struct objects.
      code_ +=
          "#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, "
          "Default)]";
      code_ += "#[repr(transparent)]";
      code_ += "{{ACCESS_TYPE}} struct {{ENUM_TY}}(pub {{BASE_TYPE}});";
      code_ += "#[allow(non_upper_case_globals)]";
      code_ += "impl {{ENUM_TY}} {";
      ForAllEnumValues1(enum_def, [&](const EnumVal &ev) {
        this->GenComment(ev.doc_comment);
        code_ += "pub const {{VARIANT}}: Self = Self({{VALUE}});";
      });
      code_ += "";
      // Generate Associated constants
      code_ += "  pub const ENUM_MIN: {{BASE_TYPE}} = {{ENUM_MIN_BASE_VALUE}};";
      code_ += "  pub const ENUM_MAX: {{BASE_TYPE}} = {{ENUM_MAX_BASE_VALUE}};";
      code_ += "  pub const ENUM_VALUES: &'static [Self] = &[";
      ForAllEnumValues(enum_def, [&]() { code_ += "  Self::{{VARIANT}},"; });
      code_ += "  ];";
      code_ += "  /// Returns the variant's name or \"\" if unknown.";
      code_ += "  pub fn variant_name(self) -> Option<&'static str> {";
      code_ += "    match self {";
      ForAllEnumValues(enum_def, [&]() {
        code_ += "    Self::{{VARIANT}} => Some(\"{{VARIANT}}\"),";
      });
      code_ += "      _ => None,";
      code_ += "    }";
      code_ += "  }";
      code_ += "}";

      // Generate Debug. Unknown variants are printed like "<UNKNOWN 42>".
      code_ += "impl core::fmt::Debug for {{ENUM_TY}} {";
      code_ +=
          "  fn fmt(&self, f: &mut core::fmt::Formatter) ->"
          " core::fmt::Result {";
      code_ += "    if let Some(name) = self.variant_name() {";
      code_ += "      f.write_str(name)";
      code_ += "    } else {";
      code_ += "      f.write_fmt(format_args!(\"<UNKNOWN {:?}>\", self.0))";
      code_ += "    }";
      code_ += "  }";
      code_ += "}";

      code_.SetValue("INTO_BASE", "self.0");
    }

    // Implement serde::Serialize
    if (parser_.opts.rust_serialize) {
      code_ += "impl Serialize for {{ENUM_TY}} {";
      code_ +=
          "  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>";
      code_ += "  where";
      code_ += "    S: Serializer,";
      code_ += "  {";
      if (IsBitFlagsEnum(enum_def)) {
        code_ += "    serializer.serialize_u32(self.bits() as u32)";
      } else {
        code_ +=
            "    serializer.serialize_unit_variant(\"{{ENUM_TY}}\", self.0 "
            "as "
            "u32, self.variant_name().unwrap())";
      }
      code_ += "  }";
      code_ += "}";
      code_ += "";
    }

    // Generate Follow and Push so we can serialize and stuff.
    code_ += "impl<'a> flatbuffers::Follow<'a> for {{ENUM_TY}} {";
    code_ += "  type Inner = Self;";
    code_ += "  #[inline]";
    code_ += "  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {";
    code_ +=
        "    let b = flatbuffers::read_scalar_at::<{{BASE_TYPE}}>(buf, loc);";
    if (IsBitFlagsEnum(enum_def)) {
      // Safety:
      // This is safe because we know bitflags is implemented with a repr
      // transparent uint of the correct size. from_bits_unchecked will be
      // replaced by an equivalent but safe from_bits_retain in bitflags 2.0
      // https://github.com/bitflags/bitflags/issues/262
      code_ += "    // Safety:";
      code_ +=
          "    // This is safe because we know bitflags is implemented with a "
          "repr transparent uint of the correct size.";
      code_ +=
          "    // from_bits_unchecked will be replaced by an equivalent but "
          "safe from_bits_retain in bitflags 2.0";
      code_ += "    // https://github.com/bitflags/bitflags/issues/262";
      code_ += "    Self::from_bits_unchecked(b)";
    } else {
      code_ += "    Self(b)";
    }
    code_ += "  }";
    code_ += "}";
    code_ += "";
    code_ += "impl flatbuffers::Push for {{ENUM_TY}} {";
    code_ += "    type Output = {{ENUM_TY}};";
    code_ += "    #[inline]";
    code_ += "    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {";
    code_ +=
        "        flatbuffers::emplace_scalar::<{{BASE_TYPE}}>(dst, "
        "{{INTO_BASE}});";
    code_ += "    }";
    code_ += "}";
    code_ += "";
    code_ += "impl flatbuffers::EndianScalar for {{ENUM_TY}} {";
    code_ += "  type Scalar = {{BASE_TYPE}};";
    code_ += "  #[inline]";
    code_ += "  fn to_little_endian(self) -> {{BASE_TYPE}} {";
    code_ += "    {{INTO_BASE}}.to_le()";
    code_ += "  }";
    code_ += "  #[inline]";
    code_ += "  #[allow(clippy::wrong_self_convention)]";
    code_ += "  fn from_little_endian(v: {{BASE_TYPE}}) -> Self {";
    code_ += "    let b = {{BASE_TYPE}}::from_le(v);";
    if (IsBitFlagsEnum(enum_def)) {
      // Safety:
      // This is safe because we know bitflags is implemented with a repr
      // transparent uint of the correct size. from_bits_unchecked will be
      // replaced by an equivalent but safe from_bits_retain in bitflags 2.0
      // https://github.com/bitflags/bitflags/issues/262
      code_ += "    // Safety:";
      code_ +=
          "    // This is safe because we know bitflags is implemented with a "
          "repr transparent uint of the correct size.";
      code_ +=
          "    // from_bits_unchecked will be replaced by an equivalent but "
          "safe from_bits_retain in bitflags 2.0";
      code_ += "    // https://github.com/bitflags/bitflags/issues/262";
      code_ += "    unsafe { Self::from_bits_unchecked(b) }";
    } else {
      code_ += "    Self(b)";
    }
    code_ += "  }";
    code_ += "}";
    code_ += "";

    // Generate verifier - deferring to the base type.
    code_ += "impl<'a> flatbuffers::Verifiable for {{ENUM_TY}} {";
    code_ += "  #[inline]";
    code_ += "  fn run_verifier(";
    code_ += "    v: &mut flatbuffers::Verifier, pos: usize";
    code_ += "  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {";
    code_ += "    use self::flatbuffers::Verifiable;";
    code_ += "    {{BASE_TYPE}}::run_verifier(v, pos)";
    code_ += "  }";
    code_ += "}";
    code_ += "";
    // Enums are basically integers.
    code_ += "impl flatbuffers::SimpleToVerifyInSlice for {{ENUM_TY}} {}";

    if (enum_def.is_union) {
      // Generate typesafe offset(s) for unions
      code_.SetValue("UNION_TYPE", namer_.Type(enum_def));
      code_ += "{{ACCESS_TYPE}} struct {{UNION_TYPE}}UnionTableOffset {}";
      code_ += "";
      if (parser_.opts.generate_object_based_api) { GenUnionObject(enum_def); }
    }
  }

  // TODO(cneo): dedup Object versions from non object versions.
  void ForAllUnionObjectVariantsBesidesNone(const EnumDef &enum_def,
                                            std::function<void()> cb) {
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &enum_val = **it;
      if (enum_val.union_type.base_type == BASE_TYPE_NONE) continue;
      code_.SetValue("VARIANT_NAME", namer_.Variant(enum_val));
      // For legacy reasons, enum variants are Keep case while enum native
      // variants are UpperCamel case.
      code_.SetValue("NATIVE_VARIANT",
                     namer_.LegacyRustNativeVariant(enum_val));
      code_.SetValue("U_ELEMENT_NAME", namer_.Method(enum_val));
      code_.SetValue("U_ELEMENT_TABLE_TYPE",
                     NamespacedNativeName(*enum_val.union_type.struct_def));
      code_.IncrementIdentLevel();
      cb();
      code_.DecrementIdentLevel();
    }
  }
  void GenUnionObject(const EnumDef &enum_def) {
    code_.SetValue("ENUM_TY", namer_.Type(enum_def));
    code_.SetValue("ENUM_FN", namer_.Function(enum_def));
    code_.SetValue("ENUM_OTY", namer_.ObjectType(enum_def));

    // Generate native union.
    code_ += "#[allow(clippy::upper_case_acronyms)]";  // NONE's spelling is
                                                       // intended.
    code_ += "#[non_exhaustive]";
    code_ += "#[derive(Debug, Clone, PartialEq)]";
    code_ += "{{ACCESS_TYPE}} enum {{ENUM_OTY}} {";
    code_ += "  NONE,";
    ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
      code_ += "{{NATIVE_VARIANT}}(Box<{{U_ELEMENT_TABLE_TYPE}}>),";
    });
    code_ += "}";
    // Generate Default (NONE).
    code_ += "impl Default for {{ENUM_OTY}} {";
    code_ += "  fn default() -> Self {";
    code_ += "    Self::NONE";
    code_ += "  }";
    code_ += "}";

    // Generate native union methods.
    code_ += "impl {{ENUM_OTY}} {";

    // Get flatbuffers union key.
    // TODO(cneo): add docstrings?
    code_ += "  pub fn {{ENUM_FN}}_type(&self) -> {{ENUM_TY}} {";
    code_ += "    match self {";
    code_ += "      Self::NONE => {{ENUM_TY}}::NONE,";
    ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
      code_ +=
          "    Self::{{NATIVE_VARIANT}}(_) => {{ENUM_TY}}::"
          "{{VARIANT_NAME}},";
    });
    code_ += "    }";
    code_ += "  }";
    // Pack flatbuffers union value
    code_ +=
        "  pub fn pack<'b, A: flatbuffers::Allocator + 'b>(&self, fbb: &mut "
        "flatbuffers::FlatBufferBuilder<'b, A>)"
        " -> Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>"
        " {";
    code_ += "    match self {";
    code_ += "      Self::NONE => None,";
    ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
      code_ += "    Self::{{NATIVE_VARIANT}}(v) => \\";
      code_ += "Some(v.pack(fbb).as_union_value()),";
    });
    code_ += "    }";
    code_ += "  }";

    // Generate some accessors;
    ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
      // Move accessor.
      code_ +=
          "/// If the union variant matches, return the owned "
          "{{U_ELEMENT_TABLE_TYPE}}, setting the union to NONE.";
      code_ +=
          "pub fn take_{{U_ELEMENT_NAME}}(&mut self) -> "
          "Option<Box<{{U_ELEMENT_TABLE_TYPE}}>> {";
      code_ += "  if let Self::{{NATIVE_VARIANT}}(_) = self {";
      code_ += "    let v = core::mem::replace(self, Self::NONE);";
      code_ += "    if let Self::{{NATIVE_VARIANT}}(w) = v {";
      code_ += "      Some(w)";
      code_ += "    } else {";
      code_ += "      unreachable!()";
      code_ += "    }";
      code_ += "  } else {";
      code_ += "    None";
      code_ += "  }";
      code_ += "}";
      // Immutable reference accessor.
      code_ +=
          "/// If the union variant matches, return a reference to the "
          "{{U_ELEMENT_TABLE_TYPE}}.";
      code_ +=
          "pub fn as_{{U_ELEMENT_NAME}}(&self) -> "
          "Option<&{{U_ELEMENT_TABLE_TYPE}}> {";
      code_ +=
          "  if let Self::{{NATIVE_VARIANT}}(v) = self "
          "{ Some(v.as_ref()) } else { None }";
      code_ += "}";
      // Mutable reference accessor.
      code_ +=
          "/// If the union variant matches, return a mutable reference"
          " to the {{U_ELEMENT_TABLE_TYPE}}.";
      code_ +=
          "pub fn as_{{U_ELEMENT_NAME}}_mut(&mut self) -> "
          "Option<&mut {{U_ELEMENT_TABLE_TYPE}}> {";
      code_ +=
          "  if let Self::{{NATIVE_VARIANT}}(v) = self "
          "{ Some(v.as_mut()) } else { None }";
      code_ += "}";
    });
    code_ += "}";  // End union methods impl.
  }

  enum DefaultContext { kBuilder, kAccessor, kObject };
  std::string GetDefaultValue(const FieldDef &field,
                              const DefaultContext context) {
    if (context == kBuilder) {
      // Builders and Args structs model nonscalars "optional" even if they're
      // required or have defaults according to the schema. I guess its because
      // WIPOffset is not nullable.
      if (!IsScalar(field.value.type.base_type) || field.IsOptional()) {
        return "None";
      }
    } else {
      // This for defaults in objects.
      // Unions have a NONE variant instead of using Rust's None.
      if (field.IsOptional() && !IsUnion(field.value.type)) { return "None"; }
    }
    switch (GetFullType(field.value.type)) {
      case ftInteger: {
        return field.value.constant;
      }
      case ftFloat: {
        const std::string float_prefix =
            (field.value.type.base_type == BASE_TYPE_FLOAT) ? "f32::" : "f64::";
        if (StringIsFlatbufferNan(field.value.constant)) {
          return float_prefix + "NAN";
        } else if (StringIsFlatbufferPositiveInfinity(field.value.constant)) {
          return float_prefix + "INFINITY";
        } else if (StringIsFlatbufferNegativeInfinity(field.value.constant)) {
          return float_prefix + "NEG_INFINITY";
        }
        return field.value.constant;
      }
      case ftBool: {
        return field.value.constant == "0" ? "false" : "true";
      }
      case ftUnionKey:
      case ftEnumKey: {
        auto ev = field.value.type.enum_def->FindByValue(field.value.constant);
        if (!ev) return "Default::default()";  // Bitflags enum.
        return WrapInNameSpace(
            field.value.type.enum_def->defined_namespace,
            namer_.EnumVariant(*field.value.type.enum_def, *ev));
      }
      case ftUnionValue: {
        return ObjectFieldType(field, true) + "::NONE";
      }
      case ftString: {
        // Required fields do not have defaults defined by the schema, but we
        // need one for Rust's Default trait so we use empty string. The usual
        // value of field.value.constant is `0`, which is non-sensical except
        // maybe to c++ (nullptr == 0).
        // TODO: Escape strings?
        const std::string defval =
            field.IsRequired() ? "\"\"" : "\"" + field.value.constant + "\"";
        if (context == kObject) return defval + ".to_string()";
        if (context == kAccessor) return "&" + defval;
        FLATBUFFERS_ASSERT(false);
        return "INVALID_CODE_GENERATION";
      }

      case ftArrayOfStruct:
      case ftArrayOfEnum:
      case ftArrayOfBuiltin:
      case ftVectorOfBool:
      case ftVectorOfFloat:
      case ftVectorOfInteger:
      case ftVectorOfString:
      case ftVectorOfStruct:
      case ftVectorOfTable:
      case ftVectorOfEnumKey:
      case ftVectorOfUnionValue:
      case ftStruct:
      case ftTable: {
        // We only support empty vectors which matches the defaults for
        // &[T] and Vec<T> anyway.
        //
        // For required structs and tables fields, we defer to their object API
        // defaults. This works so long as there's nothing recursive happening,
        // but `table Infinity { i: Infinity (required); }` does compile.
        return "Default::default()";
      }
    }
    FLATBUFFERS_ASSERT(false);
    return "INVALID_CODE_GENERATION";
  }

  // Create the return type for fields in the *BuilderArgs structs that are
  // used to create Tables.
  //
  // Note: we could make all inputs to the BuilderArgs be an Option, as well
  // as all outputs. But, the UX of Flatbuffers is that the user doesn't get to
  // know if the value is default or not, because there are three ways to
  // return a default value:
  // 1) return a stored value that happens to be the default,
  // 2) return a hardcoded value because the relevant vtable field is not in
  //    the vtable, or
  // 3) return a hardcoded value because the vtable field value is set to zero.
  std::string TableBuilderArgsDefnType(const FieldDef &field,
                                       const std::string &lifetime) {
    const Type &type = field.value.type;
    auto WrapOption = [&](std::string s) {
      return IsOptionalToBuilder(field) ? "Option<" + s + ">" : s;
    };
    auto WrapVector = [&](std::string ty) {
      return WrapOption("flatbuffers::WIPOffset<flatbuffers::Vector<" +
                        lifetime + ", " + ty + ">>");
    };
    auto WrapUOffsetsVector = [&](std::string ty) {
      return WrapVector("flatbuffers::ForwardsUOffset<" + ty + ">");
    };

    switch (GetFullType(type)) {
      case ftInteger:
      case ftFloat:
      case ftBool: {
        return WrapOption(GetTypeBasic(type));
      }
      case ftStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption("&" + lifetime + " " + typname);
      }
      case ftTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption("flatbuffers::WIPOffset<" + typname + "<" + lifetime +
                          ">>");
      }
      case ftString: {
        return WrapOption("flatbuffers::WIPOffset<&" + lifetime + " str>");
      }
      case ftEnumKey:
      case ftUnionKey: {
        return WrapOption(WrapInNameSpace(*type.enum_def));
      }
      case ftUnionValue: {
        return "Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>";
      }

      case ftVectorOfInteger:
      case ftVectorOfBool:
      case ftVectorOfFloat: {
        const auto typname = GetTypeBasic(type.VectorType());
        return WrapVector(typname);
      }
      case ftVectorOfEnumKey: {
        const auto typname = WrapInNameSpace(*type.enum_def);
        return WrapVector(typname);
      }
      case ftVectorOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapVector(typname);
      }
      case ftVectorOfTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapUOffsetsVector(typname + "<" + lifetime + ">");
      }
      case ftVectorOfString: {
        return WrapUOffsetsVector("&" + lifetime + " str");
      }
      case ftVectorOfUnionValue: {
        return WrapUOffsetsVector("flatbuffers::Table<" + lifetime + ">");
      }
      case ftArrayOfEnum:
      case ftArrayOfStruct:
      case ftArrayOfBuiltin: {
        FLATBUFFERS_ASSERT(false && "arrays are not supported within tables");
        return "ARRAYS_NOT_SUPPORTED_IN_TABLES";
      }
    }
    return "INVALID_CODE_GENERATION";  // for return analysis
  }

  std::string ObjectFieldType(const FieldDef &field, bool in_a_table) {
    const Type &type = field.value.type;
    std::string ty;
    switch (GetFullType(type)) {
      case ftInteger:
      case ftBool:
      case ftFloat: {
        ty = GetTypeBasic(type);
        break;
      }
      case ftString: {
        ty = "String";
        break;
      }
      case ftStruct: {
        ty = NamespacedNativeName(*type.struct_def);
        break;
      }
      case ftTable: {
        // Since Tables can contain themselves, Box is required to avoid
        // infinite types.
        ty = "Box<" + NamespacedNativeName(*type.struct_def) + ">";
        break;
      }
      case ftUnionKey: {
        // There is no native "UnionKey", natively, unions are rust enums with
        // newtype-struct-variants.
        return "INVALID_CODE_GENERATION";
      }
      case ftUnionValue: {
        ty = NamespacedNativeName(*type.enum_def);
        break;
      }
      case ftEnumKey: {
        ty = WrapInNameSpace(*type.enum_def);
        break;
      }
      // Vectors are in tables and are optional
      case ftVectorOfEnumKey: {
        ty = "Vec<" + WrapInNameSpace(*type.VectorType().enum_def) + ">";
        break;
      }
      case ftVectorOfInteger:
      case ftVectorOfBool:
      case ftVectorOfFloat: {
        ty = "Vec<" + GetTypeBasic(type.VectorType()) + ">";
        break;
      }
      case ftVectorOfString: {
        ty = "Vec<String>";
        break;
      }
      case ftVectorOfTable:
      case ftVectorOfStruct: {
        ty = NamespacedNativeName(*type.VectorType().struct_def);
        ty = "Vec<" + ty + ">";
        break;
      }
      case ftVectorOfUnionValue: {
        FLATBUFFERS_ASSERT(false && "vectors of unions are not yet supported");
        return "INVALID_CODE_GENERATION";  // OH NO!
      }
      case ftArrayOfEnum: {
        ty = "[" + WrapInNameSpace(*type.VectorType().enum_def) + "; " +
             NumToString(type.fixed_length) + "]";
        break;
      }
      case ftArrayOfStruct: {
        ty = "[" + NamespacedNativeName(*type.VectorType().struct_def) + "; " +
             NumToString(type.fixed_length) + "]";
        break;
      }
      case ftArrayOfBuiltin: {
        ty = "[" + GetTypeBasic(type.VectorType()) + "; " +
             NumToString(type.fixed_length) + "]";
        break;
      }
    }
    if (in_a_table && !IsUnion(type) && field.IsOptional()) {
      return "Option<" + ty + ">";
    } else {
      return ty;
    }
  }

  std::string TableBuilderArgsAddFuncType(const FieldDef &field,
                                          const std::string &lifetime) {
    const Type &type = field.value.type;

    switch (GetFullType(field.value.type)) {
      case ftVectorOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime + ", " +
               typname + ">>";
      }
      case ftVectorOfTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime +
               ", flatbuffers::ForwardsUOffset<" + typname + "<" + lifetime +
               ">>>>";
      }
      case ftVectorOfInteger:
      case ftVectorOfBool:
      case ftVectorOfFloat: {
        const auto typname = GetTypeBasic(type.VectorType());
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime + ", " +
               typname + ">>";
      }
      case ftVectorOfString: {
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime +
               ", flatbuffers::ForwardsUOffset<&" + lifetime + " str>>>";
      }
      case ftVectorOfEnumKey: {
        const auto typname = WrapInNameSpace(*type.enum_def);
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime + ", " +
               typname + ">>";
      }
      case ftVectorOfUnionValue: {
        return "flatbuffers::WIPOffset<flatbuffers::Vector<" + lifetime +
               ", flatbuffers::ForwardsUOffset<flatbuffers::Table<" + lifetime +
               ">>>";
      }
      case ftEnumKey:
      case ftUnionKey: {
        const auto typname = WrapInNameSpace(*type.enum_def);
        return typname;
      }
      case ftStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "&" + typname + "";
      }
      case ftTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "flatbuffers::WIPOffset<" + typname + "<" + lifetime + ">>";
      }
      case ftInteger:
      case ftBool:
      case ftFloat: {
        return GetTypeBasic(type);
      }
      case ftString: {
        return "flatbuffers::WIPOffset<&" + lifetime + " str>";
      }
      case ftUnionValue: {
        return "flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>";
      }
      case ftArrayOfBuiltin: {
        const auto typname = GetTypeBasic(type.VectorType());
        return "flatbuffers::Array<" + lifetime + ", " + typname + ", " +
               NumToString(type.fixed_length) + ">";
      }
      case ftArrayOfEnum: {
        const auto typname = WrapInNameSpace(*type.enum_def);
        return "flatbuffers::Array<" + lifetime + ", " + typname + ", " +
               NumToString(type.fixed_length) + ">";
      }
      case ftArrayOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "flatbuffers::Array<" + lifetime + ", " + typname + ", " +
               NumToString(type.fixed_length) + ">";
      }
    }

    return "INVALID_CODE_GENERATION";  // for return analysis
  }

  std::string TableBuilderArgsAddFuncBody(const FieldDef &field) {
    const Type &type = field.value.type;

    switch (GetFullType(field.value.type)) {
      case ftInteger:
      case ftBool:
      case ftFloat: {
        const auto typname = GetTypeBasic(field.value.type);
        return (field.IsOptional() ? "self.fbb_.push_slot_always::<"
                                   : "self.fbb_.push_slot::<") +
               typname + ">";
      }
      case ftEnumKey:
      case ftUnionKey: {
        const auto underlying_typname = GetTypeBasic(type);
        return (field.IsOptional() ? "self.fbb_.push_slot_always::<"
                                   : "self.fbb_.push_slot::<") +
               underlying_typname + ">";
      }

      case ftStruct: {
        const std::string typname = WrapInNameSpace(*type.struct_def);
        return "self.fbb_.push_slot_always::<&" + typname + ">";
      }
      case ftTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return "self.fbb_.push_slot_always::<flatbuffers::WIPOffset<" +
               typname + ">>";
      }

      case ftUnionValue:
      case ftString:
      case ftVectorOfInteger:
      case ftVectorOfFloat:
      case ftVectorOfBool:
      case ftVectorOfEnumKey:
      case ftVectorOfStruct:
      case ftVectorOfTable:
      case ftVectorOfString:
      case ftVectorOfUnionValue: {
        return "self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>";
      }
      case ftArrayOfEnum:
      case ftArrayOfStruct:
      case ftArrayOfBuiltin: {
        FLATBUFFERS_ASSERT(false && "arrays are not supported within tables");
        return "ARRAYS_NOT_SUPPORTED_IN_TABLES";
      }
    }
    return "INVALID_CODE_GENERATION";  // for return analysis
  }

  std::string GenTableAccessorFuncReturnType(const FieldDef &field,
                                             const std::string &lifetime) {
    const Type &type = field.value.type;
    const auto WrapOption = [&](std::string s) {
      return field.IsOptional() ? "Option<" + s + ">" : s;
    };

    switch (GetFullType(field.value.type)) {
      case ftInteger:
      case ftFloat:
      case ftBool: {
        return WrapOption(GetTypeBasic(type));
      }
      case ftStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption("&" + lifetime + " " + typname);
      }
      case ftTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption(typname + "<" + lifetime + ">");
      }
      case ftEnumKey:
      case ftUnionKey: {
        return WrapOption(WrapInNameSpace(*type.enum_def));
      }

      case ftUnionValue: {
        return WrapOption("flatbuffers::Table<" + lifetime + ">");
      }
      case ftString: {
        return WrapOption("&" + lifetime + " str");
      }
      case ftVectorOfInteger:
      case ftVectorOfBool:
      case ftVectorOfFloat: {
        const auto typname = GetTypeBasic(type.VectorType());
        return WrapOption("flatbuffers::Vector<" + lifetime + ", " + typname +
                          ">");
      }
      case ftVectorOfEnumKey: {
        const auto typname = WrapInNameSpace(*type.enum_def);
        return WrapOption("flatbuffers::Vector<" + lifetime + ", " + typname +
                          ">");
      }
      case ftVectorOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption("flatbuffers::Vector<" + lifetime + ", " + typname +
                          ">");
      }
      case ftVectorOfTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapOption("flatbuffers::Vector<" + lifetime +
                          ", flatbuffers::ForwardsUOffset<" + typname + "<" +
                          lifetime + ">>>");
      }
      case ftVectorOfString: {
        return WrapOption("flatbuffers::Vector<" + lifetime +
                          ", flatbuffers::ForwardsUOffset<&" + lifetime +
                          " str>>");
      }
      case ftVectorOfUnionValue: {
        FLATBUFFERS_ASSERT(false && "vectors of unions are not yet supported");
        // TODO(rw): when we do support these, we should consider using the
        //           Into trait to convert tables to typesafe union values.
        return "INVALID_CODE_GENERATION";  // for return analysis
      }
      case ftArrayOfEnum:
      case ftArrayOfStruct:
      case ftArrayOfBuiltin: {
        FLATBUFFERS_ASSERT(false && "arrays are not supported within tables");
        return "ARRAYS_NOT_SUPPORTED_IN_TABLES";
      }
    }
    return "INVALID_CODE_GENERATION";  // for return analysis
  }

  std::string FollowType(const Type &type, const std::string &lifetime) {
    // IsVector... This can be made iterative?

    const auto WrapForwardsUOffset = [](std::string ty) -> std::string {
      return "flatbuffers::ForwardsUOffset<" + ty + ">";
    };
    const auto WrapVector = [&](std::string ty) -> std::string {
      return "flatbuffers::Vector<" + lifetime + ", " + ty + ">";
    };
    const auto WrapArray = [&](std::string ty, uint16_t length) -> std::string {
      return "flatbuffers::Array<" + lifetime + ", " + ty + ", " +
             NumToString(length) + ">";
    };
    switch (GetFullType(type)) {
      case ftInteger:
      case ftFloat:
      case ftBool: {
        return GetTypeBasic(type);
      }
      case ftStruct: {
        return WrapInNameSpace(*type.struct_def);
      }
      case ftUnionKey:
      case ftEnumKey: {
        return WrapInNameSpace(*type.enum_def);
      }
      case ftTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapForwardsUOffset(typname);
      }
      case ftUnionValue: {
        return WrapForwardsUOffset("flatbuffers::Table<" + lifetime + ">");
      }
      case ftString: {
        return WrapForwardsUOffset("&str");
      }
      case ftVectorOfInteger:
      case ftVectorOfBool:
      case ftVectorOfFloat: {
        const auto typname = GetTypeBasic(type.VectorType());
        return WrapForwardsUOffset(WrapVector(typname));
      }
      case ftVectorOfEnumKey: {
        const auto typname = WrapInNameSpace(*type.VectorType().enum_def);
        return WrapForwardsUOffset(WrapVector(typname));
      }
      case ftVectorOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapForwardsUOffset(WrapVector(typname));
      }
      case ftVectorOfTable: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapForwardsUOffset(WrapVector(WrapForwardsUOffset(typname)));
      }
      case ftVectorOfString: {
        return WrapForwardsUOffset(
            WrapVector(WrapForwardsUOffset("&" + lifetime + " str")));
      }
      case ftVectorOfUnionValue: {
        FLATBUFFERS_ASSERT(false && "vectors of unions are not yet supported");
        return "INVALID_CODE_GENERATION";  // for return analysis
      }
      case ftArrayOfEnum: {
        const auto typname = WrapInNameSpace(*type.VectorType().enum_def);
        return WrapArray(typname, type.fixed_length);
      }
      case ftArrayOfStruct: {
        const auto typname = WrapInNameSpace(*type.struct_def);
        return WrapArray(typname, type.fixed_length);
      }
      case ftArrayOfBuiltin: {
        const auto typname = GetTypeBasic(type.VectorType());
        return WrapArray(typname, type.fixed_length);
      }
    }
    return "INVALID_CODE_GENERATION";  // for return analysis
  }

  std::string GenTableAccessorFuncBody(const FieldDef &field,
                                       const std::string &lifetime) {
    const std::string vt_offset = namer_.LegacyRustFieldOffsetName(field);
    const std::string typname = FollowType(field.value.type, lifetime);
    // Default-y fields (scalars so far) are neither optional nor required.
    const std::string default_value =
        !(field.IsOptional() || field.IsRequired())
            ? "Some(" + GetDefaultValue(field, kAccessor) + ")"
            : "None";
    const std::string unwrap = field.IsOptional() ? "" : ".unwrap()";

    return "unsafe { self._tab.get::<" + typname +
           ">({{STRUCT_TY}}::" + vt_offset + ", " + default_value + ")" +
           unwrap + "}";
  }

  // Generates a fully-qualified name getter for use with --gen-name-strings
  void GenFullyQualifiedNameGetter(const StructDef &struct_def,
                                   const std::string &name) {
    const std::string fully_qualified_name =
        struct_def.defined_namespace->GetFullyQualifiedName(name);
    code_ += "  pub const fn get_fully_qualified_name() -> &'static str {";
    code_ += "    \"" + fully_qualified_name + "\"";
    code_ += "  }";
    code_ += "";
  }

  void ForAllUnionVariantsBesidesNone(
      const EnumDef &def, std::function<void(const EnumVal &ev)> cb) {
    FLATBUFFERS_ASSERT(def.is_union);

    for (auto it = def.Vals().begin(); it != def.Vals().end(); ++it) {
      const EnumVal &ev = **it;
      // TODO(cneo): Can variants be deprecated, should we skip them?
      if (ev.union_type.base_type == BASE_TYPE_NONE) { continue; }
      code_.SetValue(
          "U_ELEMENT_ENUM_TYPE",
          WrapInNameSpace(def.defined_namespace, namer_.EnumVariant(def, ev)));
      code_.SetValue(
          "U_ELEMENT_TABLE_TYPE",
          WrapInNameSpace(ev.union_type.struct_def->defined_namespace,
                          ev.union_type.struct_def->name));
      code_.SetValue("U_ELEMENT_NAME", namer_.Function(ev.name));
      cb(ev);
    }
  }

  void ForAllTableFields(const StructDef &struct_def,
                         std::function<void(const FieldDef &)> cb,
                         bool reversed = false) {
    // TODO(cneo): Remove `reversed` overload. It's only here to minimize the
    // diff when refactoring to the `ForAllX` helper functions.
    auto go = [&](const FieldDef &field) {
      if (field.deprecated) return;
      code_.SetValue("OFFSET_NAME", namer_.LegacyRustFieldOffsetName(field));
      code_.SetValue("OFFSET_VALUE", NumToString(field.value.offset));
      code_.SetValue("FIELD", namer_.Field(field));
      code_.SetValue("BLDR_DEF_VAL", GetDefaultValue(field, kBuilder));
      code_.SetValue("DISCRIMINANT", namer_.LegacyRustUnionTypeMethod(field));
      code_.IncrementIdentLevel();
      cb(field);
      code_.DecrementIdentLevel();
    };
    const auto &fields = struct_def.fields.vec;
    if (reversed) {
      for (auto it = fields.rbegin(); it != fields.rend(); ++it) go(**it);
    } else {
      for (auto it = fields.begin(); it != fields.end(); ++it) go(**it);
    }
  }
  // Generate an accessor struct, builder struct, and create function for a
  // table.
  void GenTable(const StructDef &struct_def) {
    const bool is_private =
        parser_.opts.no_leak_private_annotations &&
        (struct_def.attributes.Lookup("private") != nullptr);
    code_.SetValue("ACCESS_TYPE", is_private ? "pub(crate)" : "pub");
    code_.SetValue("STRUCT_TY", namer_.Type(struct_def));
    code_.SetValue("STRUCT_FN", namer_.Function(struct_def));

    // Generate an offset type, the base type, the Follow impl, and the
    // init_from_table impl.
    code_ += "{{ACCESS_TYPE}} enum {{STRUCT_TY}}Offset {}";
    code_ += "#[derive(Copy, Clone, PartialEq)]";
    code_ += "";

    GenComment(struct_def.doc_comment);

    code_ += "{{ACCESS_TYPE}} struct {{STRUCT_TY}}<'a> {";
    code_ += "  pub _tab: flatbuffers::Table<'a>,";
    code_ += "}";
    code_ += "";
    code_ += "impl<'a> flatbuffers::Follow<'a> for {{STRUCT_TY}}<'a> {";
    code_ += "  type Inner = {{STRUCT_TY}}<'a>;";
    code_ += "  #[inline]";
    code_ += "  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {";
    code_ += "    Self { _tab: flatbuffers::Table::new(buf, loc) }";
    code_ += "  }";
    code_ += "}";
    code_ += "";
    code_ += "impl<'a> {{STRUCT_TY}}<'a> {";

    // Generate field id constants.
    ForAllTableFields(struct_def, [&](const FieldDef &unused) {
      (void)unused;
      code_ +=
          "pub const {{OFFSET_NAME}}: flatbuffers::VOffsetT = "
          "{{OFFSET_VALUE}};";
    });
    code_ += "";

    if (parser_.opts.generate_name_strings) {
      GenFullyQualifiedNameGetter(struct_def, struct_def.name);
    }

    code_ += "  #[inline]";
    code_ +=
        "  pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> "
        "Self {";
    code_ += "    {{STRUCT_TY}} { _tab: table }";
    code_ += "  }";

    // Generate a convenient create* function that uses the above builder
    // to create a table in one function call.
    code_.SetValue("MAYBE_US", struct_def.fields.vec.size() == 0 ? "_" : "");
    code_.SetValue("MAYBE_LT",
                   TableBuilderArgsNeedsLifetime(struct_def) ? "<'args>" : "");
    code_ += "  #[allow(unused_mut)]";
    code_ +=
        "  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: "
        "flatbuffers::Allocator + 'bldr>(";
    code_ +=
        "    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,";
    code_ += "    {{MAYBE_US}}args: &'args {{STRUCT_TY}}Args{{MAYBE_LT}}";
    code_ += "  ) -> flatbuffers::WIPOffset<{{STRUCT_TY}}<'bldr>> {";

    code_ += "    let mut builder = {{STRUCT_TY}}Builder::new(_fbb);";
    for (size_t size = struct_def.sortbysize ? sizeof(largest_scalar_t) : 1;
         size; size /= 2) {
      ForAllTableFields(
          struct_def,
          [&](const FieldDef &field) {
            if (struct_def.sortbysize &&
                size != SizeOf(field.value.type.base_type))
              return;
            if (IsOptionalToBuilder(field)) {
              code_ +=
                  "  if let Some(x) = args.{{FIELD}} "
                  "{ builder.add_{{FIELD}}(x); }";
            } else {
              code_ += "  builder.add_{{FIELD}}(args.{{FIELD}});";
            }
          },
          /*reverse=*/true);
    }
    code_ += "    builder.finish()";
    code_ += "  }";
    code_ += "";
    // Generate Object API Packer function.
    if (parser_.opts.generate_object_based_api) {
      // TODO(cneo): Replace more for loops with ForAllX stuff.
      // TODO(cneo): Manage indentation with IncrementIdentLevel?
      code_.SetValue("STRUCT_OTY", namer_.ObjectType(struct_def));
      code_ += "  pub fn unpack(&self) -> {{STRUCT_OTY}} {";
      ForAllObjectTableFields(struct_def, [&](const FieldDef &field) {
        const Type &type = field.value.type;
        switch (GetFullType(type)) {
          case ftInteger:
          case ftBool:
          case ftFloat:
          case ftEnumKey: {
            code_ += "  let {{FIELD}} = self.{{FIELD}}();";
            return;
          }
          case ftUnionKey: return;
          case ftUnionValue: {
            const auto &enum_def = *type.enum_def;
            code_.SetValue("ENUM_TY", WrapInNameSpace(enum_def));
            code_.SetValue("NATIVE_ENUM_NAME", NamespacedNativeName(enum_def));
            code_.SetValue("UNION_TYPE_METHOD",
                           namer_.LegacyRustUnionTypeMethod(field));

            code_ += "  let {{FIELD}} = match self.{{UNION_TYPE_METHOD}}() {";
            code_ += "    {{ENUM_TY}}::NONE => {{NATIVE_ENUM_NAME}}::NONE,";
            ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
              code_ +=
                  "  {{ENUM_TY}}::{{VARIANT_NAME}} => "
                  "{{NATIVE_ENUM_NAME}}::{{NATIVE_VARIANT}}(Box::new(";
              code_ += "    self.{{FIELD}}_as_{{U_ELEMENT_NAME}}()";
              code_ +=
                  "        .expect(\"Invalid union table, "
                  "expected `{{ENUM_TY}}::{{VARIANT_NAME}}`.\")";
              code_ += "        .unpack()";
              code_ += "  )),";
            });
            // Maybe we shouldn't throw away unknown discriminants?
            code_ += "    _ => {{NATIVE_ENUM_NAME}}::NONE,";
            code_ += "  };";
            return;
          }
          // The rest of the types need special handling based on if the field
          // is optional or not.
          case ftString: {
            code_.SetValue("EXPR", "x.to_string()");
            break;
          }
          case ftStruct: {
            code_.SetValue("EXPR", "x.unpack()");
            break;
          }
          case ftTable: {
            code_.SetValue("EXPR", "Box::new(x.unpack())");
            break;
          }
          case ftVectorOfInteger:
          case ftVectorOfBool:
          case ftVectorOfFloat:
          case ftVectorOfEnumKey: {
            code_.SetValue("EXPR", "x.into_iter().collect()");
            break;
          }
          case ftVectorOfString: {
            code_.SetValue("EXPR", "x.iter().map(|s| s.to_string()).collect()");
            break;
          }
          case ftVectorOfStruct:
          case ftVectorOfTable: {
            code_.SetValue("EXPR", "x.iter().map(|t| t.unpack()).collect()");
            break;
          }
          case ftVectorOfUnionValue: {
            FLATBUFFERS_ASSERT(false && "vectors of unions not yet supported");
            return;
          }
          case ftArrayOfEnum:
          case ftArrayOfStruct:
          case ftArrayOfBuiltin: {
            FLATBUFFERS_ASSERT(false &&
                               "arrays are not supported within tables");
            return;
          }
        }
        if (field.IsOptional()) {
          code_ += "  let {{FIELD}} = self.{{FIELD}}().map(|x| {";
          code_ += "    {{EXPR}}";
          code_ += "  });";
        } else {
          code_ += "  let {{FIELD}} = {";
          code_ += "    let x = self.{{FIELD}}();";
          code_ += "    {{EXPR}}";
          code_ += "  };";
        }
      });
      code_ += "    {{STRUCT_OTY}} {";
      ForAllObjectTableFields(struct_def, [&](const FieldDef &field) {
        if (field.value.type.base_type == BASE_TYPE_UTYPE) return;
        code_ += "    {{FIELD}},";
      });
      code_ += "    }";
      code_ += "  }";
    }

    if (struct_def.fields.vec.size() > 0) code_ += "";

    // Generate the accessors. Each has one of two forms:
    //
    // If a value can be None:
    //   pub fn name(&'a self) -> Option<user_facing_type> {
    //     self._tab.get::<internal_type>(offset, defaultval)
    //   }
    //
    // If a value is always Some:
    //   pub fn name(&'a self) -> user_facing_type {
    //     self._tab.get::<internal_type>(offset, defaultval).unwrap()
    //   }
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      code_.SetValue("RETURN_TYPE",
                     GenTableAccessorFuncReturnType(field, "'a"));

      this->GenComment(field.doc_comment);
      code_ += "#[inline]";
      code_ += "pub fn {{FIELD}}(&self) -> {{RETURN_TYPE}} {";
      code_ += "  // Safety:";
      code_ += "  // Created from valid Table for this object";
      code_ += "  // which contains a valid value in this slot";
      code_ += "  " + GenTableAccessorFuncBody(field, "'a");
      code_ += "}";

      // Generate a comparison function for this field if it is a key.
      if (field.key) { GenKeyFieldMethods(field); }

      // Generate a nested flatbuffer field, if applicable.
      auto nested = field.attributes.Lookup("nested_flatbuffer");
      if (nested) {
        std::string qualified_name = nested->constant;
        auto nested_root = parser_.LookupStruct(nested->constant);
        if (nested_root == nullptr) {
          qualified_name = parser_.current_namespace_->GetFullyQualifiedName(
              nested->constant);
          nested_root = parser_.LookupStruct(qualified_name);
        }
        FLATBUFFERS_ASSERT(nested_root);  // Guaranteed to exist by parser.

        code_.SetValue("NESTED", WrapInNameSpace(*nested_root));
        code_ += "pub fn {{FIELD}}_nested_flatbuffer(&'a self) -> \\";
        if (field.IsRequired()) {
          code_ += "{{NESTED}}<'a> {";
          code_ += "  let data = self.{{FIELD}}();";
          code_ += "  use flatbuffers::Follow;";
          code_ += "  // Safety:";
          code_ += "  // Created from a valid Table for this object";
          code_ += "  // Which contains a valid flatbuffer in this slot";
          code_ +=
              "  unsafe { <flatbuffers::ForwardsUOffset<{{NESTED}}<'a>>>"
              "::follow(data.bytes(), 0) }";
        } else {
          code_ += "Option<{{NESTED}}<'a>> {";
          code_ += "  self.{{FIELD}}().map(|data| {";
          code_ += "    use flatbuffers::Follow;";
          code_ += "    // Safety:";
          code_ += "    // Created from a valid Table for this object";
          code_ += "    // Which contains a valid flatbuffer in this slot";
          code_ +=
              "    unsafe { <flatbuffers::ForwardsUOffset<{{NESTED}}<'a>>>"
              "::follow(data.bytes(), 0) }";
          code_ += "  })";
        }
        code_ += "}";
      }
    });

    // Explicit specializations for union accessors
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      if (field.value.type.base_type != BASE_TYPE_UNION) return;
      ForAllUnionVariantsBesidesNone(
          *field.value.type.enum_def, [&](const EnumVal &unused) {
            (void)unused;
            code_ += "#[inline]";
            code_ += "#[allow(non_snake_case)]";
            code_ +=
                "pub fn {{FIELD}}_as_{{U_ELEMENT_NAME}}(&self) -> "
                "Option<{{U_ELEMENT_TABLE_TYPE}}<'a>> {";
            // If the user defined schemas name a field that clashes with a
            // language reserved word, flatc will try to escape the field name
            // by appending an underscore. This works well for most cases,
            // except one. When generating union accessors (and referring to
            // them internally within the code generated here), an extra
            // underscore will be appended to the name, causing build failures.
            //
            // This only happens when unions have members that overlap with
            // language reserved words.
            //
            // To avoid this problem the type field name is used unescaped here:
            code_ +=
                "  if self.{{DISCRIMINANT}}() == {{U_ELEMENT_ENUM_TYPE}} {";

            // The following logic is not tested in the integration test,
            // as of April 10, 2020
            if (field.IsRequired()) {
              code_ += "    let u = self.{{FIELD}}();";
              code_ += "    // Safety:";
              code_ += "    // Created from a valid Table for this object";
              code_ += "    // Which contains a valid union in this slot";
              code_ +=
                  "    Some(unsafe { "
                  "{{U_ELEMENT_TABLE_TYPE}}::init_from_table(u) })";
            } else {
              code_ += "    self.{{FIELD}}().map(|t| {";
              code_ += "     // Safety:";
              code_ += "     // Created from a valid Table for this object";
              code_ += "     // Which contains a valid union in this slot";
              code_ +=
                  "     unsafe { {{U_ELEMENT_TABLE_TYPE}}::init_from_table(t) "
                  "}";
              code_ += "   })";
            }
            code_ += "  } else {";
            code_ += "    None";
            code_ += "  }";
            code_ += "}";
            code_ += "";
          });
    });
    code_ += "}";  // End of table impl.
    code_ += "";

    // Generate Verifier;
    code_ += "impl flatbuffers::Verifiable for {{STRUCT_TY}}<'_> {";
    code_ += "  #[inline]";
    code_ += "  fn run_verifier(";
    code_ += "    v: &mut flatbuffers::Verifier, pos: usize";
    code_ += "  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {";
    code_ += "    use self::flatbuffers::Verifiable;";
    code_ += "    v.visit_table(pos)?\\";
    // Escape newline and insert it onthe next line so we can end the builder
    // with a nice semicolon.
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      if (GetFullType(field.value.type) == ftUnionKey) return;

      code_.SetValue("IS_REQ", field.IsRequired() ? "true" : "false");
      if (GetFullType(field.value.type) != ftUnionValue) {
        // All types besides unions.
        code_.SetValue("TY", FollowType(field.value.type, "'_"));
        code_ +=
            "\n     .visit_field::<{{TY}}>(\"{{FIELD}}\", "
            "Self::{{OFFSET_NAME}}, {{IS_REQ}})?\\";
        return;
      }
      // Unions.
      const EnumDef &union_def = *field.value.type.enum_def;
      code_.SetValue("UNION_TYPE", WrapInNameSpace(union_def));
      code_.SetValue("UNION_TYPE_OFFSET_NAME",
                     namer_.LegacyRustUnionTypeOffsetName(field));
      code_.SetValue("UNION_TYPE_METHOD",
                     namer_.LegacyRustUnionTypeMethod(field));
      code_ +=
          "\n     .visit_union::<{{UNION_TYPE}}, _>("
          "\"{{UNION_TYPE_METHOD}}\", Self::{{UNION_TYPE_OFFSET_NAME}}, "
          "\"{{FIELD}}\", Self::{{OFFSET_NAME}}, {{IS_REQ}}, "
          "|key, v, pos| {";
      code_ += "      match key {";
      ForAllUnionVariantsBesidesNone(union_def, [&](const EnumVal &unused) {
        (void)unused;
        code_ +=
            "        {{U_ELEMENT_ENUM_TYPE}} => v.verify_union_variant::"
            "<flatbuffers::ForwardsUOffset<{{U_ELEMENT_TABLE_TYPE}}>>("
            "\"{{U_ELEMENT_ENUM_TYPE}}\", pos),";
      });
      code_ += "        _ => Ok(()),";
      code_ += "      }";
      code_ += "   })?\\";
    });
    code_ += "\n     .finish();";
    code_ += "    Ok(())";
    code_ += "  }";
    code_ += "}";

    // Generate an args struct:
    code_.SetValue("MAYBE_LT",
                   TableBuilderArgsNeedsLifetime(struct_def) ? "<'a>" : "");
    code_ += "{{ACCESS_TYPE}} struct {{STRUCT_TY}}Args{{MAYBE_LT}} {";
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      code_.SetValue("PARAM_TYPE", TableBuilderArgsDefnType(field, "'a"));
      code_ += "  pub {{FIELD}}: {{PARAM_TYPE}},";
    });
    code_ += "}";

    // Generate an impl of Default for the *Args type:
    code_ += "impl<'a> Default for {{STRUCT_TY}}Args{{MAYBE_LT}} {";
    code_ += "  #[inline]";
    code_ += "  fn default() -> Self {";
    code_ += "    {{STRUCT_TY}}Args {";
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      code_ += "    {{FIELD}}: {{BLDR_DEF_VAL}},\\";
      code_ += field.IsRequired() ? " // required field" : "";
    });
    code_ += "    }";
    code_ += "  }";
    code_ += "}";
    code_ += "";

    // Implement serde::Serialize
    if (parser_.opts.rust_serialize) {
      const auto numFields = struct_def.fields.vec.size();
      code_.SetValue("NUM_FIELDS", NumToString(numFields));
      code_ += "impl Serialize for {{STRUCT_TY}}<'_> {";
      code_ +=
          "  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>";
      code_ += "  where";
      code_ += "    S: Serializer,";
      code_ += "  {";
      if (numFields == 0) {
        code_ +=
            "    let s = serializer.serialize_struct(\"{{STRUCT_TY}}\", 0)?;";
      } else {
        code_ +=
            "    let mut s = serializer.serialize_struct(\"{{STRUCT_TY}}\", "
            "{{NUM_FIELDS}})?;";
      }
      ForAllTableFields(struct_def, [&](const FieldDef &field) {
        const Type &type = field.value.type;
        if (IsUnion(type)) {
          if (type.base_type == BASE_TYPE_UNION) {
            const auto &enum_def = *type.enum_def;
            code_.SetValue("ENUM_TY", WrapInNameSpace(enum_def));
            code_.SetValue("FIELD", namer_.Field(field));
            code_.SetValue("UNION_TYPE_METHOD",
                           namer_.LegacyRustUnionTypeMethod(field));

            code_ += "    match self.{{UNION_TYPE_METHOD}}() {";
            code_ += "      {{ENUM_TY}}::NONE => (),";
            ForAllUnionObjectVariantsBesidesNone(enum_def, [&] {
              code_.SetValue("FIELD", namer_.Field(field));
              code_ += "      {{ENUM_TY}}::{{VARIANT_NAME}} => {";
              code_ +=
                  "        let f = "
                  "self.{{FIELD}}_as_{{U_ELEMENT_NAME}}()";
              code_ +=
                  "          .expect(\"Invalid union table, expected "
                  "`{{ENUM_TY}}::{{VARIANT_NAME}}`.\");";
              code_ += "        s.serialize_field(\"{{FIELD}}\", &f)?;";
              code_ += "      }";
            });
            code_ += "      _ => unimplemented!(),";
            code_ += "    }";
          } else {
            code_ +=
                "    s.serialize_field(\"{{FIELD}}\", "
                "&self.{{FIELD}}())?;";
          }
        } else {
          if (field.IsOptional()) {
            code_ += "    if let Some(f) = self.{{FIELD}}() {";
            code_ += "      s.serialize_field(\"{{FIELD}}\", &f)?;";
            code_ += "    } else {";
            code_ += "      s.skip_field(\"{{FIELD}}\")?;";
            code_ += "    }";
          } else {
            code_ +=
                "    s.serialize_field(\"{{FIELD}}\", "
                "&self.{{FIELD}}())?;";
          }
        }
      });
      code_ += "    s.end()";
      code_ += "  }";
      code_ += "}";
      code_ += "";
    }

    // Generate a builder struct:
    code_ +=
        "{{ACCESS_TYPE}} struct {{STRUCT_TY}}Builder<'a: 'b, 'b, A: "
        "flatbuffers::Allocator + 'a> {";
    code_ += "  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,";
    code_ +=
        "  start_: flatbuffers::WIPOffset<"
        "flatbuffers::TableUnfinishedWIPOffset>,";
    code_ += "}";

    // Generate builder functions:
    code_ +=
        "impl<'a: 'b, 'b, A: flatbuffers::Allocator + 'a> "
        "{{STRUCT_TY}}Builder<'a, "
        "'b, A> {";
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      const bool is_scalar = IsScalar(field.value.type.base_type);
      std::string offset = namer_.LegacyRustFieldOffsetName(field);
      // Generate functions to add data, which take one of two forms.
      //
      // If a value has a default:
      //   fn add_x(x_: type) {
      //     fbb_.push_slot::<type>(offset, x_, Some(default));
      //   }
      //
      // If a value does not have a default:
      //   fn add_x(x_: type) {
      //     fbb_.push_slot_always::<type>(offset, x_);
      //   }
      code_.SetValue("FIELD_OFFSET", namer_.Type(struct_def) + "::" + offset);
      code_.SetValue("FIELD_TYPE", TableBuilderArgsAddFuncType(field, "'b "));
      code_.SetValue("FUNC_BODY", TableBuilderArgsAddFuncBody(field));
      code_ += "#[inline]";
      code_ +=
          "pub fn add_{{FIELD}}(&mut self, {{FIELD}}: "
          "{{FIELD_TYPE}}) {";
      if (is_scalar && !field.IsOptional()) {
        code_ +=
            "  {{FUNC_BODY}}({{FIELD_OFFSET}}, {{FIELD}}, "
            "{{BLDR_DEF_VAL}});";
      } else {
        code_ += "  {{FUNC_BODY}}({{FIELD_OFFSET}}, {{FIELD}});";
      }
      code_ += "}";
    });

    // Struct initializer (all fields required);
    code_ += "  #[inline]";
    code_ +=
        "  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>) -> "
        "{{STRUCT_TY}}Builder<'a, 'b, A> {";
    code_.SetValue("NUM_FIELDS", NumToString(struct_def.fields.vec.size()));
    code_ += "    let start = _fbb.start_table();";
    code_ += "    {{STRUCT_TY}}Builder {";
    code_ += "      fbb_: _fbb,";
    code_ += "      start_: start,";
    code_ += "    }";
    code_ += "  }";

    // finish() function.
    code_ += "  #[inline]";
    code_ +=
        "  pub fn finish(self) -> "
        "flatbuffers::WIPOffset<{{STRUCT_TY}}<'a>> {";
    code_ += "    let o = self.fbb_.end_table(self.start_);";

    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      if (!field.IsRequired()) return;
      code_ +=
          "  self.fbb_.required(o, {{STRUCT_TY}}::{{OFFSET_NAME}},"
          "\"{{FIELD}}\");";
    });
    code_ += "    flatbuffers::WIPOffset::new(o.value())";
    code_ += "  }";
    code_ += "}";
    code_ += "";

    code_ += "impl core::fmt::Debug for {{STRUCT_TY}}<'_> {";
    code_ +=
        "  fn fmt(&self, f: &mut core::fmt::Formatter<'_>"
        ") -> core::fmt::Result {";
    code_ += "    let mut ds = f.debug_struct(\"{{STRUCT_TY}}\");";
    ForAllTableFields(struct_def, [&](const FieldDef &field) {
      if (GetFullType(field.value.type) == ftUnionValue) {
        // Generate a match statement to handle unions properly.
        code_.SetValue("KEY_TYPE", GenTableAccessorFuncReturnType(field, ""));
        code_.SetValue("UNION_ERR",
                       "&\"InvalidFlatbuffer: Union discriminant"
                       " does not match value.\"");

        code_ += "    match self.{{DISCRIMINANT}}() {";
        ForAllUnionVariantsBesidesNone(
            *field.value.type.enum_def, [&](const EnumVal &unused) {
              (void)unused;
              code_ += "      {{U_ELEMENT_ENUM_TYPE}} => {";
              code_ +=
                  "        if let Some(x) = "
                  "self.{{FIELD}}_as_"
                  "{{U_ELEMENT_NAME}}() {";
              code_ += "          ds.field(\"{{FIELD}}\", &x)";
              code_ += "        } else {";
              code_ += "          ds.field(\"{{FIELD}}\", {{UNION_ERR}})";
              code_ += "        }";
              code_ += "      },";
            });
        code_ += "      _ => {";
        code_ += "        let x: Option<()> = None;";
        code_ += "        ds.field(\"{{FIELD}}\", &x)";
        code_ += "      },";
        code_ += "    };";
      } else {
        // Most fields.
        code_ += "    ds.field(\"{{FIELD}}\", &self.{{FIELD}}());";
      }
    });
    code_ += "      ds.finish()";
    code_ += "  }";
    code_ += "}";
  }

  void GenTableObject(const StructDef &table) {
    code_.SetValue("STRUCT_OTY", namer_.ObjectType(table));
    code_.SetValue("STRUCT_TY", namer_.Type(table));

    // Generate the native object.
    code_ += "#[non_exhaustive]";
    code_ += "#[derive(Debug, Clone, PartialEq)]";
    code_ += "{{ACCESS_TYPE}} struct {{STRUCT_OTY}} {";
    ForAllObjectTableFields(table, [&](const FieldDef &field) {
      // Union objects combine both the union discriminant and value, so we
      // skip making a field for the discriminant.
      if (field.value.type.base_type == BASE_TYPE_UTYPE) return;
      code_ += "pub {{FIELD}}: {{FIELD_OTY}},";
    });
    code_ += "}";

    code_ += "impl Default for {{STRUCT_OTY}} {";
    code_ += "  fn default() -> Self {";
    code_ += "    Self {";
    ForAllObjectTableFields(table, [&](const FieldDef &field) {
      if (field.value.type.base_type == BASE_TYPE_UTYPE) return;
      std::string default_value = GetDefaultValue(field, kObject);
      code_ += "    {{FIELD}}: " + default_value + ",";
    });
    code_ += "    }";
    code_ += "  }";
    code_ += "}";

    // TODO(cneo): Generate defaults for Native tables. However, since structs
    // may be required, they, and therefore enums need defaults.

    // Generate pack function.
    code_ += "impl {{STRUCT_OTY}} {";
    code_ += "  pub fn pack<'b, A: flatbuffers::Allocator + 'b>(";
    code_ += "    &self,";
    code_ += "    _fbb: &mut flatbuffers::FlatBufferBuilder<'b, A>";
    code_ += "  ) -> flatbuffers::WIPOffset<{{STRUCT_TY}}<'b>> {";
    // First we generate variables for each field and then later assemble them
    // using "StructArgs" to more easily manage ownership of the builder.
    ForAllObjectTableFields(table, [&](const FieldDef &field) {
      const Type &type = field.value.type;
      switch (GetFullType(type)) {
        case ftInteger:
        case ftBool:
        case ftFloat:
        case ftEnumKey: {
          code_ += "  let {{FIELD}} = self.{{FIELD}};";
          return;
        }
        case ftUnionKey: return;  // Generate union type with union value.
        case ftUnionValue: {
          code_.SetValue("ENUM_METHOD",
                         namer_.Method(*field.value.type.enum_def));
          code_.SetValue("DISCRIMINANT",
                         namer_.LegacyRustUnionTypeMethod(field));
          code_ +=
              "  let {{DISCRIMINANT}} = "
              "self.{{FIELD}}.{{ENUM_METHOD}}_type();";
          code_ += "  let {{FIELD}} = self.{{FIELD}}.pack(_fbb);";
          return;
        }
        // The rest of the types require special casing around optionalness
        // due to "required" annotation.
        case ftString: {
          MapNativeTableField(field, "_fbb.create_string(x)");
          return;
        }
        case ftStruct: {
          // Hold the struct in a variable so we can reference it.
          if (field.IsRequired()) {
            code_ += "  let {{FIELD}}_tmp = Some(self.{{FIELD}}.pack());";
          } else {
            code_ +=
                "  let {{FIELD}}_tmp = self.{{FIELD}}"
                ".as_ref().map(|x| x.pack());";
          }
          code_ += "  let {{FIELD}} = {{FIELD}}_tmp.as_ref();";

          return;
        }
        case ftTable: {
          MapNativeTableField(field, "x.pack(_fbb)");
          return;
        }
        case ftVectorOfEnumKey:
        case ftVectorOfInteger:
        case ftVectorOfBool:
        case ftVectorOfFloat: {
          MapNativeTableField(field, "_fbb.create_vector(x)");
          return;
        }
        case ftVectorOfStruct: {
          MapNativeTableField(
              field,
              "let w: Vec<_> = x.iter().map(|t| t.pack()).collect();"
              "_fbb.create_vector(&w)");
          return;
        }
        case ftVectorOfString: {
          // TODO(cneo): create_vector* should be more generic to avoid
          // allocations.

          MapNativeTableField(field,
                              "let w: Vec<_> = x.iter().map(|s| "
                              "_fbb.create_string(s)).collect();"
                              "_fbb.create_vector(&w)");
          return;
        }
        case ftVectorOfTable: {
          MapNativeTableField(
              field,
              "let w: Vec<_> = x.iter().map(|t| t.pack(_fbb)).collect();"
              "_fbb.create_vector(&w)");
          return;
        }
        case ftVectorOfUnionValue: {
          FLATBUFFERS_ASSERT(false && "vectors of unions not yet supported");
          return;
        }
        case ftArrayOfEnum:
        case ftArrayOfStruct:
        case ftArrayOfBuiltin: {
          FLATBUFFERS_ASSERT(false && "arrays are not supported within tables");
          return;
        }
      }
    });
    code_ += "    {{STRUCT_TY}}::create(_fbb, &{{STRUCT_TY}}Args{";
    ForAllObjectTableFields(table, [&](const FieldDef &field) {
      (void)field;  // Unused.
      code_ += "    {{FIELD}},";
    });
    code_ += "    })";
    code_ += "  }";
    code_ += "}";
  }
  void ForAllObjectTableFields(const StructDef &table,
                               std::function<void(const FieldDef &)> cb) {
    const std::vector<FieldDef *> &v = table.fields.vec;
    for (auto it = v.begin(); it != v.end(); it++) {
      const FieldDef &field = **it;
      if (field.deprecated) continue;
      code_.SetValue("FIELD", namer_.Field(field));
      code_.SetValue("FIELD_OTY", ObjectFieldType(field, true));
      code_.IncrementIdentLevel();
      cb(field);
      code_.DecrementIdentLevel();
    }
  }
  void MapNativeTableField(const FieldDef &field, const std::string &expr) {
    if (field.IsOptional()) {
      code_ += "  let {{FIELD}} = self.{{FIELD}}.as_ref().map(|x|{";
      code_ += "    " + expr;
      code_ += "  });";
    } else {
      // For some reason Args has optional types for required fields.
      // TODO(cneo): Fix this... but its a breaking change?
      code_ += "  let {{FIELD}} = Some({";
      code_ += "    let x = &self.{{FIELD}};";
      code_ += "    " + expr;
      code_ += "  });";
    }
  }

  // Generate functions to compare tables and structs by key. This function
  // must only be called if the field key is defined.
  void GenKeyFieldMethods(const FieldDef &field) {
    FLATBUFFERS_ASSERT(field.key);

    code_.SetValue("KEY_TYPE", GenTableAccessorFuncReturnType(field, ""));
    code_.SetValue("REF", IsString(field.value.type) ? "" : "&");

    code_ += "#[inline]";
    code_ +=
        "pub fn key_compare_less_than(&self, o: &{{STRUCT_TY}}) -> "
        "bool {";
    code_ += "  self.{{FIELD}}() < o.{{FIELD}}()";
    code_ += "}";
    code_ += "";
    code_ += "#[inline]";
    code_ +=
        "pub fn key_compare_with_value(&self, val: {{KEY_TYPE}}) -> "
        "::core::cmp::Ordering {";
    code_ += "  let key = self.{{FIELD}}();";
    code_ += "  key.cmp({{REF}}val)";
    code_ += "}";
  }

  // Generate functions for accessing the root table object. This function
  // must only be called if the root table is defined.
  void GenRootTableFuncs(const StructDef &struct_def) {
    FLATBUFFERS_ASSERT(parser_.root_struct_def_ && "root table not defined");
    code_.SetValue("STRUCT_TY", namer_.Type(struct_def));
    code_.SetValue("STRUCT_FN", namer_.Function(struct_def));
    code_.SetValue("STRUCT_CONST", namer_.Constant(struct_def.name));

    // Default verifier root fns.
    code_ += "#[inline]";
    code_ += "/// Verifies that a buffer of bytes contains a `{{STRUCT_TY}}`";
    code_ += "/// and returns it.";
    code_ += "/// Note that verification is still experimental and may not";
    code_ += "/// catch every error, or be maximally performant. For the";
    code_ += "/// previous, unchecked, behavior use";
    code_ += "/// `root_as_{{STRUCT_FN}}_unchecked`.";
    code_ +=
        "pub fn root_as_{{STRUCT_FN}}(buf: &[u8]) "
        "-> Result<{{STRUCT_TY}}, flatbuffers::InvalidFlatbuffer> {";
    code_ += "  flatbuffers::root::<{{STRUCT_TY}}>(buf)";
    code_ += "}";
    code_ += "#[inline]";
    code_ += "/// Verifies that a buffer of bytes contains a size prefixed";
    code_ += "/// `{{STRUCT_TY}}` and returns it.";
    code_ += "/// Note that verification is still experimental and may not";
    code_ += "/// catch every error, or be maximally performant. For the";
    code_ += "/// previous, unchecked, behavior use";
    code_ += "/// `size_prefixed_root_as_{{STRUCT_FN}}_unchecked`.";
    code_ +=
        "pub fn size_prefixed_root_as_{{STRUCT_FN}}"
        "(buf: &[u8]) -> Result<{{STRUCT_TY}}, "
        "flatbuffers::InvalidFlatbuffer> {";
    code_ += "  flatbuffers::size_prefixed_root::<{{STRUCT_TY}}>(buf)";
    code_ += "}";
    // Verifier with options root fns.
    code_ += "#[inline]";
    code_ += "/// Verifies, with the given options, that a buffer of bytes";
    code_ += "/// contains a `{{STRUCT_TY}}` and returns it.";
    code_ += "/// Note that verification is still experimental and may not";
    code_ += "/// catch every error, or be maximally performant. For the";
    code_ += "/// previous, unchecked, behavior use";
    code_ += "/// `root_as_{{STRUCT_FN}}_unchecked`.";
    code_ += "pub fn root_as_{{STRUCT_FN}}_with_opts<'b, 'o>(";
    code_ += "  opts: &'o flatbuffers::VerifierOptions,";
    code_ += "  buf: &'b [u8],";
    code_ +=
        ") -> Result<{{STRUCT_TY}}<'b>, flatbuffers::InvalidFlatbuffer>"
        " {";
    code_ += "  flatbuffers::root_with_opts::<{{STRUCT_TY}}<'b>>(opts, buf)";
    code_ += "}";
    code_ += "#[inline]";
    code_ += "/// Verifies, with the given verifier options, that a buffer of";
    code_ += "/// bytes contains a size prefixed `{{STRUCT_TY}}` and returns";
    code_ += "/// it. Note that verification is still experimental and may not";
    code_ += "/// catch every error, or be maximally performant. For the";
    code_ += "/// previous, unchecked, behavior use";
    code_ += "/// `root_as_{{STRUCT_FN}}_unchecked`.";
    code_ +=
        "pub fn size_prefixed_root_as_{{STRUCT_FN}}_with_opts"
        "<'b, 'o>(";
    code_ += "  opts: &'o flatbuffers::VerifierOptions,";
    code_ += "  buf: &'b [u8],";
    code_ +=
        ") -> Result<{{STRUCT_TY}}<'b>, flatbuffers::InvalidFlatbuffer>"
        " {";
    code_ +=
        "  flatbuffers::size_prefixed_root_with_opts::<{{STRUCT_TY}}"
        "<'b>>(opts, buf)";
    code_ += "}";
    // Unchecked root fns.
    code_ += "#[inline]";
    code_ +=
        "/// Assumes, without verification, that a buffer of bytes "
        "contains a {{STRUCT_TY}} and returns it.";
    code_ += "/// # Safety";
    code_ +=
        "/// Callers must trust the given bytes do indeed contain a valid"
        " `{{STRUCT_TY}}`.";
    code_ +=
        "pub unsafe fn root_as_{{STRUCT_FN}}_unchecked"
        "(buf: &[u8]) -> {{STRUCT_TY}} {";
    code_ += "  flatbuffers::root_unchecked::<{{STRUCT_TY}}>(buf)";
    code_ += "}";
    code_ += "#[inline]";
    code_ +=
        "/// Assumes, without verification, that a buffer of bytes "
        "contains a size prefixed {{STRUCT_TY}} and returns it.";
    code_ += "/// # Safety";
    code_ +=
        "/// Callers must trust the given bytes do indeed contain a valid"
        " size prefixed `{{STRUCT_TY}}`.";
    code_ +=
        "pub unsafe fn size_prefixed_root_as_{{STRUCT_FN}}"
        "_unchecked(buf: &[u8]) -> {{STRUCT_TY}} {";
    code_ +=
        "  flatbuffers::size_prefixed_root_unchecked::<{{STRUCT_TY}}>"
        "(buf)";
    code_ += "}";

    if (parser_.file_identifier_.length()) {
      // Declare the identifier
      // (no lifetime needed as constants have static lifetimes by default)
      code_ += "pub const {{STRUCT_CONST}}_IDENTIFIER: &str\\";
      code_ += " = \"" + parser_.file_identifier_ + "\";";
      code_ += "";

      // Check if a buffer has the identifier.
      code_ += "#[inline]";
      code_ += "pub fn {{STRUCT_FN}}_buffer_has_identifier\\";
      code_ += "(buf: &[u8]) -> bool {";
      code_ += "  flatbuffers::buffer_has_identifier(buf, \\";
      code_ += "{{STRUCT_CONST}}_IDENTIFIER, false)";
      code_ += "}";
      code_ += "";
      code_ += "#[inline]";
      code_ += "pub fn {{STRUCT_FN}}_size_prefixed\\";
      code_ += "_buffer_has_identifier(buf: &[u8]) -> bool {";
      code_ += "  flatbuffers::buffer_has_identifier(buf, \\";
      code_ += "{{STRUCT_CONST}}_IDENTIFIER, true)";
      code_ += "}";
      code_ += "";
    }

    if (parser_.file_extension_.length()) {
      // Return the extension
      code_ += "pub const {{STRUCT_CONST}}_EXTENSION: &str = \\";
      code_ += "\"" + parser_.file_extension_ + "\";";
      code_ += "";
    }

    // Finish a buffer with a given root object:
    code_ += "#[inline]";
    code_ +=
        "pub fn finish_{{STRUCT_FN}}_buffer<'a, 'b, A: "
        "flatbuffers::Allocator + 'a>(";
    code_ += "    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,";
    code_ += "    root: flatbuffers::WIPOffset<{{STRUCT_TY}}<'a>>) {";
    if (parser_.file_identifier_.length()) {
      code_ += "  fbb.finish(root, Some({{STRUCT_CONST}}_IDENTIFIER));";
    } else {
      code_ += "  fbb.finish(root, None);";
    }
    code_ += "}";
    code_ += "";
    code_ += "#[inline]";
    code_ +=
        "pub fn finish_size_prefixed_{{STRUCT_FN}}_buffer"
        "<'a, 'b, A: flatbuffers::Allocator + 'a>("
        "fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>, "
        "root: flatbuffers::WIPOffset<{{STRUCT_TY}}<'a>>) {";
    if (parser_.file_identifier_.length()) {
      code_ +=
          "  fbb.finish_size_prefixed(root, "
          "Some({{STRUCT_CONST}}_IDENTIFIER));";
    } else {
      code_ += "  fbb.finish_size_prefixed(root, None);";
    }
    code_ += "}";
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
      assert(!(field.padding & ~0xF));
    }
  }

  static void PaddingDefinition(int bits, std::string *code_ptr, int *id) {
    *code_ptr +=
        "  padding" + NumToString((*id)++) + "__: u" + NumToString(bits) + ",";
  }

  static void PaddingInitializer(int bits, std::string *code_ptr, int *id) {
    (void)bits;
    *code_ptr += "padding" + NumToString((*id)++) + "__: 0,";
  }

  void ForAllStructFields(const StructDef &struct_def,
                          std::function<void(const FieldDef &field)> cb) {
    size_t offset_to_field = 0;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      code_.SetValue("FIELD_TYPE", GetTypeGet(field.value.type));
      code_.SetValue("FIELD_OTY", ObjectFieldType(field, false));
      code_.SetValue("FIELD", namer_.Field(field));
      code_.SetValue("FIELD_OFFSET", NumToString(offset_to_field));
      code_.SetValue(
          "REF",
          IsStruct(field.value.type) || IsArray(field.value.type) ? "&" : "");
      code_.IncrementIdentLevel();
      cb(field);
      code_.DecrementIdentLevel();
      const size_t size = InlineSize(field.value.type);
      offset_to_field += size + field.padding;
    }
  }
  // Generate an accessor struct with constructor for a flatbuffers struct.
  void GenStruct(const StructDef &struct_def) {
    const bool is_private =
        parser_.opts.no_leak_private_annotations &&
        (struct_def.attributes.Lookup("private") != nullptr);
    code_.SetValue("ACCESS_TYPE", is_private ? "pub(crate)" : "pub");
    // Generates manual padding and alignment.
    // Variables are private because they contain little endian data on all
    // platforms.
    GenComment(struct_def.doc_comment);
    code_.SetValue("ALIGN", NumToString(struct_def.minalign));
    code_.SetValue("STRUCT_TY", namer_.Type(struct_def));
    code_.SetValue("STRUCT_SIZE", NumToString(struct_def.bytesize));

    // We represent Flatbuffers-structs in Rust-u8-arrays since the data may be
    // of the wrong endianness and alignment 1.
    //
    // PartialEq is useful to derive because we can correctly compare structs
    // for equality by just comparing their underlying byte data. This doesn't
    // hold for PartialOrd/Ord.
    code_ += "// struct {{STRUCT_TY}}, aligned to {{ALIGN}}";
    code_ += "#[repr(transparent)]";
    code_ += "#[derive(Clone, Copy, PartialEq)]";
    code_ += "{{ACCESS_TYPE}} struct {{STRUCT_TY}}(pub [u8; {{STRUCT_SIZE}}]);";
    code_ += "impl Default for {{STRUCT_TY}} { ";
    code_ += "  fn default() -> Self { ";
    code_ += "    Self([0; {{STRUCT_SIZE}}])";
    code_ += "  }";
    code_ += "}";

    // Debug for structs.
    code_ += "impl core::fmt::Debug for {{STRUCT_TY}} {";
    code_ +=
        "  fn fmt(&self, f: &mut core::fmt::Formatter"
        ") -> core::fmt::Result {";
    code_ += "    f.debug_struct(\"{{STRUCT_TY}}\")";
    ForAllStructFields(struct_def, [&](const FieldDef &unused) {
      (void)unused;
      code_ += "    .field(\"{{FIELD}}\", &self.{{FIELD}}())";
    });
    code_ += "      .finish()";
    code_ += "  }";
    code_ += "}";
    code_ += "";

    // Generate impls for SafeSliceAccess (because all structs are endian-safe),
    // Follow for the value type, Follow for the reference type, Push for the
    // value type, and Push for the reference type.
    code_ += "impl flatbuffers::SimpleToVerifyInSlice for {{STRUCT_TY}} {}";
    code_ += "impl<'a> flatbuffers::Follow<'a> for {{STRUCT_TY}} {";
    code_ += "  type Inner = &'a {{STRUCT_TY}};";
    code_ += "  #[inline]";
    code_ += "  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {";
    code_ += "    <&'a {{STRUCT_TY}}>::follow(buf, loc)";
    code_ += "  }";
    code_ += "}";
    code_ += "impl<'a> flatbuffers::Follow<'a> for &'a {{STRUCT_TY}} {";
    code_ += "  type Inner = &'a {{STRUCT_TY}};";
    code_ += "  #[inline]";
    code_ += "  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {";
    code_ += "    flatbuffers::follow_cast_ref::<{{STRUCT_TY}}>(buf, loc)";
    code_ += "  }";
    code_ += "}";
    code_ += "impl<'b> flatbuffers::Push for {{STRUCT_TY}} {";
    code_ += "    type Output = {{STRUCT_TY}};";
    code_ += "    #[inline]";
    code_ += "    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {";
    code_ +=
        "        let src = ::core::slice::from_raw_parts(self as *const "
        "{{STRUCT_TY}} as *const u8, Self::size());";
    code_ += "        dst.copy_from_slice(src);";
    code_ += "    }";
    code_ += "}";
    code_ += "";

    // Generate verifier: Structs are simple so presence and alignment are
    // all that need to be checked.
    code_ += "impl<'a> flatbuffers::Verifiable for {{STRUCT_TY}} {";
    code_ += "  #[inline]";
    code_ += "  fn run_verifier(";
    code_ += "    v: &mut flatbuffers::Verifier, pos: usize";
    code_ += "  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {";
    code_ += "    use self::flatbuffers::Verifiable;";
    code_ += "    v.in_buffer::<Self>(pos)";
    code_ += "  }";
    code_ += "}";
    code_ += "";

    // Implement serde::Serialize
    if (parser_.opts.rust_serialize) {
      const auto numFields = struct_def.fields.vec.size();
      code_.SetValue("NUM_FIELDS", NumToString(numFields));
      code_ += "impl Serialize for {{STRUCT_TY}} {";
      code_ +=
          "  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>";
      code_ += "  where";
      code_ += "    S: Serializer,";
      code_ += "  {";
      if (numFields == 0) {
        code_ +=
            "    let s = serializer.serialize_struct(\"{{STRUCT_TY}}\", 0)?;";
      } else {
        code_ +=
            "    let mut s = serializer.serialize_struct(\"{{STRUCT_TY}}\", "
            "{{NUM_FIELDS}})?;";
      }
      ForAllStructFields(struct_def, [&](const FieldDef &unused) {
        (void)unused;
        code_ +=
            "    s.serialize_field(\"{{FIELD}}\", "
            "&self.{{FIELD}}())?;";
      });
      code_ += "    s.end()";
      code_ += "  }";
      code_ += "}";
      code_ += "";
    }

    // Generate a constructor that takes all fields as arguments.
    code_ += "impl<'a> {{STRUCT_TY}} {";
    code_ += "  #[allow(clippy::too_many_arguments)]";
    code_ += "  pub fn new(";
    ForAllStructFields(struct_def, [&](const FieldDef &unused) {
      (void)unused;
      code_ += "  {{FIELD}}: {{REF}}{{FIELD_TYPE}},";
    });
    code_ += "  ) -> Self {";
    code_ += "    let mut s = Self([0; {{STRUCT_SIZE}}]);";
    ForAllStructFields(struct_def, [&](const FieldDef &unused) {
      (void)unused;
      code_ += "  s.set_{{FIELD}}({{FIELD}});";
    });
    code_ += "    s";
    code_ += "  }";
    code_ += "";

    if (parser_.opts.generate_name_strings) {
      GenFullyQualifiedNameGetter(struct_def, struct_def.name);
    }

    // Generate accessor methods for the struct.
    ForAllStructFields(struct_def, [&](const FieldDef &field) {
      this->GenComment(field.doc_comment);
      // Getter.
      if (IsStruct(field.value.type)) {
        code_ += "pub fn {{FIELD}}(&self) -> &{{FIELD_TYPE}} {";
        code_ += "  // Safety:";
        code_ += "  // Created from a valid Table for this object";
        code_ += "  // Which contains a valid struct in this slot";
        code_ +=
            "  unsafe {"
            " &*(self.0[{{FIELD_OFFSET}}..].as_ptr() as *const"
            " {{FIELD_TYPE}}) }";
      } else if (IsArray(field.value.type)) {
        code_.SetValue("ARRAY_SIZE",
                       NumToString(field.value.type.fixed_length));
        code_.SetValue("ARRAY_ITEM", GetTypeGet(field.value.type.VectorType()));
        code_ +=
            "pub fn {{FIELD}}(&'a self) -> "
            "flatbuffers::Array<'a, {{ARRAY_ITEM}}, {{ARRAY_SIZE}}> {";
        code_ += "  // Safety:";
        code_ += "  // Created from a valid Table for this object";
        code_ += "  // Which contains a valid array in this slot";
        code_ +=
            "  unsafe { flatbuffers::Array::follow(&self.0, {{FIELD_OFFSET}}) "
            "}";
      } else {
        code_ += "pub fn {{FIELD}}(&self) -> {{FIELD_TYPE}} {";
        code_ +=
            "  let mut mem = core::mem::MaybeUninit::"
            "<<{{FIELD_TYPE}} as EndianScalar>::Scalar>::uninit();";
        code_ += "  // Safety:";
        code_ += "  // Created from a valid Table for this object";
        code_ += "  // Which contains a valid value in this slot";
        code_ += "  EndianScalar::from_little_endian(unsafe {";
        code_ += "    core::ptr::copy_nonoverlapping(";
        code_ += "      self.0[{{FIELD_OFFSET}}..].as_ptr(),";
        code_ += "      mem.as_mut_ptr() as *mut u8,";
        code_ +=
            "      core::mem::size_of::<<{{FIELD_TYPE}} as "
            "EndianScalar>::Scalar>(),";
        code_ += "    );";
        code_ += "    mem.assume_init()";
        code_ += "  })";
      }
      code_ += "}\n";
      // Setter.
      if (IsStruct(field.value.type)) {
        code_.SetValue("FIELD_SIZE", NumToString(InlineSize(field.value.type)));
        code_ += "#[allow(clippy::identity_op)]";  // If FIELD_OFFSET=0.
        code_ += "pub fn set_{{FIELD}}(&mut self, x: &{{FIELD_TYPE}}) {";
        code_ +=
            "  self.0[{{FIELD_OFFSET}}..{{FIELD_OFFSET}} + {{FIELD_SIZE}}]"
            ".copy_from_slice(&x.0)";
      } else if (IsArray(field.value.type)) {
        if (GetFullType(field.value.type) == ftArrayOfBuiltin) {
          code_.SetValue("ARRAY_ITEM",
                         GetTypeGet(field.value.type.VectorType()));
          code_.SetValue(
              "ARRAY_ITEM_SIZE",
              NumToString(InlineSize(field.value.type.VectorType())));
          code_ +=
              "pub fn set_{{FIELD}}(&mut self, items: &{{FIELD_TYPE}}) "
              "{";
          code_ += "  // Safety:";
          code_ += "  // Created from a valid Table for this object";
          code_ += "  // Which contains a valid array in this slot";
          code_ +=
              "  unsafe { flatbuffers::emplace_scalar_array(&mut self.0, "
              "{{FIELD_OFFSET}}, items) };";
        } else {
          code_.SetValue("FIELD_SIZE",
                         NumToString(InlineSize(field.value.type)));
          code_ += "pub fn set_{{FIELD}}(&mut self, x: &{{FIELD_TYPE}}) {";
          code_ += "  // Safety:";
          code_ += "  // Created from a valid Table for this object";
          code_ += "  // Which contains a valid array in this slot";
          code_ += "  unsafe {";
          code_ += "    core::ptr::copy(";
          code_ += "      x.as_ptr() as *const u8,";
          code_ += "      self.0.as_mut_ptr().add({{FIELD_OFFSET}}),";
          code_ += "      {{FIELD_SIZE}},";
          code_ += "    );";
          code_ += "  }";
        }
      } else {
        code_ += "pub fn set_{{FIELD}}(&mut self, x: {{FIELD_TYPE}}) {";
        code_ += "  let x_le = x.to_little_endian();";
        code_ += "  // Safety:";
        code_ += "  // Created from a valid Table for this object";
        code_ += "  // Which contains a valid value in this slot";
        code_ += "  unsafe {";
        code_ += "    core::ptr::copy_nonoverlapping(";
        code_ += "      &x_le as *const _ as *const u8,";
        code_ += "      self.0[{{FIELD_OFFSET}}..].as_mut_ptr(),";
        code_ +=
            "      core::mem::size_of::<<{{FIELD_TYPE}} as "
            "EndianScalar>::Scalar>(),";
        code_ += "    );";
        code_ += "  }";
      }
      code_ += "}\n";

      // Generate a comparison function for this field if it is a key.
      if (field.key) { GenKeyFieldMethods(field); }
    });

    // Generate Object API unpack method.
    if (parser_.opts.generate_object_based_api) {
      code_.SetValue("STRUCT_OTY", namer_.ObjectType(struct_def));
      code_ += "  pub fn unpack(&self) -> {{STRUCT_OTY}} {";
      code_ += "    {{STRUCT_OTY}} {";
      ForAllStructFields(struct_def, [&](const FieldDef &field) {
        if (IsArray(field.value.type)) {
          if (GetFullType(field.value.type) == ftArrayOfStruct) {
            code_ +=
                "    {{FIELD}}: { let {{FIELD}} = "
                "self.{{FIELD}}(); flatbuffers::array_init(|i| "
                "{{FIELD}}.get(i).unpack()) },";
          } else {
            code_ += "    {{FIELD}}: self.{{FIELD}}().into(),";
          }
        } else {
          std::string unpack = IsStruct(field.value.type) ? ".unpack()" : "";
          code_ += "    {{FIELD}}: self.{{FIELD}}()" + unpack + ",";
        }
      });
      code_ += "    }";
      code_ += "  }";
    }

    code_ += "}";  // End impl Struct methods.
    code_ += "";

    // Generate Struct Object.
    if (parser_.opts.generate_object_based_api) {
      // Struct declaration
      code_ += "#[derive(Debug, Clone, PartialEq, Default)]";
      code_ += "{{ACCESS_TYPE}} struct {{STRUCT_OTY}} {";
      ForAllStructFields(struct_def, [&](const FieldDef &field) {
        (void)field;  // unused.
        code_ += "pub {{FIELD}}: {{FIELD_OTY}},";
      });
      code_ += "}";
      // The `pack` method that turns the native struct into its Flatbuffers
      // counterpart.
      code_ += "impl {{STRUCT_OTY}} {";
      code_ += "  pub fn pack(&self) -> {{STRUCT_TY}} {";
      code_ += "    {{STRUCT_TY}}::new(";
      ForAllStructFields(struct_def, [&](const FieldDef &field) {
        if (IsStruct(field.value.type)) {
          code_ += "    &self.{{FIELD}}.pack(),";
        } else if (IsArray(field.value.type)) {
          if (GetFullType(field.value.type) == ftArrayOfStruct) {
            code_ +=
                "    &flatbuffers::array_init(|i| "
                "self.{{FIELD}}[i].pack()),";
          } else {
            code_ += "    &self.{{FIELD}},";
          }
        } else {
          code_ += "    self.{{FIELD}},";
        }
      });
      code_ += "    )";
      code_ += "  }";
      code_ += "}";
      code_ += "";
    }
  }

  void GenNamespaceImports(const int white_spaces) {
    // DO not use global attributes (i.e. #![...]) since it interferes
    // with users who include! generated files.
    // See: https://github.com/google/flatbuffers/issues/6261
    std::string indent = std::string(white_spaces, ' ');
    code_ += "";
    if (!parser_.opts.generate_all) {
      for (auto it = parser_.included_files_.begin();
           it != parser_.included_files_.end(); ++it) {
        if (it->second.empty()) continue;
        auto noext = flatbuffers::StripExtension(it->second);
        auto basename = flatbuffers::StripPath(noext);

        if (parser_.opts.include_prefix.empty()) {
          code_ += indent + "use crate::" + basename +
                   parser_.opts.filename_suffix + "::*;";
        } else {
          auto prefix = parser_.opts.include_prefix;
          prefix.pop_back();

          code_ += indent + "use crate::" + prefix + "::" + basename +
                   parser_.opts.filename_suffix + "::*;";
        }
      }
    }
    code_ += indent + "use core::mem;";
    code_ += indent + "use core::cmp::Ordering;";
    code_ += "";
    if (parser_.opts.rust_serialize) {
      code_ += indent + "extern crate serde;";
      code_ +=
          indent +
          "use self::serde::ser::{Serialize, Serializer, SerializeStruct};";
      code_ += "";
    }
    code_ += indent + "extern crate flatbuffers;";
    code_ += indent + "use self::flatbuffers::{EndianScalar, Follow};";
  }

  // Set up the correct namespace. This opens a namespace if the current
  // namespace is different from the target namespace. This function
  // closes and opens the namespaces only as necessary.
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
      code_ += "}  // pub mod " + cur_name_space_->components[j - 1];
    }
    if (old_size != common_prefix_size) { code_ += ""; }

    // open namespace parts to reach the ns namespace
    // in the previous example, E, then F, then G are opened
    for (auto j = common_prefix_size; j != new_size; ++j) {
      code_ += "#[allow(unused_imports, dead_code)]";
      code_ += "pub mod " + namer_.Namespace(ns->components[j]) + " {";
      // Generate local namespace imports.
      GenNamespaceImports(2);
    }
    if (new_size != common_prefix_size) { code_ += ""; }

    cur_name_space_ = ns;
  }

 private:
  IdlNamer namer_;
};

}  // namespace rust

static bool GenerateRust(const Parser &parser, const std::string &path,
                         const std::string &file_name) {
  rust::RustGenerator generator(parser, path, file_name);
  return generator.generate();
}

static std::string RustMakeRule(const Parser &parser, const std::string &path,
                                const std::string &file_name) {
  std::string filebase =
      flatbuffers::StripPath(flatbuffers::StripExtension(file_name));
  rust::RustGenerator generator(parser, path, file_name);
  std::string make_rule =
      generator.GeneratedFileName(path, filebase, parser.opts) + ": ";

  auto included_files = parser.GetIncludedFilesRecursive(file_name);
  for (auto it = included_files.begin(); it != included_files.end(); ++it) {
    make_rule += " " + *it;
  }
  return make_rule;
}

namespace {

class RustCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateRust(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = RustMakeRule(parser, path, filename);
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
    if (!GenerateRustModuleRootFile(parser, path)) { return Status::ERROR; }
    return Status::OK;
  }

  bool IsSchemaOnly() const override { return true; }

  bool SupportsBfbsGeneration() const override { return false; }

  bool SupportsRootFileGeneration() const override { return true; }

  IDLOptions::Language Language() const override { return IDLOptions::kRust; }

  std::string LanguageName() const override { return "Rust"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewRustCodeGenerator() {
  return std::unique_ptr<RustCodeGenerator>(new RustCodeGenerator());
}

}  // namespace flatbuffers

// TODO(rw): Generated code should import other generated files.
// TODO(rw): Generated code should refer to namespaces in included files in a
//           way that makes them referrable.
// TODO(rw): Generated code should indent according to nesting level.
// TODO(rw): Generated code should generate endian-safe Debug impls.
// TODO(rw): Generated code could use a Rust-only enum type to access unions,
//           instead of making the user use _type() to manually switch.
// TODO(maxburke): There should be test schemas added that use language
//           keywords as fields of structs, tables, unions, enums, to make sure
//           that internal code generated references escaped names correctly.
// TODO(maxburke): We should see if there is a more flexible way of resolving
//           module paths for use declarations. Right now if schemas refer to
//           other flatbuffer files, the include paths in emitted Rust bindings
//           are crate-relative which may undesirable.
