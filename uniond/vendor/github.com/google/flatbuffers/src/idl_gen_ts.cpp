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

#include "idl_gen_ts.h"

#include <algorithm>
#include <cassert>
#include <cmath>
#include <iostream>
#include <unordered_map>
#include <unordered_set>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/flatc.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

namespace flatbuffers {
namespace {
struct ImportDefinition {
  std::string name;
  std::string import_statement;
  std::string export_statement;
  std::string bare_file_path;
  std::string rel_file_path;
  std::string object_name;
  const Definition *dependent = nullptr;
  const Definition *dependency = nullptr;
};

struct NsDefinition {
  std::string path;
  std::string filepath;
  std::string symbolic_name;
  const Namespace *ns;
  std::map<std::string, const Definition *> definitions;
};

Namer::Config TypeScriptDefaultConfig() {
  return { /*types=*/Case::kKeep,
           /*constants=*/Case::kUnknown,
           /*methods=*/Case::kLowerCamel,
           /*functions=*/Case::kLowerCamel,
           /*fields=*/Case::kLowerCamel,
           /*variables=*/Case::kLowerCamel,
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/"::",
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kKeep,
           /*namespace_seperator=*/"_",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kDasher,
           /*directories=*/Case::kDasher,
           /*output_path=*/"",
           /*filename_suffix=*/"_generated",
           /*filename_extension=*/".ts" };
}

std::set<std::string> TypescriptKeywords() {
  // List of keywords retrieved from here:
  // https://github.com/microsoft/TypeScript/issues/2536
  return {
    "arguments", "break",    "case",      "catch",      "class",      "const",
    "continue",  "debugger", "default",   "delete",     "do",         "else",
    "enum",      "export",   "extends",   "false",      "finally",    "for",
    "function",  "if",       "import",    "in",         "instanceof", "new",
    "null",      "Object",   "return",    "super",      "switch",     "this",
    "throw",     "true",     "try",       "typeof",     "var",        "void",
    "while",     "with",     "as",        "implements", "interface",  "let",
    "package",   "private",  "protected", "public",     "static",     "yield",
  };
}

enum AnnotationType { kParam = 0, kType = 1, kReturns = 2 };

template<typename T> struct SupportsObjectAPI : std::false_type {};

template<> struct SupportsObjectAPI<StructDef> : std::true_type {};

}  // namespace

namespace ts {
// Iterate through all definitions we haven't generate code for (enums, structs,
// and tables) and output them to a single file.
class TsGenerator : public BaseGenerator {
 public:
  typedef std::map<std::string, ImportDefinition> import_set;

  TsGenerator(const Parser &parser, const std::string &path,
              const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "", "_", "ts"),
        namer_(WithFlagOptions(TypeScriptDefaultConfig(), parser.opts, path),
               TypescriptKeywords()) {}

  bool generate() {
    generateEnums();
    generateStructs();
    if (!parser_.opts.ts_omit_entrypoint) { generateEntry(); }
    if (!generateBundle()) return false;
    return true;
  }

  std::string GetTypeName(const EnumDef &def, const bool = false,
                          const bool force_ns_wrap = false) {
    if (force_ns_wrap) { return namer_.NamespacedType(def); }
    return namer_.Type(def);
  }

  std::string GetTypeName(const StructDef &def, const bool object_api = false,
                          const bool force_ns_wrap = false) {
    if (object_api && parser_.opts.generate_object_based_api) {
      if (force_ns_wrap) {
        return namer_.NamespacedObjectType(def);
      } else {
        return namer_.ObjectType(def);
      }
    } else {
      if (force_ns_wrap) {
        return namer_.NamespacedType(def);
      } else {
        return namer_.Type(def);
      }
    }
  }

  // Save out the generated code for a single class while adding
  // declaration boilerplate.
  bool SaveType(const Definition &definition, const std::string &class_code,
                import_set &imports, import_set &bare_imports) {
    if (!class_code.length()) return true;

    std::string code;

    code += "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n" +
        "/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */\n\n";

    for (auto it = bare_imports.begin(); it != bare_imports.end(); it++) {
      code += it->second.import_statement + "\n";
    }
    if (!bare_imports.empty()) code += "\n";

    for (auto it = imports.begin(); it != imports.end(); it++) {
      if (it->second.dependency != &definition) {
        code += it->second.import_statement + "\n";
      }
    }
    if (!imports.empty()) code += "\n\n";

    code += class_code;

    auto dirs = namer_.Directories(*definition.defined_namespace);
    EnsureDirExists(dirs);
    auto basename = dirs + namer_.File(definition, SkipFile::Suffix);

    return SaveFile(basename.c_str(), code, false);
  }

  void TrackNsDef(const Definition &definition, std::string type_name) {
    std::string path;
    std::string filepath;
    std::string symbolic_name;
    if (definition.defined_namespace->components.size() > 0) {
      path = namer_.Directories(*definition.defined_namespace,
                                SkipDir::TrailingPathSeperator);
      filepath = path + ".ts";
      path = namer_.Directories(*definition.defined_namespace,
                                SkipDir::OutputPathAndTrailingPathSeparator);
      symbolic_name = definition.defined_namespace->components.back();
    } else {
      auto def_mod_name = namer_.File(definition, SkipFile::SuffixAndExtension);
      symbolic_name = file_name_;
      filepath = path_ + symbolic_name + ".ts";
    }
    if (ns_defs_.count(path) == 0) {
      NsDefinition nsDef;
      nsDef.path = path;
      nsDef.filepath = filepath;
      nsDef.ns = definition.defined_namespace;
      nsDef.definitions.insert(std::make_pair(type_name, &definition));
      nsDef.symbolic_name = symbolic_name;
      ns_defs_[path] = nsDef;
    } else {
      ns_defs_[path].definitions.insert(std::make_pair(type_name, &definition));
    }
  }

 private:
  IdlNamer namer_;

  std::map<std::string, NsDefinition> ns_defs_;

  // Generate code for all enums.
  void generateEnums() {
    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      import_set bare_imports;
      import_set imports;
      std::string enumcode;
      auto &enum_def = **it;
      GenEnum(enum_def, &enumcode, imports, false);
      GenEnum(enum_def, &enumcode, imports, true);
      std::string type_name = GetTypeName(enum_def);
      TrackNsDef(enum_def, type_name);
      SaveType(enum_def, enumcode, imports, bare_imports);
    }
  }

  // Generate code for all structs.
  void generateStructs() {
    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      import_set bare_imports;
      import_set imports;
      AddImport(bare_imports, "* as flatbuffers", "flatbuffers");
      auto &struct_def = **it;
      std::string declcode;
      GenStruct(parser_, struct_def, &declcode, imports);
      std::string type_name = GetTypeName(struct_def);
      TrackNsDef(struct_def, type_name);
      SaveType(struct_def, declcode, imports, bare_imports);
    }
  }

  // Generate code for a single entry point module.
  void generateEntry() {
    std::string code;

    // add root namespace def if not already existing from defs tracking
    std::string root;
    if (ns_defs_.count(root) == 0) {
      NsDefinition nsDef;
      nsDef.path = root;
      nsDef.symbolic_name = file_name_;
      nsDef.filepath = path_ + file_name_ + ".ts";
      nsDef.ns = new Namespace();
      ns_defs_[nsDef.path] = nsDef;
    }

    for (const auto &it : ns_defs_) {
      code = "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n" +
        "/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */\n\n";
      
      // export all definitions in ns entry point module
      int export_counter = 0;
      for (const auto &def : it.second.definitions) {
        std::vector<std::string> rel_components;
        // build path for root level vs child level
        if (it.second.ns->components.size() > 1)
          std::copy(it.second.ns->components.begin() + 1,
                    it.second.ns->components.end(),
                    std::back_inserter(rel_components));
        else
          std::copy(it.second.ns->components.begin(),
                    it.second.ns->components.end(),
                    std::back_inserter(rel_components));
        auto base_file_name =
            namer_.File(*(def.second), SkipFile::SuffixAndExtension);
        auto base_name =
            namer_.Directories(it.second.ns->components, SkipDir::OutputPath) +
            base_file_name;
        auto ts_file_path = base_name + ".ts";
        auto base_name_rel = std::string("./");
        base_name_rel +=
            namer_.Directories(rel_components, SkipDir::OutputPath);
        base_name_rel += base_file_name;
        auto ts_file_path_rel = base_name_rel + ".ts";
        auto type_name = def.first;
        auto fully_qualified_type_name =
            it.second.ns->GetFullyQualifiedName(type_name);
        auto is_struct = parser_.structs_.Lookup(fully_qualified_type_name);
        code += "export { " + type_name;
        if (parser_.opts.generate_object_based_api && is_struct) {
          code += ", " + type_name + parser_.opts.object_suffix;
        }
        code += " } from '";
        std::string import_extension =
            parser_.opts.ts_no_import_ext ? "" : ".js";
        code += base_name_rel + import_extension + "';\n";
        export_counter++;
      }

      // re-export child namespace(s) in parent
      const auto child_ns_level = it.second.ns->components.size() + 1;
      for (const auto &it2 : ns_defs_) {
        if (it2.second.ns->components.size() != child_ns_level) continue;
        auto ts_file_path = it2.second.path + ".ts";
        code += "export * as " + it2.second.symbolic_name + " from './";
        std::string rel_path = it2.second.path;
        code += rel_path + ".js';\n";
        export_counter++;
      }

      if (export_counter > 0) SaveFile(it.second.filepath.c_str(), code, false);
    }
  }

  bool generateBundle() {
    if (parser_.opts.ts_flat_files) {
      std::string inputpath;
      std::string symbolic_name = file_name_;
      inputpath = path_ + file_name_ + ".ts";
      std::string bundlepath =
          GeneratedFileName(path_, file_name_, parser_.opts);
      bundlepath = bundlepath.substr(0, bundlepath.size() - 3) + ".js";
      std::string cmd = "esbuild";
      cmd += " ";
      cmd += inputpath;
      // cmd += " --minify";
      cmd += " --format=cjs --bundle --outfile=";
      cmd += bundlepath;
      cmd += " --external:flatbuffers";
      std::cout << "Entry point " << inputpath << " generated." << std::endl;
      std::cout << "A single file bundle can be created using fx. esbuild with:"
                << std::endl;
      std::cout << "> " << cmd << std::endl;
    }
    return true;
  }

  // Generate a documentation comment, if available.
  static void GenDocComment(const std::vector<std::string> &dc,
                            std::string *code_ptr,
                            const char *indent = nullptr) {
    if (dc.empty()) {
      // Don't output empty comment blocks with 0 lines of comment content.
      return;
    }

    std::string &code = *code_ptr;
    if (indent) code += indent;
    code += "/**\n";
    for (auto it = dc.begin(); it != dc.end(); ++it) {
      if (indent) code += indent;
      code += " *" + *it + "\n";
    }
    if (indent) code += indent;
    code += " */\n";
  }

  static void GenDocComment(std::string *code_ptr) {
    GenDocComment(std::vector<std::string>(), code_ptr);
  }

  // Generate an enum declaration and an enum string lookup table.
  void GenEnum(EnumDef &enum_def, std::string *code_ptr, import_set &imports,
               bool reverse) {
    if (enum_def.generated) return;
    if (reverse) return;  // FIXME.
    std::string &code = *code_ptr;
    GenDocComment(enum_def.doc_comment, code_ptr);
    code += "export enum ";
    code += GetTypeName(enum_def);
    code += " {\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      if (!ev.doc_comment.empty()) {
        if (it != enum_def.Vals().begin()) { code += '\n'; }
        GenDocComment(ev.doc_comment, code_ptr, "  ");
      }

      // Generate mapping between EnumName: EnumValue(int)
      if (reverse) {
        code += "  '" + enum_def.ToString(ev) + "'";
        code += " = ";
        code += "'" + namer_.Variant(ev) + "'";
      } else {
        code += "  " + namer_.Variant(ev);
        code += " = ";
        // Unfortunately, because typescript does not support bigint enums,
        // for 64-bit enums, we instead map the enum names to strings.
        switch (enum_def.underlying_type.base_type) {
          case BASE_TYPE_LONG:
          case BASE_TYPE_ULONG: {
            code += "'" + enum_def.ToString(ev) + "'";
            break;
          }
          default: code += enum_def.ToString(ev);
        }
      }

      code += (it + 1) != enum_def.Vals().end() ? ",\n" : "\n";
    }
    code += "}";

    if (enum_def.is_union) {
      code += GenUnionConvFunc(enum_def.underlying_type, imports);
    }

    code += "\n";
  }

  static std::string GenType(const Type &type) {
    switch (type.base_type) {
      case BASE_TYPE_BOOL:
      case BASE_TYPE_CHAR: return "Int8";
      case BASE_TYPE_UTYPE: return GenType(GetUnionUnderlyingType(type));
      case BASE_TYPE_UCHAR: return "Uint8";
      case BASE_TYPE_SHORT: return "Int16";
      case BASE_TYPE_USHORT: return "Uint16";
      case BASE_TYPE_INT: return "Int32";
      case BASE_TYPE_UINT: return "Uint32";
      case BASE_TYPE_LONG: return "Int64";
      case BASE_TYPE_ULONG: return "Uint64";
      case BASE_TYPE_FLOAT: return "Float32";
      case BASE_TYPE_DOUBLE: return "Float64";
      case BASE_TYPE_STRING: return "String";
      case BASE_TYPE_VECTOR: return GenType(type.VectorType());
      case BASE_TYPE_STRUCT: return type.struct_def->name;
      default: return "flatbuffers.Table";
    }
  }

  std::string GenGetter(const Type &type, const std::string &arguments) {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return GenBBAccess() + ".__string" + arguments;
      case BASE_TYPE_STRUCT: return GenBBAccess() + ".__struct" + arguments;
      case BASE_TYPE_UNION:
        if (!UnionHasStringType(*type.enum_def)) {
          return GenBBAccess() + ".__union" + arguments;
        }
        return GenBBAccess() + ".__union_with_string" + arguments;
      case BASE_TYPE_VECTOR: return GenGetter(type.VectorType(), arguments);
      default: {
        auto getter = GenBBAccess() + "." + "read" + GenType(type) + arguments;
        if (type.base_type == BASE_TYPE_BOOL) { getter = "!!" + getter; }
        return getter;
      }
    }
  }

  std::string GenBBAccess() const { return "this.bb!"; }

  std::string GenDefaultValue(const FieldDef &field, import_set &imports) {
    if (field.IsScalarOptional()) { return "null"; }

    const auto &value = field.value;
    if (value.type.enum_def && value.type.base_type != BASE_TYPE_UNION &&
        value.type.base_type != BASE_TYPE_VECTOR) {
      switch (value.type.base_type) {
        case BASE_TYPE_ARRAY: {
          std::string ret = "[";
          for (auto i = 0; i < value.type.fixed_length; ++i) {
            std::string enum_name =
                AddImport(imports, *value.type.enum_def, *value.type.enum_def)
                    .name;
            std::string enum_value = namer_.Variant(
                *value.type.enum_def->FindByValue(value.constant));
            ret += enum_name + "." + enum_value +
                   (i < value.type.fixed_length - 1 ? ", " : "");
          }
          ret += "]";
          return ret;
        }
        case BASE_TYPE_LONG:
        case BASE_TYPE_ULONG: {
          // If the value is an enum with a 64-bit base type, we have to just
          // return the bigint value directly since typescript does not support
          // enums with bigint backing types.
          return "BigInt('" + value.constant + "')";
        }
        default: {
          EnumVal *val = value.type.enum_def->FindByValue(value.constant);
          if (val == nullptr)
            val = const_cast<EnumVal *>(value.type.enum_def->MinValue());
          return AddImport(imports, *value.type.enum_def, *value.type.enum_def)
                     .name +
                 "." + namer_.Variant(*val);
        }
      }
    }

    switch (value.type.base_type) {
      case BASE_TYPE_BOOL: return value.constant == "0" ? "false" : "true";

      case BASE_TYPE_STRING:
      case BASE_TYPE_UNION:
      case BASE_TYPE_STRUCT: {
        return "null";
      }

      case BASE_TYPE_ARRAY:
      case BASE_TYPE_VECTOR: return "[]";

      case BASE_TYPE_LONG:
      case BASE_TYPE_ULONG: {
        return "BigInt('" + value.constant + "')";
      }

      default: {
        if (StringIsFlatbufferNan(value.constant)) {
          return "NaN";
        } else if (StringIsFlatbufferPositiveInfinity(value.constant)) {
          return "Infinity";
        } else if (StringIsFlatbufferNegativeInfinity(value.constant)) {
          return "-Infinity";
        }
        return value.constant;
      }
    }
  }

  std::string GenTypeName(import_set &imports, const Definition &owner,
                          const Type &type, bool input,
                          bool allowNull = false) {
    if (!input) {
      if (IsString(type) || type.base_type == BASE_TYPE_STRUCT) {
        std::string name;
        if (IsString(type)) {
          name = "string|Uint8Array";
        } else {
          name = AddImport(imports, owner, *type.struct_def).name;
        }
        return allowNull ? (name + "|null") : name;
      }
    }

    switch (type.base_type) {
      case BASE_TYPE_BOOL: return allowNull ? "boolean|null" : "boolean";
      case BASE_TYPE_LONG:
      case BASE_TYPE_ULONG: return allowNull ? "bigint|null" : "bigint";
      case BASE_TYPE_ARRAY: {
        std::string name;
        if (type.element == BASE_TYPE_LONG || type.element == BASE_TYPE_ULONG) {
          name = "bigint[]";
        } else if (type.element != BASE_TYPE_STRUCT) {
          name = "number[]";
        } else {
          name = "any[]";
          if (parser_.opts.generate_object_based_api) {
            name = "(any|" +
                   GetTypeName(*type.struct_def, /*object_api =*/true) + ")[]";
          }
        }

        return name + (allowNull ? "|null" : "");
      }
      default:
        if (IsScalar(type.base_type)) {
          if (type.enum_def) {
            const auto enum_name =
                AddImport(imports, owner, *type.enum_def).name;
            return allowNull ? (enum_name + "|null") : enum_name;
          }
          return allowNull ? "number|null" : "number";
        }
        return "flatbuffers.Offset";
    }
  }

  static Type GetUnionUnderlyingType(const Type &type)
  {
    if (type.enum_def != nullptr && 
        type.enum_def->underlying_type.base_type != type.base_type) {
      return type.enum_def->underlying_type;
    } else {
        return Type(BASE_TYPE_UCHAR);
    }
  }

  static Type GetUnderlyingVectorType(const Type &vector_type)
  {
    return (vector_type.base_type == BASE_TYPE_UTYPE) ? GetUnionUnderlyingType(vector_type) : vector_type;
  }

  // Returns the method name for use with add/put calls.
  std::string GenWriteMethod(const Type &type) {
    // Forward to signed versions since unsigned versions don't exist
    switch (type.base_type) {
      case BASE_TYPE_UTYPE: return GenWriteMethod(GetUnionUnderlyingType(type));
      case BASE_TYPE_UCHAR: return GenWriteMethod(Type(BASE_TYPE_CHAR));
      case BASE_TYPE_USHORT: return GenWriteMethod(Type(BASE_TYPE_SHORT));
      case BASE_TYPE_UINT: return GenWriteMethod(Type(BASE_TYPE_INT));
      case BASE_TYPE_ULONG: return GenWriteMethod(Type(BASE_TYPE_LONG));
      default: break;
    }

    return IsScalar(type.base_type) ? namer_.Type(GenType(type))
                                    : (IsStruct(type) ? "Struct" : "Offset");
  }

  template<typename T> static std::string MaybeAdd(T value) {
    return value != 0 ? " + " + NumToString(value) : "";
  }

  template<typename T> static std::string MaybeScale(T value) {
    return value != 1 ? " * " + NumToString(value) : "";
  }

  void GenStructArgs(import_set &imports, const StructDef &struct_def,
                     std::string *arguments, const std::string &nameprefix) {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (IsStruct(field.value.type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        GenStructArgs(imports, *field.value.type.struct_def, arguments,
                      nameprefix + field.name + "_");
      } else {
        *arguments += ", " + nameprefix + field.name + ": " +
                      GenTypeName(imports, field, field.value.type, true,
                                  field.IsOptional());
      }
    }
  }

  void GenStructBody(const StructDef &struct_def, std::string *body,
                     const std::string &nameprefix) {
    *body += "  builder.prep(";
    *body += NumToString(struct_def.minalign) + ", ";
    *body += NumToString(struct_def.bytesize) + ");\n";

    for (auto it = struct_def.fields.vec.rbegin();
         it != struct_def.fields.vec.rend(); ++it) {
      auto &field = **it;
      if (field.padding) {
        *body += "  builder.pad(" + NumToString(field.padding) + ");\n";
      }
      if (IsStruct(field.value.type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        GenStructBody(
            *field.value.type.struct_def, body,
            nameprefix.length() ? nameprefix + "_" + field.name : field.name);
      } else {
        auto element_type = field.value.type.element;

        if (field.value.type.base_type == BASE_TYPE_ARRAY) {
          switch (field.value.type.element) {
            case BASE_TYPE_STRUCT: {
              std::string str_last_item_idx =
                  NumToString(field.value.type.fixed_length - 1);
              *body += "\n  for (let i = " + str_last_item_idx +
                       "; i >= 0; --i" + ") {\n";

              std::string fname = nameprefix.length()
                                      ? nameprefix + "_" + field.name
                                      : field.name;

              *body += "    const item = " + fname + "?.[i];\n\n";

              if (parser_.opts.generate_object_based_api) {
                *body += "    if (item instanceof " +
                         GetTypeName(*field.value.type.struct_def,
                                     /*object_api =*/true) +
                         ") {\n";
                *body += "      item.pack(builder);\n";
                *body += "      continue;\n";
                *body += "    }\n\n";
              }

              std::string class_name =
                  GetPrefixedName(*field.value.type.struct_def);
              std::string pack_func_create_call =
                  class_name + ".create" + class_name + "(builder,\n";
              pack_func_create_call +=
                  "    " +
                  GenStructMemberValueTS(*field.value.type.struct_def, "item",
                                         ",\n    ", false) +
                  "\n  ";
              *body += "    " + pack_func_create_call;
              *body += "  );\n  }\n\n";

              break;
            }
            default: {
              std::string str_last_item_idx =
                  NumToString(field.value.type.fixed_length - 1);
              std::string fname = nameprefix.length()
                                      ? nameprefix + "_" + field.name
                                      : field.name;

              *body += "\n  for (let i = " + str_last_item_idx +
                       "; i >= 0; --i) {\n";
              *body += "    builder.write";
              *body += GenWriteMethod(
                  static_cast<flatbuffers::Type>(field.value.type.element));
              *body += "(";
              *body += element_type == BASE_TYPE_BOOL ? "+" : "";

              if (element_type == BASE_TYPE_LONG ||
                  element_type == BASE_TYPE_ULONG) {
                *body += "BigInt(" + fname + "?.[i] ?? 0));\n";
              } else {
                *body += "(" + fname + "?.[i] ?? 0));\n\n";
              }
              *body += "  }\n\n";
              break;
            }
          }
        } else {
          std::string fname =
              nameprefix.length() ? nameprefix + "_" + field.name : field.name;

          *body += "  builder.write" + GenWriteMethod(field.value.type) + "(";
          if (field.value.type.base_type == BASE_TYPE_BOOL) {
            *body += "Number(Boolean(" + fname + ")));\n";
            continue;
          } else if (field.value.type.base_type == BASE_TYPE_LONG ||
                     field.value.type.base_type == BASE_TYPE_ULONG) {
            *body += "BigInt(" + fname + " ?? 0));\n";
            continue;
          }

          *body += fname + ");\n";
        }
      }
    }
  }

  std::string GenerateNewExpression(const std::string &object_name) {
    return "new " + namer_.Type(object_name) + "()";
  }

  void GenerateRootAccessor(StructDef &struct_def, std::string *code_ptr,
                            std::string &code, const std::string &object_name,
                            bool size_prefixed) {
    if (!struct_def.fixed) {
      GenDocComment(code_ptr);
      std::string sizePrefixed("SizePrefixed");
      code += "static get" + (size_prefixed ? sizePrefixed : "") + "Root" +
              GetPrefixedName(struct_def, "As");
      code += "(bb:flatbuffers.ByteBuffer, obj?:" + object_name +
              "):" + object_name + " {\n";
      if (size_prefixed) {
        code +=
            "  bb.setPosition(bb.position() + "
            "flatbuffers.SIZE_PREFIX_LENGTH);\n";
      }
      code += "  return (obj || " + GenerateNewExpression(object_name);
      code += ").__init(bb.readInt32(bb.position()) + bb.position(), bb);\n";
      code += "}\n\n";
    }
  }

  void GenerateFinisher(StructDef &struct_def, std::string *code_ptr,
                        std::string &code, bool size_prefixed) {
    if (parser_.root_struct_def_ == &struct_def) {
      std::string sizePrefixed("SizePrefixed");
      GenDocComment(code_ptr);

      code += "static finish" + (size_prefixed ? sizePrefixed : "") +
              GetPrefixedName(struct_def) + "Buffer";
      code += "(builder:flatbuffers.Builder, offset:flatbuffers.Offset) {\n";
      code += "  builder.finish(offset";
      if (!parser_.file_identifier_.empty()) {
        code += ", '" + parser_.file_identifier_ + "'";
      }
      if (size_prefixed) {
        if (parser_.file_identifier_.empty()) { code += ", undefined"; }
        code += ", true";
      }
      code += ");\n";
      code += "}\n\n";
    }
  }

  bool UnionHasStringType(const EnumDef &union_enum) {
    return std::any_of(union_enum.Vals().begin(), union_enum.Vals().end(),
                       [](const EnumVal *ev) {
                         return !ev->IsZero() && IsString(ev->union_type);
                       });
  }

  std::string GenUnionGenericTypeTS(const EnumDef &union_enum) {
    // TODO: make it work without any
    // return std::string("T") + (UnionHasStringType(union_enum) ? "|string" :
    // "");
    return std::string("any") +
           (UnionHasStringType(union_enum) ? "|string" : "");
  }

  std::string GenUnionTypeTS(const EnumDef &union_enum, import_set &imports) {
    std::string ret;
    std::set<std::string> type_list;

    for (auto it = union_enum.Vals().begin(); it != union_enum.Vals().end();
         ++it) {
      const auto &ev = **it;
      if (ev.IsZero()) { continue; }

      std::string type = "";
      if (IsString(ev.union_type)) {
        type = "string";  // no need to wrap string type in namespace
      } else if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
        type = AddImport(imports, union_enum, *ev.union_type.struct_def).name;
      } else {
        FLATBUFFERS_ASSERT(false);
      }
      type_list.insert(type);
    }

    for (auto it = type_list.begin(); it != type_list.end(); ++it) {
      ret += *it + ((std::next(it) == type_list.end()) ? "" : "|");
    }

    return ret;
  }

  static bool CheckIfNameClashes(const import_set &imports,
                                 const std::string &name) {
    // TODO: this would be better as a hashset.
    for (auto it = imports.begin(); it != imports.end(); it++) {
      if (it->second.name == name) { return true; }
    }
    return false;
  }

  std::string GenSymbolExpression(const StructDef &struct_def,
                                  const bool has_name_clash,
                                  const std::string &import_name,
                                  const std::string &name,
                                  const std::string &object_name) {
    std::string symbols_expression;

    if (has_name_clash) {
      // We have a name clash
      symbols_expression += import_name + " as " + name;

      if (parser_.opts.generate_object_based_api) {
        symbols_expression += ", " +
                              GetTypeName(struct_def, /*object_api =*/true) +
                              " as " + object_name;
      }
    } else {
      // No name clash, use the provided name
      symbols_expression += name;

      if (parser_.opts.generate_object_based_api) {
        symbols_expression += ", " + object_name;
      }
    }

    return symbols_expression;
  }

  std::string GenSymbolExpression(const EnumDef &enum_def,
                                  const bool has_name_clash,
                                  const std::string &import_name,
                                  const std::string &name,
                                  const std::string &) {
    std::string symbols_expression;
    if (has_name_clash) {
      symbols_expression += import_name + " as " + name;
    } else {
      symbols_expression += name;
    }

    if (enum_def.is_union) {
      symbols_expression += (", " + namer_.Function("unionTo" + name));
      symbols_expression += (", " + namer_.Function("unionListTo" + name));
    }

    return symbols_expression;
  }

  template<typename DefinitionT>
  ImportDefinition AddImport(import_set &imports, const Definition &dependent,
                             const DefinitionT &dependency) {
    // The unique name of the dependency, fully qualified in its namespace.
    const std::string unique_name = GetTypeName(
        dependency, /*object_api = */ false, /*force_ns_wrap=*/true);

    // Look if we have already added this import and return its name if found.
    const auto import_pair = imports.find(unique_name);
    if (import_pair != imports.end()) { return import_pair->second; }

    // Check if this name would have a name clash with another type. Just use
    // the "base" name (properly escaped) without any namespacing applied.
    const std::string import_name = GetTypeName(dependency);
    const bool has_name_clash = CheckIfNameClashes(imports, import_name);

    // If we have a name clash, use the unique name, otherwise use simple name.
    std::string name = has_name_clash ? unique_name : import_name;

    const std::string object_name =
        GetTypeName(dependency, /*object_api=*/true, has_name_clash);

    const std::string symbols_expression = GenSymbolExpression(
        dependency, has_name_clash, import_name, name, object_name);

    std::string bare_file_path;
    std::string rel_file_path;
    const auto &dep_comps = dependent.defined_namespace->components;
    for (size_t i = 0; i < dep_comps.size(); i++) {
      rel_file_path += i == 0 ? ".." : (kPathSeparator + std::string(".."));
    }
    if (dep_comps.size() == 0) { rel_file_path += "."; }

    bare_file_path +=
        kPathSeparator +
        namer_.Directories(dependency.defined_namespace->components,
                           SkipDir::OutputPath) +
        namer_.File(dependency, SkipFile::SuffixAndExtension);
    rel_file_path += bare_file_path;

    ImportDefinition import;
    import.name = name;
    import.object_name = object_name;
    import.bare_file_path = bare_file_path;
    import.rel_file_path = rel_file_path;
    std::string import_extension = parser_.opts.ts_no_import_ext ? "" : ".js";
    import.import_statement = "import { " + symbols_expression + " } from '" +
                              rel_file_path + import_extension + "';";
    import.export_statement = "export { " + symbols_expression + " } from '." +
                              bare_file_path + import_extension + "';";
    import.dependency = &dependency;
    import.dependent = &dependent;

    imports.insert(std::make_pair(unique_name, import));

    return import;
  }

  void AddImport(import_set &imports, std::string import_name,
                 std::string fileName) {
    ImportDefinition import;
    import.name = import_name;
    import.import_statement =
        "import " + import_name + " from '" + fileName + "';";
    imports.insert(std::make_pair(import_name, import));
  }

  // Generate a TS union type based on a union's enum
  std::string GenObjApiUnionTypeTS(import_set &imports,
                                   const StructDef &dependent,
                                   const IDLOptions &,
                                   const EnumDef &union_enum) {
    std::string ret = "";
    std::set<std::string> type_list;

    for (auto it = union_enum.Vals().begin(); it != union_enum.Vals().end();
         ++it) {
      const auto &ev = **it;
      if (ev.IsZero()) { continue; }

      std::string type = "";
      if (IsString(ev.union_type)) {
        type = "string";  // no need to wrap string type in namespace
      } else if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
        type = AddImport(imports, dependent, *ev.union_type.struct_def)
                   .object_name;
      } else {
        FLATBUFFERS_ASSERT(false);
      }
      type_list.insert(type);
    }

    size_t totalPrinted = 0;
    for (auto it = type_list.begin(); it != type_list.end(); ++it) {
      ++totalPrinted;
      ret += *it + ((totalPrinted == type_list.size()) ? "" : "|");
    }

    return ret;
  }

  std::string GenUnionConvFuncName(const EnumDef &enum_def) {
    return namer_.Function("unionTo", enum_def);
  }

  std::string GenUnionListConvFuncName(const EnumDef &enum_def) {
    return namer_.Function("unionListTo", enum_def);
  }

  std::string GenUnionConvFunc(const Type &union_type, import_set &imports) {
    if (union_type.enum_def) {
      const auto &enum_def = *union_type.enum_def;

      const auto valid_union_type = GenUnionTypeTS(enum_def, imports);
      const auto valid_union_type_with_null = valid_union_type + "|null";

      auto ret = "\n\nexport function " + GenUnionConvFuncName(enum_def) +
                 "(\n  type: " + GetTypeName(enum_def) +
                 ",\n  accessor: (obj:" + valid_union_type + ") => " +
                 valid_union_type_with_null +
                 "\n): " + valid_union_type_with_null + " {\n";

      const auto enum_type = AddImport(imports, enum_def, enum_def).name;

      const auto union_enum_loop = [&](const std::string &accessor_str) {
        ret += "  switch(" + enum_type + "[type]) {\n";
        ret += "    case 'NONE': return null; \n";

        for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
             ++it) {
          const auto &ev = **it;
          if (ev.IsZero()) { continue; }

          ret += "    case '" + namer_.Variant(ev) + "': ";

          if (IsString(ev.union_type)) {
            ret += "return " + accessor_str + "'') as string;";
          } else if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
            const auto type =
                AddImport(imports, enum_def, *ev.union_type.struct_def).name;
            ret += "return " + accessor_str + "new " + type + "())! as " +
                   type + ";";
          } else {
            FLATBUFFERS_ASSERT(false);
          }
          ret += "\n";
        }

        ret += "    default: return null;\n";
        ret += "  }\n";
      };

      union_enum_loop("accessor(");
      ret += "}";

      ret += "\n\nexport function " + GenUnionListConvFuncName(enum_def) +
             "(\n  type: " + GetTypeName(enum_def) +
             ", \n  accessor: (index: number, obj:" + valid_union_type +
             ") => " + valid_union_type_with_null +
             ", \n  index: number\n): " + valid_union_type_with_null + " {\n";
      union_enum_loop("accessor(index, ");
      ret += "}";

      return ret;
    }
    FLATBUFFERS_ASSERT(0);
    return "";
  }

  // Used for generating a short function that returns the correct class
  // based on union enum type. Assume the context is inside the non object api
  // type
  std::string GenUnionValTS(import_set &imports, const StructDef &dependent,
                            const std::string &field_name,
                            const Type &union_type,
                            const bool is_array = false) {
    if (union_type.enum_def) {
      const auto &enum_def = *union_type.enum_def;
      const auto enum_type = AddImport(imports, dependent, enum_def).name;
      const std::string union_accessor = "this." + field_name;

      const auto union_has_string = UnionHasStringType(enum_def);
      const auto field_binded_method = "this." + field_name + ".bind(this)";

      std::string ret;

      if (!is_array) {
        const auto conversion_function = GenUnionConvFuncName(enum_def);

        ret = "(() => {\n";
        ret += "      const temp = " + conversion_function + "(this." +
               namer_.Method(field_name, "Type") + "(), " +
               field_binded_method + ");\n";
        ret += "      if(temp === null) { return null; }\n";
        ret += union_has_string
                   ? "      if(typeof temp === 'string') { return temp; }\n"
                   : "";
        ret += "      return temp.unpack()\n";
        ret += "  })()";
      } else {
        const auto conversion_function = GenUnionListConvFuncName(enum_def);

        ret = "(() => {\n";
        ret += "    const ret: (" +
               GenObjApiUnionTypeTS(imports, *union_type.struct_def,
                                    parser_.opts, *union_type.enum_def) +
               ")[] = [];\n";
        ret += "    for(let targetEnumIndex = 0; targetEnumIndex < this." +
               namer_.Method(field_name, "TypeLength") + "()" +
               "; "
               "++targetEnumIndex) {\n";
        ret += "      const targetEnum = this." +
               namer_.Method(field_name, "Type") + "(targetEnumIndex);\n";
        ret += "      if(targetEnum === null || " + enum_type +
               "[targetEnum!] === 'NONE') { "
               "continue; }\n\n";
        ret += "      const temp = " + conversion_function + "(targetEnum, " +
               field_binded_method + ", targetEnumIndex);\n";
        ret += "      if(temp === null) { continue; }\n";
        ret += union_has_string ? "      if(typeof temp === 'string') { "
                                  "ret.push(temp); continue; }\n"
                                : "";
        ret += "      ret.push(temp.unpack());\n";
        ret += "    }\n";
        ret += "    return ret;\n";
        ret += "  })()";
      }

      return ret;
    }

    FLATBUFFERS_ASSERT(0);
    return "";
  }

  static std::string GenNullCheckConditional(
      const std::string &nullCheckVar, const std::string &trueVal,
      const std::string &falseVal = "null") {
    return "(" + nullCheckVar + " !== null ? " + trueVal + " : " + falseVal +
           ")";
  }

  std::string GenStructMemberValueTS(const StructDef &struct_def,
                                     const std::string &prefix,
                                     const std::string &delimiter,
                                     const bool nullCheck = true) {
    std::string ret;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;

      auto curr_member_accessor = prefix + "." + namer_.Method(field);
      if (prefix != "this") {
        curr_member_accessor = prefix + "?." + namer_.Method(field);
      }
      if (IsStruct(field.value.type)) {
        ret += GenStructMemberValueTS(*field.value.type.struct_def,
                                      curr_member_accessor, delimiter);
      } else {
        if (nullCheck) {
          std::string nullValue = "0";
          if (field.value.type.base_type == BASE_TYPE_BOOL) {
            nullValue = "false";
          } else if (field.value.type.base_type == BASE_TYPE_LONG ||
                     field.value.type.base_type == BASE_TYPE_ULONG) {
            nullValue = "BigInt(0)";
          } else if (field.value.type.base_type == BASE_TYPE_ARRAY) {
            nullValue = "[]";
          }
          ret += "(" + curr_member_accessor + " ?? " + nullValue + ")";
        } else {
          ret += curr_member_accessor;
        }
      }

      if (std::next(it) != struct_def.fields.vec.end()) { ret += delimiter; }
    }

    return ret;
  }

  void GenObjApi(const Parser &parser, StructDef &struct_def,
                 std::string &obj_api_unpack_func, std::string &obj_api_class,
                 import_set &imports) {
    const auto class_name = GetTypeName(struct_def, /*object_api=*/true);

    std::string unpack_func = "\nunpack(): " + class_name +
                              " {\n  return new " + class_name + "(" +
                              (struct_def.fields.vec.empty() ? "" : "\n");
    std::string unpack_to_func = "\nunpackTo(_o: " + class_name + "): void {" +
                                 +(struct_def.fields.vec.empty() ? "" : "\n");

    std::string constructor_func = "constructor(";
    constructor_func += (struct_def.fields.vec.empty() ? "" : "\n");

    const auto has_create =
        struct_def.fixed || CanCreateFactoryMethod(struct_def);

    std::string pack_func_prototype =
        "\npack(builder:flatbuffers.Builder): flatbuffers.Offset {\n";

    std::string pack_func_offset_decl;
    std::string pack_func_create_call;

    const auto struct_name = AddImport(imports, struct_def, struct_def).name;

    if (has_create) {
      pack_func_create_call = "  return " + struct_name + ".create" +
                              GetPrefixedName(struct_def) + "(builder" +
                              (struct_def.fields.vec.empty() ? "" : ",\n    ");
    } else {
      pack_func_create_call = "  " + struct_name + ".start" +
                              GetPrefixedName(struct_def) + "(builder);\n";
    }

    if (struct_def.fixed) {
      // when packing struct, nested struct's members instead of the struct's
      // offset are used
      pack_func_create_call +=
          GenStructMemberValueTS(struct_def, "this", ",\n    ", false) + "\n  ";
    }

    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      const auto field_method = namer_.Method(field);
      const auto field_field = namer_.Field(field);
      const std::string field_binded_method =
          "this." + field_method + ".bind(this)";

      std::string field_val;
      std::string field_type;
      // a string that declares a variable containing the
      // offset for things that can't be generated inline
      // empty otw
      std::string field_offset_decl;
      // a string that contains values for things that can be created inline or
      // the variable name from field_offset_decl
      std::string field_offset_val;
      const auto field_default_val = GenDefaultValue(field, imports);

      // Emit a scalar field
      const auto is_string = IsString(field.value.type);
      if (IsScalar(field.value.type.base_type) || is_string) {
        const auto has_null_default = is_string || HasNullDefault(field);

        field_type += GenTypeName(imports, field, field.value.type, false,
                                  has_null_default);
        field_val = "this." + namer_.Method(field) + "()";

        if (field.value.type.base_type != BASE_TYPE_STRING) {
          field_offset_val = "this." + namer_.Field(field);
        } else {
          field_offset_decl = GenNullCheckConditional(
              "this." + namer_.Field(field),
              "builder.createString(this." + field_field + "!)", "0");
        }
      }

      // Emit an object field
      else {
        auto is_vector = false;
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            const auto &sd = *field.value.type.struct_def;
            field_type += AddImport(imports, struct_def, sd).object_name;

            const std::string field_accessor =
                "this." + namer_.Method(field) + "()";
            field_val = GenNullCheckConditional(field_accessor,
                                                field_accessor + "!.unpack()");
            auto packing = GenNullCheckConditional(
                "this." + field_field,
                "this." + field_field + "!.pack(builder)", "0");

            if (sd.fixed) {
              field_offset_val = std::move(packing);
            } else {
              field_offset_decl = std::move(packing);
            }

            break;
          }

          case BASE_TYPE_ARRAY: {
            auto vectortype = field.value.type.VectorType();
            auto vectortypename =
                GenTypeName(imports, struct_def, vectortype, false);
            is_vector = true;

            field_type = "(";

            switch (vectortype.base_type) {
              case BASE_TYPE_STRUCT: {
                const auto &sd = *field.value.type.struct_def;
                const auto field_type_name =
                    GetTypeName(sd, /*object_api=*/true);
                field_type += field_type_name;
                field_type += ")[]";

                field_val = GenBBAccess() + ".createObjList<" + vectortypename +
                            ", " + field_type_name + ">(" +
                            field_binded_method + ", " +
                            NumToString(field.value.type.fixed_length) + ")";

                if (sd.fixed) {
                  field_offset_decl =
                      "builder.createStructOffsetList(this." + field_field +
                      ", " + AddImport(imports, struct_def, struct_def).name +
                      "." + namer_.Method("start", field, "Vector") + ")";
                } else {
                  field_offset_decl =
                      AddImport(imports, struct_def, struct_def).name + "." +
                      namer_.Method("create", field, "Vector") +
                      "(builder, builder.createObjectOffsetList(" + "this." +
                      field_field + "))";
                }

                break;
              }

              case BASE_TYPE_STRING: {
                field_type += "string)[]";
                field_val = GenBBAccess() + ".createScalarList<string>(" +
                            field_binded_method + ", this." +
                            namer_.Field(field, "Length") + "())";
                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, builder.createObjectOffsetList(" + "this." +
                    namer_.Field(field) + "))";
                break;
              }

              case BASE_TYPE_UNION: {
                field_type += GenObjApiUnionTypeTS(
                    imports, struct_def, parser.opts, *(vectortype.enum_def));
                field_type += ")[]";
                field_val = GenUnionValTS(imports, struct_def, field_method,
                                          vectortype, true);

                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, builder.createObjectOffsetList(" + "this." +
                    namer_.Field(field) + "))";

                break;
              }
              default: {
                if (vectortype.enum_def) {
                  field_type += GenTypeName(imports, struct_def, vectortype,
                                            false, HasNullDefault(field));
                } else {
                  field_type += vectortypename;
                }
                field_type += ")[]";
                field_val = GenBBAccess() + ".createScalarList<" +
                            vectortypename + ">(" + field_binded_method + ", " +
                            NumToString(field.value.type.fixed_length) + ")";

                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, this." + field_field + ")";

                break;
              }
            }

            break;
          }

          case BASE_TYPE_VECTOR: {
            auto vectortype = field.value.type.VectorType();
            auto vectortypename =
                GenTypeName(imports, struct_def, vectortype, false);
            is_vector = true;

            field_type = "(";

            switch (vectortype.base_type) {
              case BASE_TYPE_STRUCT: {
                const auto &sd = *field.value.type.struct_def;
                const auto field_type_name =
                    GetTypeName(sd, /*object_api=*/true);
                field_type += field_type_name;
                field_type += ")[]";

                field_val = GenBBAccess() + ".createObjList<" + vectortypename +
                            ", " + field_type_name + ">(" +
                            field_binded_method + ", this." +
                            namer_.Method(field, "Length") + "())";

                if (sd.fixed) {
                  field_offset_decl =
                      "builder.createStructOffsetList(this." + field_field +
                      ", " + AddImport(imports, struct_def, struct_def).name +
                      "." + namer_.Method("start", field, "Vector") + ")";
                } else {
                  field_offset_decl =
                      AddImport(imports, struct_def, struct_def).name + "." +
                      namer_.Method("create", field, "Vector") +
                      "(builder, builder.createObjectOffsetList(" + "this." +
                      field_field + "))";
                }

                break;
              }

              case BASE_TYPE_STRING: {
                field_type += "string)[]";
                field_val = GenBBAccess() + ".createScalarList<string>(" +
                            field_binded_method + ", this." +
                            namer_.Field(field, "Length") + "())";
                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, builder.createObjectOffsetList(" + "this." +
                    namer_.Field(field) + "))";
                break;
              }

              case BASE_TYPE_UNION: {
                field_type += GenObjApiUnionTypeTS(
                    imports, struct_def, parser.opts, *(vectortype.enum_def));
                field_type += ")[]";
                field_val = GenUnionValTS(imports, struct_def, field_method,
                                          vectortype, true);

                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, builder.createObjectOffsetList(" + "this." +
                    namer_.Field(field) + "))";

                break;
              }
              default: {
                if (vectortype.enum_def) {
                  field_type += GenTypeName(imports, struct_def, vectortype,
                                            false, HasNullDefault(field));
                } else {
                  field_type += vectortypename;
                }
                field_type += ")[]";
                field_val = GenBBAccess() + ".createScalarList<" +
                            vectortypename + ">(" + field_binded_method +
                            ", this." + namer_.Method(field, "Length") + "())";

                field_offset_decl =
                    AddImport(imports, struct_def, struct_def).name + "." +
                    namer_.Method("create", field, "Vector") +
                    "(builder, this." + field_field + ")";

                break;
              }
            }

            break;
          }

          case BASE_TYPE_UNION: {
            field_type += GenObjApiUnionTypeTS(imports, struct_def, parser.opts,
                                               *(field.value.type.enum_def));

            field_val = GenUnionValTS(imports, struct_def, field_method,
                                      field.value.type);
            field_offset_decl =
                "builder.createObjectOffset(this." + field_field + ")";
            break;
          }

          default: FLATBUFFERS_ASSERT(0); break;
        }

        // length 0 vector is simply empty instead of null
        field_type += is_vector ? "" : "|null";
      }

      if (!field_offset_decl.empty()) {
        field_offset_decl =
            "  const " + field_field + " = " + field_offset_decl + ";";
      }
      if (field_offset_val.empty()) { field_offset_val = field_field; }

      unpack_func += "    " + field_val;
      unpack_to_func += "  _o." + field_field + " = " + field_val + ";";

      // FIXME: if field_type and field_field are identical, then
      // this generates invalid typescript.
      constructor_func += "  public " + field_field + ": " + field_type +
                          " = " + field_default_val;

      if (!struct_def.fixed) {
        if (!field_offset_decl.empty()) {
          pack_func_offset_decl += field_offset_decl + "\n";
        }

        if (has_create) {
          pack_func_create_call += field_offset_val;
        } else {
          if (field.IsScalarOptional()) {
            pack_func_create_call +=
                "  if (" + field_offset_val + " !== null)\n  ";
          }
          pack_func_create_call += "  " + struct_name + "." +
                                   namer_.Method("add", field) + "(builder, " +
                                   field_offset_val + ");\n";
        }
      }

      if (std::next(it) != struct_def.fields.vec.end()) {
        constructor_func += ",\n";

        if (!struct_def.fixed && has_create) {
          pack_func_create_call += ",\n    ";
        }

        unpack_func += ",\n";
        unpack_to_func += "\n";
      } else {
        constructor_func += "\n";
        if (!struct_def.fixed) {
          pack_func_offset_decl += (pack_func_offset_decl.empty() ? "" : "\n");
          pack_func_create_call += "\n  ";
        }

        unpack_func += "\n  ";
        unpack_to_func += "\n";
      }
    }

    constructor_func += "){}\n\n";

    if (has_create) {
      pack_func_create_call += ");";
    } else {
      pack_func_create_call += "return " + struct_name + ".end" +
                               GetPrefixedName(struct_def) + "(builder);";
    }
    obj_api_class = "\n";
    obj_api_class += "export class ";
    obj_api_class += GetTypeName(struct_def, /*object_api=*/true);
    obj_api_class += " implements flatbuffers.IGeneratedObject {\n";
    obj_api_class += constructor_func;
    obj_api_class += pack_func_prototype + pack_func_offset_decl +
                     pack_func_create_call + "\n}";

    obj_api_class += "\n}\n";

    unpack_func += ");\n}";
    unpack_to_func += "}\n";

    obj_api_unpack_func = unpack_func + "\n\n" + unpack_to_func;
  }

  static bool CanCreateFactoryMethod(const StructDef &struct_def) {
    // to preserve backwards compatibility, we allow the first field to be a
    // struct
    return struct_def.fields.vec.size() < 2 ||
           std::all_of(std::begin(struct_def.fields.vec) + 1,
                       std::end(struct_def.fields.vec),
                       [](const FieldDef *f) -> bool {
                         FLATBUFFERS_ASSERT(f != nullptr);
                         return f->value.type.base_type != BASE_TYPE_STRUCT;
                       });
  }

  // Generate an accessor struct with constructor for a flatbuffers struct.
  void GenStruct(const Parser &parser, StructDef &struct_def,
                 std::string *code_ptr, import_set &imports) {
    if (struct_def.generated) return;
    std::string &code = *code_ptr;

    // Special case for the root struct, since no one will necessarily reference
    // it, we have to explicitly add it to the import list.
    if (&struct_def == parser_.root_struct_def_) {
      AddImport(imports, struct_def, struct_def);
    }

    const std::string object_name = GetTypeName(struct_def);
    const std::string object_api_name = GetTypeName(struct_def, true);

    // Emit constructor
    GenDocComment(struct_def.doc_comment, code_ptr);
    code += "export class ";
    code += object_name;
    if (parser.opts.generate_object_based_api)
      code += " implements flatbuffers.IUnpackableObject<" + object_api_name +
              "> {\n";
    else
      code += " {\n";
    code += "  bb: flatbuffers.ByteBuffer|null = null;\n";
    code += "  bb_pos = 0;\n";

    // Generate the __init method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    code +=
        "  __init(i:number, bb:flatbuffers.ByteBuffer):" + object_name + " {\n";
    code += "  this.bb_pos = i;\n";
    code += "  this.bb = bb;\n";
    code += "  return this;\n";
    code += "}\n\n";

    // Generate special accessors for the table that when used as the root of a
    // FlatBuffer
    GenerateRootAccessor(struct_def, code_ptr, code, object_name, false);
    GenerateRootAccessor(struct_def, code_ptr, code, object_name, true);

    // Generate the identifier check method
    if (!struct_def.fixed && parser_.root_struct_def_ == &struct_def &&
        !parser_.file_identifier_.empty()) {
      GenDocComment(code_ptr);
      code +=
          "static bufferHasIdentifier(bb:flatbuffers.ByteBuffer):boolean "
          "{\n";
      code += "  return bb.__has_identifier('" + parser_.file_identifier_;
      code += "');\n}\n\n";
    }

    // Emit field accessors
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      std::string offset_prefix = "";

      if (field.value.type.base_type == BASE_TYPE_ARRAY) {
        offset_prefix = "    return ";
      } else {
        offset_prefix = "  const offset = " + GenBBAccess() +
                        ".__offset(this.bb_pos, " +
                        NumToString(field.value.offset) + ");\n";
        offset_prefix += "  return offset ? ";
      }

      // Emit a scalar field
      const auto is_string = IsString(field.value.type);
      if (IsScalar(field.value.type.base_type) || is_string) {
        const auto has_null_default = is_string || HasNullDefault(field);

        GenDocComment(field.doc_comment, code_ptr);
        std::string prefix = namer_.Method(field) + "(";
        if (is_string) {
          code += prefix + "):string|null\n";
          code +=
              prefix + "optionalEncoding:flatbuffers.Encoding" + "):" +
              GenTypeName(imports, struct_def, field.value.type, false, true) +
              "\n";
          code += prefix + "optionalEncoding?:any";
        } else {
          code += prefix;
        }
        if (field.value.type.enum_def) {
          code += "):" +
                  GenTypeName(imports, struct_def, field.value.type, false,
                              field.IsOptional()) +
                  " {\n";
        } else {
          code += "):" +
                  GenTypeName(imports, struct_def, field.value.type, false,
                              has_null_default) +
                  " {\n";
        }

        if (struct_def.fixed) {
          code +=
              "  return " +
              GenGetter(field.value.type,
                        "(this.bb_pos" + MaybeAdd(field.value.offset) + ")") +
              ";\n";
        } else {
          std::string index = "this.bb_pos + offset";
          if (is_string) { index += ", optionalEncoding"; }
          code +=
              offset_prefix + GenGetter(field.value.type, "(" + index + ")");
          if (field.value.type.base_type != BASE_TYPE_ARRAY) {
            code += " : " + GenDefaultValue(field, imports);
          }
          code += ";\n";
        }
      }

      // Emit an object field
      else {
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            const auto type =
                AddImport(imports, struct_def, *field.value.type.struct_def)
                    .name;
            GenDocComment(field.doc_comment, code_ptr);
            code += namer_.Method(field);
            code += "(obj?:" + type + "):" + type + "|null {\n";

            if (struct_def.fixed) {
              code += "  return (obj || " + GenerateNewExpression(type);
              code += ").__init(this.bb_pos";
              code +=
                  MaybeAdd(field.value.offset) + ", " + GenBBAccess() + ");\n";
            } else {
              code += offset_prefix + "(obj || " + GenerateNewExpression(type) +
                      ").__init(";
              code += field.value.type.struct_def->fixed
                          ? "this.bb_pos + offset"
                          : GenBBAccess() + ".__indirect(this.bb_pos + offset)";
              code += ", " + GenBBAccess() + ") : null;\n";
            }

            break;
          }

          case BASE_TYPE_ARRAY: {
            auto vectortype = field.value.type.VectorType();
            auto vectortypename =
                GenTypeName(imports, struct_def, vectortype, false);
            auto inline_size = InlineSize(vectortype);
            auto index = "this.bb_pos + " + NumToString(field.value.offset) +
                         " + index" + MaybeScale(inline_size);
            std::string ret_type;
            bool is_union = false;
            switch (vectortype.base_type) {
              case BASE_TYPE_STRUCT: ret_type = vectortypename; break;
              case BASE_TYPE_STRING: ret_type = vectortypename; break;
              case BASE_TYPE_UNION:
                ret_type = "?flatbuffers.Table";
                is_union = true;
                break;
              default: ret_type = vectortypename;
            }
            GenDocComment(field.doc_comment, code_ptr);
            std::string prefix = namer_.Method(field);
            // TODO: make it work without any
            // if (is_union) { prefix += "<T extends flatbuffers.Table>"; }
            if (is_union) { prefix += ""; }
            prefix += "(index: number";
            if (is_union) {
              const auto union_type =
                  GenUnionGenericTypeTS(*(field.value.type.enum_def));

              vectortypename = union_type;
              code += prefix + ", obj:" + union_type;
            } else if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += prefix + ", obj?:" + vectortypename;
            } else if (IsString(vectortype)) {
              code += prefix + "):string\n";
              code += prefix + ",optionalEncoding:flatbuffers.Encoding" +
                      "):" + vectortypename + "\n";
              code += prefix + ",optionalEncoding?:any";
            } else {
              code += prefix;
            }
            code += "):" + vectortypename + "|null {\n";

            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += offset_prefix + "(obj || " +
                      GenerateNewExpression(vectortypename);
              code += ").__init(";
              code += vectortype.struct_def->fixed
                          ? index
                          : GenBBAccess() + ".__indirect(" + index + ")";
              code += ", " + GenBBAccess() + ")";
            } else {
              if (is_union) {
                index = "obj, " + index;
              } else if (IsString(vectortype)) {
                index += ", optionalEncoding";
              }
              code += offset_prefix + GenGetter(vectortype, "(" + index + ")");
            }

            switch (field.value.type.base_type) {
              case BASE_TYPE_ARRAY: {
                break;
              }
              case BASE_TYPE_BOOL: {
                code += " : false";
                break;
              }
              case BASE_TYPE_LONG:
              case BASE_TYPE_ULONG: {
                code += " : BigInt(0)";
                break;
              }
              default: {
                if (IsScalar(field.value.type.element)) {
                  if (field.value.type.enum_def) {
                    code += field.value.constant;
                  } else {
                    code += " : 0";
                  }
                } else {
                  code += ": null";
                }
                break;
              }
            }
            code += ";\n";
            break;
          }

          case BASE_TYPE_VECTOR: {
            auto vectortype = field.value.type.VectorType();
            auto vectortypename =
                GenTypeName(imports, struct_def, vectortype, false);
            auto type = GetUnderlyingVectorType(vectortype);
            auto inline_size = InlineSize(type);
            auto index = GenBBAccess() +
                         ".__vector(this.bb_pos + offset) + index" +
                         MaybeScale(inline_size);
            std::string ret_type;
            bool is_union = false;
            switch (vectortype.base_type) {
              case BASE_TYPE_STRUCT: ret_type = vectortypename; break;
              case BASE_TYPE_STRING: ret_type = vectortypename; break;
              case BASE_TYPE_UNION:
                ret_type = "?flatbuffers.Table";
                is_union = true;
                break;
              default: ret_type = vectortypename;
            }
            GenDocComment(field.doc_comment, code_ptr);
            std::string prefix = namer_.Method(field);
            // TODO: make it work without any
            // if (is_union) { prefix += "<T extends flatbuffers.Table>"; }
            if (is_union) { prefix += ""; }
            prefix += "(index: number";
            if (is_union) {
              const auto union_type =
                  GenUnionGenericTypeTS(*(field.value.type.enum_def));

              vectortypename = union_type;
              code += prefix + ", obj:" + union_type;
            } else if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += prefix + ", obj?:" + vectortypename;
            } else if (IsString(vectortype)) {
              code += prefix + "):string\n";
              code += prefix + ",optionalEncoding:flatbuffers.Encoding" +
                      "):" + vectortypename + "\n";
              code += prefix + ",optionalEncoding?:any";
            } else {
              code += prefix;
            }
            code += "):" + vectortypename + "|null {\n";

            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += offset_prefix + "(obj || " +
                      GenerateNewExpression(vectortypename);
              code += ").__init(";
              code += vectortype.struct_def->fixed
                          ? index
                          : GenBBAccess() + ".__indirect(" + index + ")";
              code += ", " + GenBBAccess() + ")";
            } else {
              if (is_union) {
                index = "obj, " + index;
              } else if (IsString(vectortype)) {
                index += ", optionalEncoding";
              }
              code += offset_prefix + GenGetter(vectortype, "(" + index + ")");
            }
            code += " : ";
            if (field.value.type.element == BASE_TYPE_BOOL) {
              code += "false";
            } else if (field.value.type.element == BASE_TYPE_LONG ||
                       field.value.type.element == BASE_TYPE_ULONG) {
              code += "BigInt(0)";
            } else if (IsScalar(field.value.type.element)) {
              if (field.value.type.enum_def) {
                code += field.value.constant;
              } else {
                code += "0";
              }
            } else {
              code += "null";
            }
            code += ";\n";
            break;
          }

          case BASE_TYPE_UNION: {
            GenDocComment(field.doc_comment, code_ptr);
            code += namer_.Method(field);

            const auto &union_enum = *(field.value.type.enum_def);
            const auto union_type = GenUnionGenericTypeTS(union_enum);
            code += "<T extends flatbuffers.Table>(obj:" + union_type +
                    "):" + union_type +
                    "|null "
                    "{\n";

            code += offset_prefix +
                    GenGetter(field.value.type, "(obj, this.bb_pos + offset)") +
                    " : null;\n";
            break;
          }
          default: FLATBUFFERS_ASSERT(0);
        }
      }
      code += "}\n\n";

      // Adds the mutable scalar value to the output
      if (IsScalar(field.value.type.base_type) && parser.opts.mutable_buffer &&
          !IsUnion(field.value.type)) {
        std::string type =
            GenTypeName(imports, struct_def, field.value.type, true);

        code += namer_.LegacyTsMutateMethod(field) + "(value:" + type +
                "):boolean {\n";

        const std::string write_method =
            "." + namer_.Method("write", GenType(field.value.type));

        if (struct_def.fixed) {
          code += "  " + GenBBAccess() + write_method + "(this.bb_pos + " +
                  NumToString(field.value.offset) + ", ";
        } else {
          code += "  const offset = " + GenBBAccess() +
                  ".__offset(this.bb_pos, " + NumToString(field.value.offset) +
                  ");\n\n";
          code += "  if (offset === 0) {\n";
          code += "    return false;\n";
          code += "  }\n\n";

          // special case for bools, which are treated as uint8
          code +=
              "  " + GenBBAccess() + write_method + "(this.bb_pos + offset, ";
          if (field.value.type.base_type == BASE_TYPE_BOOL) { code += "+"; }
        }

        code += "value);\n";
        code += "  return true;\n";
        code += "}\n\n";
      }

      // Emit vector helpers
      if (IsVector(field.value.type)) {
        // Emit a length helper
        GenDocComment(code_ptr);
        code += namer_.Method(field, "Length");
        code += "():number {\n" + offset_prefix;

        code +=
            GenBBAccess() + ".__vector_len(this.bb_pos + offset) : 0;\n}\n\n";

        // For scalar types, emit a typed array helper
        auto vectorType = field.value.type.VectorType();
        if (IsScalar(vectorType.base_type) && !IsLong(vectorType.base_type)) {
          GenDocComment(code_ptr);

          code += namer_.Method(field, "Array");
          code +=
              "():" + GenType(vectorType) + "Array|null {\n" + offset_prefix;

          code += "new " + GenType(vectorType) + "Array(" + GenBBAccess() +
                  ".bytes().buffer, " + GenBBAccess() +
                  ".bytes().byteOffset + " + GenBBAccess() +
                  ".__vector(this.bb_pos + offset), " + GenBBAccess() +
                  ".__vector_len(this.bb_pos + offset)) : null;\n}\n\n";
        }
      }
    }

    // Emit the fully qualified name
    if (parser_.opts.generate_name_strings) {
      GenDocComment(code_ptr);
      code += "static getFullyQualifiedName():string {\n";
      code +=
          "  return '" +
          struct_def.defined_namespace->GetFullyQualifiedName(struct_def.name) +
          "';\n";
      code += "}\n\n";
    }

    // Emit the size of the struct.
    if (struct_def.fixed) {
      GenDocComment(code_ptr);
      code += "static sizeOf():number {\n";
      code += "  return " + NumToString(struct_def.bytesize) + ";\n";
      code += "}\n\n";
    }

    // Emit a factory constructor
    if (struct_def.fixed) {
      std::string arguments;
      GenStructArgs(imports, struct_def, &arguments, "");
      GenDocComment(code_ptr);

      code += "static create" + GetPrefixedName(struct_def) +
              "(builder:flatbuffers.Builder";
      code += arguments + "):flatbuffers.Offset {\n";

      GenStructBody(struct_def, &code, "");
      code += "  return builder.offset();\n}\n\n";
    } else {
      // Generate a method to start building a new object
      GenDocComment(code_ptr);

      code += "static start" + GetPrefixedName(struct_def) +
              "(builder:flatbuffers.Builder) {\n";

      code += "  builder.startObject(" +
              NumToString(struct_def.fields.vec.size()) + ");\n";
      code += "}\n\n";

      // Generate a set of static methods that allow table construction
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        const auto argname = GetArgName(field);

        // Generate the field insertion method
        GenDocComment(code_ptr);
        code += "static " + namer_.Method("add", field);
        code += "(builder:flatbuffers.Builder, " + argname + ":" +
                GetArgType(imports, struct_def, field, false) + ") {\n";
        code += "  builder.addField" + GenWriteMethod(field.value.type) + "(";
        code += NumToString(it - struct_def.fields.vec.begin()) + ", ";
        if (field.value.type.base_type == BASE_TYPE_BOOL) { code += "+"; }
        code += argname + ", ";
        if (!IsScalar(field.value.type.base_type)) {
          code += "0";
        } else if (HasNullDefault(field)) {
          code += "null";
        } else {
          if (field.value.type.base_type == BASE_TYPE_BOOL) { code += "+"; }
          code += GenDefaultValue(field, imports);
        }
        code += ");\n}\n\n";

        if (IsVector(field.value.type)) {
          auto vector_type = field.value.type.VectorType();
          auto type = GetUnderlyingVectorType(vector_type);
          auto alignment = InlineAlignment(type);
          auto elem_size = InlineSize(type);

          // Generate a method to create a vector from a JavaScript array
          if (!IsStruct(vector_type)) {
            GenDocComment(code_ptr);

            const std::string sig_begin =
                "static " + namer_.Method("create", field, "Vector") +
                "(builder:flatbuffers.Builder, data:";
            const std::string sig_end = "):flatbuffers.Offset";
            std::string type =
                GenTypeName(imports, struct_def, vector_type, true) + "[]";
            if (type == "number[]") {
              const auto &array_type = GenType(vector_type);
              // the old type should be deprecated in the future
              std::string type_old = "number[]|Uint8Array";
              std::string type_new = "number[]|" + array_type + "Array";
              if (type_old == type_new) {
                type = type_new;
              } else {
                // add function overloads
                code += sig_begin + type_new + sig_end + ";\n";
                code +=
                    "/**\n * @deprecated This Uint8Array overload will "
                    "be removed in the future.\n */\n";
                code += sig_begin + type_old + sig_end + ";\n";
                type = type_new + "|Uint8Array";
              }
            }
            code += sig_begin + type + sig_end + " {\n";
            code += "  builder.startVector(" + NumToString(elem_size);
            code += ", data.length, " + NumToString(alignment) + ");\n";
            code += "  for (let i = data.length - 1; i >= 0; i--) {\n";
            code += "    builder.add" + GenWriteMethod(vector_type) + "(";
            if (vector_type.base_type == BASE_TYPE_BOOL) { code += "+"; }
            code += "data[i]!);\n";
            code += "  }\n";
            code += "  return builder.endVector();\n";
            code += "}\n\n";
          }

          // Generate a method to start a vector, data to be added manually
          // after
          GenDocComment(code_ptr);

          code += "static ";
          code += namer_.Method("start", field, "Vector");
          code += "(builder:flatbuffers.Builder, numElems:number) {\n";
          code += "  builder.startVector(" + NumToString(elem_size);
          code += ", numElems, " + NumToString(alignment) + ");\n";
          code += "}\n\n";
        }
      }

      // Generate a method to stop building a new object
      GenDocComment(code_ptr);

      code += "static end" + GetPrefixedName(struct_def);
      code += "(builder:flatbuffers.Builder):flatbuffers.Offset {\n";

      code += "  const offset = builder.endObject();\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (!field.deprecated && field.IsRequired()) {
          code += "  builder.requiredField(offset, ";
          code += NumToString(field.value.offset);
          code += ") // " + field.name + "\n";
        }
      }
      code += "  return offset;\n";
      code += "}\n\n";

      // Generate the methods to complete buffer construction
      GenerateFinisher(struct_def, code_ptr, code, false);
      GenerateFinisher(struct_def, code_ptr, code, true);

      // Generate a convenient CreateX function
      if (CanCreateFactoryMethod(struct_def)) {
        code += "static create" + GetPrefixedName(struct_def);
        code += "(builder:flatbuffers.Builder";
        for (auto it = struct_def.fields.vec.begin();
             it != struct_def.fields.vec.end(); ++it) {
          const auto &field = **it;
          if (field.deprecated) continue;
          code += ", " + GetArgName(field) + ":" +
                  GetArgType(imports, struct_def, field, true);
        }

        code += "):flatbuffers.Offset {\n";
        code += "  " + object_name + ".start" + GetPrefixedName(struct_def) +
                "(builder);\n";

        std::string methodPrefix = object_name;
        for (auto it = struct_def.fields.vec.begin();
             it != struct_def.fields.vec.end(); ++it) {
          const auto &field = **it;
          if (field.deprecated) continue;

          const auto arg_name = GetArgName(field);

          if (field.IsScalarOptional()) {
            code += "  if (" + arg_name + " !== null)\n  ";
          }

          code += "  " + methodPrefix + "." + namer_.Method("add", field) + "(";
          code += "builder, " + arg_name + ");\n";
        }

        code += "  return " + methodPrefix + ".end" +
                GetPrefixedName(struct_def) + "(builder);\n";
        code += "}\n";
      }
    }

    if (!struct_def.fixed && parser_.services_.vec.size() != 0) {
      auto name = GetPrefixedName(struct_def, "");
      code += "\n";
      code += "serialize():Uint8Array {\n";
      code += "  return this.bb!.bytes();\n";
      code += "}\n";

      code += "\n";
      code += "static deserialize(buffer: Uint8Array):" +
              namer_.EscapeKeyword(name) + " {\n";
      code += "  return " + AddImport(imports, struct_def, struct_def).name +
              ".getRootAs" + name + "(new flatbuffers.ByteBuffer(buffer))\n";
      code += "}\n";
    }

    if (parser_.opts.generate_object_based_api) {
      std::string obj_api_class;
      std::string obj_api_unpack_func;
      GenObjApi(parser_, struct_def, obj_api_unpack_func, obj_api_class,
                imports);

      code += obj_api_unpack_func + "}\n" + obj_api_class;
    } else {
      code += "}\n";
    }
  }

  static bool HasNullDefault(const FieldDef &field) {
    return field.IsOptional() && field.value.constant == "null";
  }

  std::string GetArgType(import_set &imports, const Definition &owner,
                         const FieldDef &field, bool allowNull) {
    return GenTypeName(imports, owner, field.value.type, true,
                       allowNull && field.IsOptional());
  }

  std::string GetArgName(const FieldDef &field) {
    auto argname = namer_.Variable(field);
    if (!IsScalar(field.value.type.base_type)) { argname += "Offset"; }
    return argname;
  }

  std::string GetPrefixedName(const StructDef &struct_def,
                              const char *prefix = "") {
    return prefix + struct_def.name;
  }
};  // namespace ts
}  // namespace ts

static bool GenerateTS(const Parser &parser, const std::string &path,
                       const std::string &file_name) {
  ts::TsGenerator generator(parser, path, file_name);
  return generator.generate();
}

static std::string TSMakeRule(const Parser &parser, const std::string &path,
                              const std::string &file_name) {
  std::string filebase =
      flatbuffers::StripPath(flatbuffers::StripExtension(file_name));
  ts::TsGenerator generator(parser, path, file_name);
  std::string make_rule =
      generator.GeneratedFileName(path, filebase, parser.opts) + ": ";

  auto included_files = parser.GetIncludedFilesRecursive(file_name);
  for (auto it = included_files.begin(); it != included_files.end(); ++it) {
    make_rule += " " + *it;
  }
  return make_rule;
}

namespace {

class TsCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateTS(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = TSMakeRule(parser, path, filename);
    return Status::OK;
  }

  Status GenerateGrpcCode(const Parser &parser, const std::string &path,
                          const std::string &filename) override {
    if (!GenerateTSGRPC(parser, path, filename)) { return Status::ERROR; }
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

  IDLOptions::Language Language() const override { return IDLOptions::kTs; }

  std::string LanguageName() const override { return "TS"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewTsCodeGenerator() {
  return std::unique_ptr<TsCodeGenerator>(new TsCodeGenerator());
}

}  // namespace flatbuffers
