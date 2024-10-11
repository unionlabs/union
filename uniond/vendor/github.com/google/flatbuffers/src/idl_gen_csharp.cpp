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

#include "idl_gen_csharp.h"

#include <unordered_set>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"

namespace flatbuffers {

static TypedFloatConstantGenerator CSharpFloatGen("Double.", "Single.", "NaN",
                                                  "PositiveInfinity",
                                                  "NegativeInfinity");
static CommentConfig comment_config = {
  nullptr,
  "///",
  nullptr,
};

namespace csharp {
class CSharpGenerator : public BaseGenerator {
  struct FieldArrayLength {
    std::string name;
    int length;
  };

 public:
  CSharpGenerator(const Parser &parser, const std::string &path,
                  const std::string &file_name)
      : BaseGenerator(parser, path, file_name,
                      parser.opts.cs_global_alias ? "global::" : "", ".", "cs"),
        cur_name_space_(nullptr) {
    // clang-format off

    // List of keywords retrieved from here:
    // https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/

    // One per line to ease comparisons to that list are easier

    static const char *const keywords[] = {
      "abstract",
      "as",
      "base",
      "bool",
      "break",
      "byte",
      "case",
      "catch",
      "char",
      "checked",
      "class",
      "const",
      "continue",
      "decimal",
      "default",
      "delegate",
      "do",
      "double",
      "else",
      "enum",
      "event",
      "explicit",
      "extern",
      "false",
      "finally",
      "fixed",
      "float",
      "for",
      "foreach",
      "goto",
      "if",
      "implicit",
      "in",
      "int",
      "interface",
      "internal",
      "is",
      "lock",
      "long",
      "namespace",
      "new",
      "null",
      "object",
      "operator",
      "out",
      "override",
      "params",
      "private",
      "protected",
      "public",
      "readonly",
      "ref",
      "return",
      "sbyte",
      "sealed",
      "short",
      "sizeof",
      "stackalloc",
      "static",
      "string",
      "struct",
      "switch",
      "this",
      "throw",
      "true",
      "try",
      "typeof",
      "uint",
      "ulong",
      "unchecked",
      "unsafe",
      "ushort",
      "using",
      "virtual",
      "void",
      "volatile",
      "while",
      nullptr,
      // clang-format on
    };

    for (auto kw = keywords; *kw; kw++) keywords_.insert(*kw);
  }

  CSharpGenerator &operator=(const CSharpGenerator &);

  bool generate() {
    std::string one_file_code;
    cur_name_space_ = parser_.current_namespace_;

    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      std::string enumcode;
      auto &enum_def = **it;
      if (!parser_.opts.one_file) cur_name_space_ = enum_def.defined_namespace;
      GenEnum(enum_def, &enumcode, parser_.opts);
      if (parser_.opts.one_file) {
        one_file_code += enumcode;
      } else {
        if (!SaveType(enum_def.name, *enum_def.defined_namespace, enumcode,
                      false, parser_.opts))
          return false;
      }
    }

    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      std::string declcode;
      auto &struct_def = **it;
      if (!parser_.opts.one_file)
        cur_name_space_ = struct_def.defined_namespace;
      GenStruct(struct_def, &declcode, parser_.opts);
      GenStructVerifier(struct_def, &declcode);
      if (parser_.opts.one_file) {
        one_file_code += declcode;
      } else {
        if (!SaveType(struct_def.name, *struct_def.defined_namespace, declcode,
                      true, parser_.opts))
          return false;
      }
    }

    if (parser_.opts.one_file) {
      return SaveType(file_name_, *parser_.current_namespace_, one_file_code,
                      true, parser_.opts);
    }
    return true;
  }

 private:
  std::unordered_set<std::string> keywords_;

  std::string EscapeKeyword(const std::string &name) const {
    return keywords_.find(name) == keywords_.end() ? name : "@" + name;
  }

  std::string Name(const FieldDef &field) const {
    std::string name = ConvertCase(field.name, Case::kUpperCamel);
    return EscapeKeyword(name);
  }

  std::string Name(const Definition &def) const {
    return EscapeKeyword(def.name);
  }

  std::string NamespacedName(const Definition &def) const {
    return WrapInNameSpace(def.defined_namespace, Name(def));
  }

  std::string Name(const EnumVal &ev) const { return EscapeKeyword(ev.name); }

  // Save out the generated code for a single class while adding
  // declaration boilerplate.
  bool SaveType(const std::string &defname, const Namespace &ns,
                const std::string &classcode, bool needs_includes,
                const IDLOptions &options) const {
    if (!classcode.length()) return true;

    std::string code =
        "// <auto-generated>\n"
        "//  " +
        std::string(FlatBuffersGeneratedWarning()) +
        "\n"
        "// </auto-generated>\n\n";

    std::string namespace_name = FullNamespace(".", ns);
    if (!namespace_name.empty()) {
      code += "namespace " + namespace_name + "\n{\n\n";
    }
    if (needs_includes) {
      code += "using global::System;\n";
      code += "using global::System.Collections.Generic;\n";
      code += "using global::Google.FlatBuffers;\n\n";
    }
    code += classcode;
    if (!namespace_name.empty()) { code += "\n}\n"; }
    auto filename = NamespaceDir(ns) + defname;
    if (options.one_file) { filename += options.filename_suffix; }
    filename +=
        options.filename_extension.empty() ? ".cs" : options.filename_extension;
    return SaveFile(filename.c_str(), code, false);
  }

  const Namespace *CurrentNameSpace() const { return cur_name_space_; }

  std::string GenTypeBasic(const Type &type, bool enableLangOverrides) const {
    // clang-format off
    static const char * const csharp_typename[] = {
      #define FLATBUFFERS_TD(ENUM, IDLTYPE, CTYPE, JTYPE, GTYPE, NTYPE, ...) \
        #NTYPE,
        FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
      #undef FLATBUFFERS_TD
    };
    // clang-format on

    if (enableLangOverrides) {
      if (IsEnum(type)) return NamespacedName(*type.enum_def);
      if (type.base_type == BASE_TYPE_STRUCT) {
        return "Offset<" + NamespacedName(*type.struct_def) + ">";
      }
    }

    return csharp_typename[type.base_type];
  }

  inline std::string GenTypeBasic(const Type &type) const {
    return GenTypeBasic(type, true);
  }

  std::string GenTypePointer(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "string";
      case BASE_TYPE_VECTOR: return GenTypeGet(type.VectorType());
      case BASE_TYPE_STRUCT: return NamespacedName(*type.struct_def);
      case BASE_TYPE_UNION: return "TTable";
      default: return "Table";
    }
  }

  std::string GenTypeGet(const Type &type) const {
    return IsScalar(type.base_type)
               ? GenTypeBasic(type)
               : (IsArray(type) ? GenTypeGet(type.VectorType())
                                : GenTypePointer(type));
  }

  std::string GenOffsetType(const StructDef &struct_def) const {
    return "Offset<" + NamespacedName(struct_def) + ">";
  }

  std::string GenOffsetConstruct(const StructDef &struct_def,
                                 const std::string &variable_name) const {
    return "new Offset<" + NamespacedName(struct_def) + ">(" + variable_name +
           ")";
  }

  // Casts necessary to correctly read serialized data
  std::string DestinationCast(const Type &type) const {
    if (IsSeries(type)) {
      return DestinationCast(type.VectorType());
    } else {
      if (IsEnum(type)) return "(" + NamespacedName(*type.enum_def) + ")";
    }
    return "";
  }

  // Cast statements for mutator method parameters.
  // In Java, parameters representing unsigned numbers need to be cast down to
  // their respective type. For example, a long holding an unsigned int value
  // would be cast down to int before being put onto the buffer. In C#, one cast
  // directly cast an Enum to its underlying type, which is essential before
  // putting it onto the buffer.
  std::string SourceCast(const Type &type,
                         const bool isOptional = false) const {
    if (IsSeries(type)) {
      return SourceCast(type.VectorType());
    } else {
      if (IsEnum(type))
        return "(" + GenTypeBasic(type, false) + (isOptional ? "?" : "") + ")";
    }
    return "";
  }

  std::string SourceCastBasic(const Type &type, const bool isOptional) const {
    return IsScalar(type.base_type) ? SourceCast(type, isOptional) : "";
  }

  std::string GenEnumDefaultValue(const FieldDef &field) const {
    auto &value = field.value;
    FLATBUFFERS_ASSERT(value.type.enum_def);
    auto &enum_def = *value.type.enum_def;
    auto enum_val = enum_def.FindByValue(value.constant);
    return enum_val ? (NamespacedName(enum_def) + "." + Name(*enum_val))
                    : value.constant;
  }

  std::string GenDefaultValue(const FieldDef &field,
                              bool enableLangOverrides) const {
    // If it is an optional scalar field, the default is null
    if (field.IsScalarOptional()) { return "null"; }

    auto &value = field.value;
    if (enableLangOverrides) {
      // handles both enum case and vector of enum case
      if (value.type.enum_def != nullptr &&
          value.type.base_type != BASE_TYPE_UNION) {
        return GenEnumDefaultValue(field);
      }
    }

    auto longSuffix = "";
    switch (value.type.base_type) {
      case BASE_TYPE_BOOL: return value.constant == "0" ? "false" : "true";
      case BASE_TYPE_ULONG: return value.constant;
      case BASE_TYPE_UINT:
      case BASE_TYPE_LONG: return value.constant + longSuffix;
      default:
        if (IsFloat(value.type.base_type))
          return CSharpFloatGen.GenFloatConstant(field);
        else
          return value.constant;
    }
  }

  std::string GenDefaultValue(const FieldDef &field) const {
    return GenDefaultValue(field, true);
  }

  std::string GenDefaultValueBasic(const FieldDef &field,
                                   bool enableLangOverrides) const {
    auto &value = field.value;
    if (!IsScalar(value.type.base_type)) {
      if (enableLangOverrides) {
        switch (value.type.base_type) {
          case BASE_TYPE_STRING: return "default(StringOffset)";
          case BASE_TYPE_STRUCT:
            return "default(Offset<" + NamespacedName(*value.type.struct_def) +
                   ">)";
          case BASE_TYPE_VECTOR: return "default(VectorOffset)";
          default: break;
        }
      }
      return "0";
    }
    return GenDefaultValue(field, enableLangOverrides);
  }

  std::string GenDefaultValueBasic(const FieldDef &field) const {
    return GenDefaultValueBasic(field, true);
  }

  void GenEnum(EnumDef &enum_def, std::string *code_ptr,
               const IDLOptions &opts) const {
    std::string &code = *code_ptr;
    if (enum_def.generated) return;

    // Generate enum definitions of the form:
    // public static (final) int name = value;
    // In Java, we use ints rather than the Enum feature, because we want them
    // to map directly to how they're used in C/C++ and file formats.
    // That, and Java Enums are expensive, and not universally liked.
    GenComment(enum_def.doc_comment, code_ptr, &comment_config);

    if (opts.cs_gen_json_serializer && opts.generate_object_based_api) {
      code +=
          "[Newtonsoft.Json.JsonConverter(typeof(Newtonsoft.Json.Converters."
          "StringEnumConverter))]\n";
    }
    // In C# this indicates enumeration values can be treated as bit flags.
    if (enum_def.attributes.Lookup("bit_flags")) {
      code += "[System.FlagsAttribute]\n";
    }
    if (enum_def.attributes.Lookup("private")) {
      code += "internal ";
    } else {
      code += "public ";
    }
    code += "enum " + Name(enum_def);
    code += " : " + GenTypeBasic(enum_def.underlying_type, false);
    code += "\n{\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      GenComment(ev.doc_comment, code_ptr, &comment_config, "  ");
      code += "  ";
      code += Name(ev) + " = ";
      code += enum_def.ToString(ev);
      code += ",\n";
    }
    // Close the class
    code += "};\n\n";

    if (opts.generate_object_based_api) {
      GenEnum_ObjectAPI(enum_def, code_ptr, opts);
    }

    if (enum_def.is_union) {
      code += GenUnionVerify(enum_def.underlying_type);
    }
  }

  bool HasUnionStringValue(const EnumDef &enum_def) const {
    if (!enum_def.is_union) return false;
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &val = **it;
      if (IsString(val.union_type)) { return true; }
    }
    return false;
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetter(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "__p.__string";
      case BASE_TYPE_STRUCT: return "__p.__struct";
      case BASE_TYPE_UNION: return "__p.__union";
      case BASE_TYPE_VECTOR: return GenGetter(type.VectorType());
      case BASE_TYPE_ARRAY: return GenGetter(type.VectorType());
      default: {
        std::string getter = "__p.bb.Get";
        if (type.base_type == BASE_TYPE_BOOL) {
          getter = "0!=" + getter;
        } else if (GenTypeBasic(type, false) != "byte") {
          getter += ConvertCase(GenTypeBasic(type, false), Case::kUpperCamel);
        }
        return getter;
      }
    }
  }

  std::string GetObjectConstructor(flatbuffers::StructDef &struct_def,
                                   const std::string &data_buffer,
                                   const std::string &offset) const {
    // Use the generated type directly, to properly handle default values that
    // might not be written to the buffer.
    return "new " + Name(struct_def) + "().__assign(" + offset + ", " +
           data_buffer + ")";
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetterForLookupByKey(flatbuffers::StructDef &struct_def,
                                      flatbuffers::FieldDef *key_field,
                                      const std::string &data_buffer,
                                      const std::string &offset) const {
    // Use the generated type directly, to properly handle default values that
    // might not be written to the buffer.
    auto name = Name(*key_field);
    if (name == struct_def.name) { name += "_"; }
    return GetObjectConstructor(struct_def, data_buffer, offset) + "." + name;
  }

  // Direct mutation is only allowed for scalar fields.
  // Hence a setter method will only be generated for such fields.
  std::string GenSetter(const Type &type) const {
    if (IsScalar(type.base_type)) {
      std::string setter = "__p.bb.Put";
      if (GenTypeBasic(type, false) != "byte" &&
          type.base_type != BASE_TYPE_BOOL) {
        setter += ConvertCase(GenTypeBasic(type, false), Case::kUpperCamel);
      }
      return setter;
    } else {
      return "";
    }
  }

  // Returns the method name for use with add/put calls.
  std::string GenMethod(const Type &type) const {
    return IsScalar(type.base_type)
               ? ConvertCase(GenTypeBasic(type, false), Case::kUpperCamel)
               : (IsStruct(type) ? "Struct" : "Offset");
  }

  // Recursively generate arguments for a constructor, to deal with nested
  // structs.
  void GenStructArgs(const StructDef &struct_def, std::string *code_ptr,
                     const char *nameprefix, size_t array_count = 0) const {
    std::string &code = *code_ptr;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      const auto array_field = IsArray(field_type);
      const auto &type = array_field ? field_type.VectorType() : field_type;
      const auto array_cnt = array_field ? (array_count + 1) : array_count;
      if (IsStruct(type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        GenStructArgs(*field_type.struct_def, code_ptr,
                      (nameprefix + (EscapeKeyword(field.name) + "_")).c_str(),
                      array_cnt);
      } else {
        code += ", ";
        code += GenTypeBasic(type);
        if (field.IsScalarOptional()) { code += "?"; }
        if (array_cnt > 0) {
          code += "[";
          for (size_t i = 1; i < array_cnt; i++) code += ",";
          code += "]";
        }
        code += " ";
        code += nameprefix;
        code += Name(field);
      }
    }
  }

  // Recusively generate struct construction statements of the form:
  // builder.putType(name);
  // and insert manual padding.
  void GenStructBody(const StructDef &struct_def, std::string *code_ptr,
                     const char *nameprefix, size_t index = 0,
                     bool in_array = false) const {
    std::string &code = *code_ptr;
    std::string indent((index + 1) * 2, ' ');
    code += indent + "  builder.Prep(";
    code += NumToString(struct_def.minalign) + ", ";
    code += NumToString(struct_def.bytesize) + ");\n";
    for (auto it = struct_def.fields.vec.rbegin();
         it != struct_def.fields.vec.rend(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      if (field.padding) {
        code += indent + "  builder.Pad(";
        code += NumToString(field.padding) + ");\n";
      }
      if (IsStruct(field_type)) {
        GenStructBody(*field_type.struct_def, code_ptr,
                      (nameprefix + (field.name + "_")).c_str(), index,
                      in_array);
      } else {
        const auto &type =
            IsArray(field_type) ? field_type.VectorType() : field_type;
        const auto index_var = "_idx" + NumToString(index);
        if (IsArray(field_type)) {
          code += indent + "  for (int " + index_var + " = ";
          code += NumToString(field_type.fixed_length);
          code += "; " + index_var + " > 0; " + index_var + "--) {\n";
          in_array = true;
        }
        if (IsStruct(type)) {
          GenStructBody(*field_type.struct_def, code_ptr,
                        (nameprefix + (field.name + "_")).c_str(), index + 1,
                        in_array);
        } else {
          code += IsArray(field_type) ? "  " : "";
          code += indent + "  builder.Put";
          code += GenMethod(type) + "(";
          code += SourceCast(type);
          auto argname = nameprefix + Name(field);
          code += argname;
          size_t array_cnt = index + (IsArray(field_type) ? 1 : 0);
          if (array_cnt > 0) {
            code += "[";
            for (size_t i = 0; in_array && i < array_cnt; i++) {
              code += "_idx" + NumToString(i) + "-1";
              if (i != (array_cnt - 1)) code += ",";
            }
            code += "]";
          }
          code += ");\n";
        }
        if (IsArray(field_type)) { code += indent + "  }\n"; }
      }
    }
  }
  std::string GenOffsetGetter(flatbuffers::FieldDef *key_field,
                              const char *num = nullptr) const {
    std::string key_offset =
        "Table.__offset(" + NumToString(key_field->value.offset) + ", ";
    if (num) {
      key_offset += num;
      key_offset += ".Value, builder.DataBuffer)";
    } else {
      key_offset += "bb.Length";
      key_offset += " - tableOffset, bb)";
    }
    return key_offset;
  }

  std::string GenKeyGetter(flatbuffers::StructDef &struct_def,
                           flatbuffers::FieldDef *key_field) const {
    // Get the getter for the key of the struct.
    return GenGetterForLookupByKey(struct_def, key_field, "builder.DataBuffer",
                                   "builder.DataBuffer.Length - o1.Value") +
           ".CompareTo(" +
           GenGetterForLookupByKey(struct_def, key_field, "builder.DataBuffer",
                                   "builder.DataBuffer.Length - o2.Value") +
           ")";
  }

  // Get the value of a table verification function start
  void GetStartOfTableVerifier(const StructDef &struct_def,
                               std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\n";
    code += "static public class " + struct_def.name + "Verify\n";
    code += "{\n";
    code += "  static public bool Verify";
    code += "(Google.FlatBuffers.Verifier verifier, uint tablePos)\n";
    code += "  {\n";
    code += "    return verifier.VerifyTableStart(tablePos)\n";
  }

  // Get the value of a table verification function end
  void GetEndOfTableVerifier(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "      && verifier.VerifyTableEnd(tablePos);\n";
    code += "  }\n";
    code += "}\n";
  }

  std::string GetNestedFlatBufferName(const FieldDef &field) {
    std::string name;
    if (field.nested_flatbuffer) {
      name = NamespacedName(*field.nested_flatbuffer);
    } else {
      name = "";
    }
    return name;
  }

  // Generate the code to call the appropriate Verify function(s) for a field.
  void GenVerifyCall(CodeWriter &code_, const FieldDef &field,
                     const char *prefix) {
    code_.SetValue("PRE", prefix);
    code_.SetValue("NAME", ConvertCase(field.name, Case::kUpperCamel));
    code_.SetValue("REQUIRED", field.IsRequired() ? "Required" : "");
    code_.SetValue("REQUIRED_FLAG", field.IsRequired() ? "true" : "false");
    code_.SetValue("TYPE", GenTypeGet(field.value.type));
    code_.SetValue("INLINESIZE", NumToString(InlineSize(field.value.type)));
    code_.SetValue("OFFSET", NumToString(field.value.offset));

    if (IsScalar(field.value.type.base_type) || IsStruct(field.value.type)) {
      code_.SetValue("ALIGN", NumToString(InlineAlignment(field.value.type)));
      code_ +=
          "{{PRE}}      && verifier.VerifyField(tablePos, "
          "{{OFFSET}} /*{{NAME}}*/, {{INLINESIZE}} /*{{TYPE}}*/, {{ALIGN}}, "
          "{{REQUIRED_FLAG}})";
    } else {
      // TODO - probably code below should go to this 'else' - code_ +=
      // "{{PRE}}VerifyOffset{{REQUIRED}}(verifier, {{OFFSET}})\\";
    }

    switch (field.value.type.base_type) {
      case BASE_TYPE_UNION: {
        auto union_name = NamespacedName(*field.value.type.enum_def);
        code_.SetValue("ENUM_NAME1", field.value.type.enum_def->name);
        code_.SetValue("ENUM_NAME", union_name);
        code_.SetValue("SUFFIX", UnionTypeFieldSuffix());
        // Caution: This construction assumes, that UNION type id element has
        // been created just before union data and its offset precedes union.
        // Such assumption is common in flatbuffer implementation
        code_.SetValue("TYPE_ID_OFFSET",
                       NumToString(field.value.offset - sizeof(voffset_t)));
        code_ +=
            "{{PRE}}      && verifier.VerifyUnion(tablePos, "
            "{{TYPE_ID_OFFSET}}, "
            "{{OFFSET}} /*{{NAME}}*/, {{ENUM_NAME}}Verify.Verify, "
            "{{REQUIRED_FLAG}})";
        break;
      }
      case BASE_TYPE_STRUCT: {
        if (!field.value.type.struct_def->fixed) {
          code_ +=
              "{{PRE}}      && verifier.VerifyTable(tablePos, "
              "{{OFFSET}} /*{{NAME}}*/, {{TYPE}}Verify.Verify, "
              "{{REQUIRED_FLAG}})";
        }
        break;
      }
      case BASE_TYPE_STRING: {
        code_ +=
            "{{PRE}}      && verifier.VerifyString(tablePos, "
            "{{OFFSET}} /*{{NAME}}*/, {{REQUIRED_FLAG}})";
        break;
      }
      case BASE_TYPE_VECTOR: {
        switch (field.value.type.element) {
          case BASE_TYPE_STRING: {
            code_ +=
                "{{PRE}}      && verifier.VerifyVectorOfStrings(tablePos, "
                "{{OFFSET}} /*{{NAME}}*/, {{REQUIRED_FLAG}})";
            break;
          }
          case BASE_TYPE_STRUCT: {
            if (!field.value.type.struct_def->fixed) {
              code_ +=
                  "{{PRE}}      && verifier.VerifyVectorOfTables(tablePos, "
                  "{{OFFSET}} /*{{NAME}}*/, {{TYPE}}Verify.Verify, "
                  "{{REQUIRED_FLAG}})";
            } else {
              code_.SetValue(
                  "VECTOR_ELEM_INLINESIZE",
                  NumToString(InlineSize(field.value.type.VectorType())));
              code_ +=
                  "{{PRE}}      && "
                  "verifier.VerifyVectorOfData(tablePos, "
                  "{{OFFSET}} /*{{NAME}}*/, {{VECTOR_ELEM_INLINESIZE}} "
                  "/*{{TYPE}}*/, {{REQUIRED_FLAG}})";
            }
            break;
          }
          case BASE_TYPE_UNION: {
            // Vectors of unions are not yet supported for go
            break;
          }
          default:
            // Generate verifier for vector of data.
            // It may be either nested flatbuffer of just vector of bytes
            auto nfn = GetNestedFlatBufferName(field);
            if (!nfn.empty()) {
              code_.SetValue("CPP_NAME", nfn);
              // FIXME: file_identifier.
              code_ +=
                  "{{PRE}}      && verifier.VerifyNestedBuffer(tablePos, "
                  "{{OFFSET}} /*{{NAME}}*/, {{CPP_NAME}}Verify.Verify, "
                  "{{REQUIRED_FLAG}})";
            } else if (field.flexbuffer) {
              code_ +=
                  "{{PRE}}      && verifier.VerifyNestedBuffer(tablePos, "
                  "{{OFFSET}} /*{{NAME}}*/, null, {{REQUIRED_FLAG}})";
            } else {
              code_.SetValue(
                  "VECTOR_ELEM_INLINESIZE",
                  NumToString(InlineSize(field.value.type.VectorType())));
              code_ +=
                  "{{PRE}}      && verifier.VerifyVectorOfData(tablePos, "
                  "{{OFFSET}} /*{{NAME}}*/, {{VECTOR_ELEM_INLINESIZE}} "
                  "/*{{TYPE}}*/, {{REQUIRED_FLAG}})";
            }
            break;
        }

        break;
      }
      default: {
        break;
      }
    }
  }

  // Generate table constructors, conditioned on its members' types.
  void GenTableVerifier(const StructDef &struct_def, std::string *code_ptr) {
    CodeWriter code_;

    GetStartOfTableVerifier(struct_def, code_ptr);

    // Generate struct fields accessors
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      GenVerifyCall(code_, field, "");
    }

    *code_ptr += code_.ToString();

    GetEndOfTableVerifier(code_ptr);
  }

  // Generate struct or table methods.
  void GenStructVerifier(const StructDef &struct_def, std::string *code_ptr) {
    if (struct_def.generated) return;

    // cur_name_space_ = struct_def.defined_namespace;

    // Generate verifiers
    if (struct_def.fixed) {
      // Fixed size structures do not require table members
      // verification - instead structure size is verified using VerifyField
    } else {
      // Create table verification function
      GenTableVerifier(struct_def, code_ptr);
    }
  }

  void GenStruct(StructDef &struct_def, std::string *code_ptr,
                 const IDLOptions &opts) const {
    if (struct_def.generated) return;
    std::string &code = *code_ptr;

    // Generate a struct accessor class, with methods of the form:
    // public type name() { return bb.getType(i + offset); }
    // or for tables of the form:
    // public type name() {
    //   int o = __offset(offset); return o != 0 ? bb.getType(o + i) : default;
    // }
    GenComment(struct_def.doc_comment, code_ptr, &comment_config);
    if (struct_def.attributes.Lookup("private")) {
      code += "internal ";
    } else {
      code += "public ";
    }
    if (struct_def.attributes.Lookup("csharp_partial")) {
      // generate a partial class for this C# struct/table
      code += "partial ";
    }
    code += "struct " + struct_def.name;
    code += " : IFlatbufferObject";
    code += "\n{\n";
    code += "  private ";
    code += struct_def.fixed ? "Struct" : "Table";
    code += " __p;\n";

    code += "  public ByteBuffer ByteBuffer { get { return __p.bb; } }\n";

    if (!struct_def.fixed) {
      // Generate version check method.
      // Force compile time error if not using the same version runtime.
      code += "  public static void ValidateVersion() {";
      code += " FlatBufferConstants.";
      code += "FLATBUFFERS_24_3_25(); ";
      code += "}\n";

      // Generate a special accessor for the table that when used as the root
      // of a FlatBuffer
      std::string method_name = "GetRootAs" + struct_def.name;
      std::string method_signature =
          "  public static " + struct_def.name + " " + method_name;

      // create convenience method that doesn't require an existing object
      code += method_signature + "(ByteBuffer _bb) ";
      code += "{ return " + method_name + "(_bb, new " + struct_def.name +
              "()); }\n";

      // create method that allows object reuse
      code +=
          method_signature + "(ByteBuffer _bb, " + struct_def.name + " obj) { ";
      code += "return (obj.__assign(_bb.GetInt(_bb.Position";
      code += ") + _bb.Position";
      code += ", _bb)); }\n";
      if (parser_.root_struct_def_ == &struct_def) {
        if (parser_.file_identifier_.length()) {
          // Check if a buffer has the identifier.
          code += "  public static ";
          code += "bool " + struct_def.name;
          code += "BufferHasIdentifier(ByteBuffer _bb) { return ";
          code += "Table.__has_identifier(_bb, \"";
          code += parser_.file_identifier_;
          code += "\"); }\n";
        }

        // Generate the Verify method that checks if a ByteBuffer is save to
        // access
        code += "  public static ";
        code += "bool Verify" + struct_def.name + "(ByteBuffer _bb) {";
        code += "Google.FlatBuffers.Verifier verifier = new ";
        code += "Google.FlatBuffers.Verifier(_bb); ";
        code += "return verifier.VerifyBuffer(\"";
        code += parser_.file_identifier_;
        code += "\", false, " + struct_def.name + "Verify.Verify);";
        code += " }\n";
      }
    }

    // Generate the __init method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    code += "  public void __init(int _i, ByteBuffer _bb) ";
    code += "{ ";
    code += "__p = new ";
    code += struct_def.fixed ? "Struct" : "Table";
    code += "(_i, _bb); ";
    code += "}\n";
    code +=
        "  public " + struct_def.name + " __assign(int _i, ByteBuffer _bb) ";
    code += "{ __init(_i, _bb); return this; }\n\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      GenComment(field.doc_comment, code_ptr, &comment_config, "  ");
      std::string type_name = GenTypeGet(field.value.type);
      std::string type_name_dest = GenTypeGet(field.value.type);
      std::string conditional_cast = "";
      std::string optional = "";
      if (!struct_def.fixed &&
          (field.value.type.base_type == BASE_TYPE_STRUCT ||
           field.value.type.base_type == BASE_TYPE_UNION ||
           (IsVector(field.value.type) &&
            (field.value.type.element == BASE_TYPE_STRUCT ||
             field.value.type.element == BASE_TYPE_UNION)))) {
        optional = "?";
        conditional_cast = "(" + type_name_dest + optional + ")";
      }
      if (field.IsScalarOptional()) { optional = "?"; }
      std::string dest_mask = "";
      std::string dest_cast = DestinationCast(field.value.type);
      std::string src_cast = SourceCast(field.value.type);
      std::string field_name_camel = Name(field);
      if (field_name_camel == struct_def.name) { field_name_camel += "_"; }
      std::string method_start =
          "  public " + type_name_dest + optional + " " + field_name_camel;
      std::string obj = "(new " + type_name + "())";

      // Most field accessors need to retrieve and test the field offset first,
      // this is the prefix code for that:
      auto offset_prefix =
          IsArray(field.value.type)
              ? " { return "
              : (" { int o = __p.__offset(" + NumToString(field.value.offset) +
                 "); return o != 0 ? ");
      // Generate the accessors that don't do object reuse.
      if (field.value.type.base_type == BASE_TYPE_STRUCT) {
      } else if (IsVector(field.value.type) &&
                 field.value.type.element == BASE_TYPE_STRUCT) {
      } else if (field.value.type.base_type == BASE_TYPE_UNION ||
                 (IsVector(field.value.type) &&
                  field.value.type.VectorType().base_type == BASE_TYPE_UNION)) {
        method_start += "<TTable>";
        type_name = type_name_dest;
      }
      std::string getter = dest_cast + GenGetter(field.value.type);
      code += method_start;
      std::string default_cast = "";
      // only create default casts for c# scalars or vectors of scalars
      if ((IsScalar(field.value.type.base_type) ||
           (IsVector(field.value.type) &&
            IsScalar(field.value.type.element)))) {
        // For scalars, default value will be returned by GetDefaultValue().
        // If the scalar is an enum, GetDefaultValue() returns an actual c# enum
        // that doesn't need to be casted. However, default values for enum
        // elements of vectors are integer literals ("0") and are still casted
        // for clarity.
        // If the scalar is optional and enum, we still need the cast.
        if ((field.value.type.enum_def == nullptr ||
             IsVector(field.value.type)) ||
            (IsEnum(field.value.type) && field.IsScalarOptional())) {
          default_cast = "(" + type_name_dest + optional + ")";
        }
      }
      std::string member_suffix = "; ";
      if (IsScalar(field.value.type.base_type)) {
        code += " { get";
        member_suffix += "} ";
        if (struct_def.fixed) {
          code += " { return " + getter;
          code += "(__p.bb_pos + ";
          code += NumToString(field.value.offset) + ")";
          code += dest_mask;
        } else {
          code += offset_prefix + getter;
          code += "(o + __p.bb_pos)" + dest_mask;
          code += " : " + default_cast;
          code += GenDefaultValue(field);
        }
      } else {
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT:
            code += " { get";
            member_suffix += "} ";
            if (struct_def.fixed) {
              code += " { return " + obj + ".__assign(" + "__p.";
              code += "bb_pos + " + NumToString(field.value.offset) + ", ";
              code += "__p.bb)";
            } else {
              code += offset_prefix + conditional_cast;
              code += obj + ".__assign(";
              code += field.value.type.struct_def->fixed
                          ? "o + __p.bb_pos"
                          : "__p.__indirect(o + __p.bb_pos)";
              code += ", __p.bb) : null";
            }
            break;
          case BASE_TYPE_STRING:
            code += " { get";
            member_suffix += "} ";
            code += offset_prefix + getter + "(o + " + "__p.";
            code += "bb_pos) : null";
            break;
          case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_VECTOR: {
            auto vectortype = field.value.type.VectorType();
            if (vectortype.base_type == BASE_TYPE_UNION) {
              conditional_cast = "(TTable?)";
              getter += "<TTable>";
            }
            code += "(";
            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              getter = obj + ".__assign";
            } else if (vectortype.base_type == BASE_TYPE_UNION) {
            }
            code += "int j)";
            const auto body = offset_prefix + conditional_cast + getter + "(";
            if (vectortype.base_type == BASE_TYPE_UNION) {
              code += " where TTable : struct, IFlatbufferObject" + body;
            } else {
              code += body;
            }
            std::string index = "__p.";
            if (IsArray(field.value.type)) {
              index += "bb_pos + " + NumToString(field.value.offset) + " + ";
            } else {
              index += "__vector(o) + ";
            }
            index += "j * " + NumToString(InlineSize(vectortype));
            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += vectortype.struct_def->fixed
                          ? index
                          : "__p.__indirect(" + index + ")";
              code += ", __p.bb";
            } else {
              code += index;
            }
            code += ")" + dest_mask;
            if (!IsArray(field.value.type)) {
              code += " : ";
              code +=
                  field.value.type.element == BASE_TYPE_BOOL
                      ? "false"
                      : (IsScalar(field.value.type.element) ? default_cast + "0"
                                                            : "null");
            }
            if (vectortype.base_type == BASE_TYPE_UNION &&
                HasUnionStringValue(*vectortype.enum_def)) {
              code += member_suffix;
              code += "}\n";
              code += "  public string " + Name(field) + "AsString(int j)";
              code += offset_prefix + GenGetter(Type(BASE_TYPE_STRING));
              code += "(" + index + ") : null";
            }
            break;
          }
          case BASE_TYPE_UNION:
            code += "() where TTable : struct, IFlatbufferObject";
            code += offset_prefix + "(TTable?)" + getter;
            code += "<TTable>(o + __p.bb_pos) : null";
            if (HasUnionStringValue(*field.value.type.enum_def)) {
              code += member_suffix;
              code += "}\n";
              code += "  public string " + Name(field) + "AsString()";
              code += offset_prefix + GenGetter(Type(BASE_TYPE_STRING));
              code += "(o + __p.bb_pos) : null";
            }
            // As<> accesors for Unions
            // Loop through all the possible union types and generate an As
            // accessor that casts to the correct type.
            for (auto uit = field.value.type.enum_def->Vals().begin();
                 uit != field.value.type.enum_def->Vals().end(); ++uit) {
              auto val = *uit;
              if (val->union_type.base_type == BASE_TYPE_NONE) { continue; }
              auto union_field_type_name = GenTypeGet(val->union_type);
              code += member_suffix + "}\n";
              if (val->union_type.base_type == BASE_TYPE_STRUCT &&
                  val->union_type.struct_def->attributes.Lookup("private")) {
                code += "  internal ";
              } else {
                code += "  public ";
              }
              code += union_field_type_name + " ";
              code += field_name_camel + "As" + val->name + "() { return ";
              code += field_name_camel;
              if (IsString(val->union_type)) {
                code += "AsString()";
              } else {
                code += "<" + union_field_type_name + ">().Value";
              }
            }
            break;
          default: FLATBUFFERS_ASSERT(0);
        }
      }
      code += member_suffix;
      code += "}\n";
      if (IsVector(field.value.type)) {
        auto camel_name = Name(field);
        if (camel_name == struct_def.name) { camel_name += "_"; }
        code += "  public int " + camel_name;
        code += "Length";
        code += " { get";
        code += offset_prefix;
        code += "__p.__vector_len(o) : 0; ";
        code += "} ";
        code += "}\n";
        // See if we should generate a by-key accessor.
        if (field.value.type.element == BASE_TYPE_STRUCT &&
            !field.value.type.struct_def->fixed) {
          auto &sd = *field.value.type.struct_def;
          auto &fields = sd.fields.vec;
          for (auto kit = fields.begin(); kit != fields.end(); ++kit) {
            auto &key_field = **kit;
            if (key_field.key) {
              auto qualified_name = NamespacedName(sd);
              code += "  public " + qualified_name + "? ";
              code += Name(field) + "ByKey(";
              code += GenTypeGet(key_field.value.type) + " key)";
              code += offset_prefix;
              code += qualified_name + ".__lookup_by_key(";
              code += "__p.__vector(o), key, ";
              code += "__p.bb) : null; ";
              code += "}\n";
              break;
            }
          }
        }
      }
      // Generate a ByteBuffer accessor for strings & vectors of scalars.
      if ((IsVector(field.value.type) &&
           IsScalar(field.value.type.VectorType().base_type)) ||
          IsString(field.value.type)) {
        code += "#if ENABLE_SPAN_T\n";
        code += "  public Span<" + GenTypeBasic(field.value.type.VectorType()) +
                "> Get";
        code += Name(field);
        code += "Bytes() { return ";
        code += "__p.__vector_as_span<" +
                GenTypeBasic(field.value.type.VectorType()) + ">(";
        code += NumToString(field.value.offset);
        code +=
            ", " + NumToString(SizeOf(field.value.type.VectorType().base_type));
        code += "); }\n";
        code += "#else\n";
        code += "  public ArraySegment<byte>? Get";
        code += Name(field);
        code += "Bytes() { return ";
        code += "__p.__vector_as_arraysegment(";
        code += NumToString(field.value.offset);
        code += "); }\n";
        code += "#endif\n";

        // For direct blockcopying the data into a typed array
        code += "  public ";
        code += GenTypeBasic(field.value.type.VectorType());
        code += "[] Get";
        code += Name(field);
        code += "Array() { ";
        if (IsEnum(field.value.type.VectorType())) {
          // Since __vector_as_array does not work for enum types,
          // fill array using an explicit loop.
          code += "int o = __p.__offset(";
          code += NumToString(field.value.offset);
          code += "); if (o == 0) return null; int p = ";
          code += "__p.__vector(o); int l = ";
          code += "__p.__vector_len(o); ";
          code += GenTypeBasic(field.value.type.VectorType());
          code += "[] a = new ";
          code += GenTypeBasic(field.value.type.VectorType());
          code += "[l]; for (int i = 0; i < l; i++) { a[i] = " + getter;
          code += "(p + i * ";
          code += NumToString(InlineSize(field.value.type.VectorType()));
          code += "); } return a;";
        } else {
          code += "return ";
          code += "__p.__vector_as_array<";
          code += GenTypeBasic(field.value.type.VectorType());
          code += ">(";
          code += NumToString(field.value.offset);
          code += ");";
        }
        code += " }\n";
      }
      // generate object accessors if is nested_flatbuffer
      if (field.nested_flatbuffer) {
        auto nested_type_name = NamespacedName(*field.nested_flatbuffer);
        auto nested_method_name =
            Name(field) + "As" + field.nested_flatbuffer->name;
        auto get_nested_method_name = nested_method_name;
        get_nested_method_name = "Get" + nested_method_name;
        conditional_cast = "(" + nested_type_name + "?)";
        obj = "(new " + nested_type_name + "())";
        code += "  public " + nested_type_name + "? ";
        code += get_nested_method_name + "(";
        code += ") { int o = __p.__offset(";
        code += NumToString(field.value.offset) + "); ";
        code += "return o != 0 ? " + conditional_cast + obj + ".__assign(";
        code += "__p.";
        code += "__indirect(__p.__vector(o)), ";
        code += "__p.bb) : null; }\n";
      }
      // Generate mutators for scalar fields or vectors of scalars.
      if (parser_.opts.mutable_buffer) {
        auto is_series = (IsSeries(field.value.type));
        const auto &underlying_type =
            is_series ? field.value.type.VectorType() : field.value.type;
        // Boolean parameters have to be explicitly converted to byte
        // representation.
        auto setter_parameter =
            underlying_type.base_type == BASE_TYPE_BOOL
                ? "(byte)(" + EscapeKeyword(field.name) + " ? 1 : 0)"
                : EscapeKeyword(field.name);
        auto mutator_prefix = "Mutate";
        // A vector mutator also needs the index of the vector element it should
        // mutate.
        auto mutator_params = (is_series ? "(int j, " : "(") +
                              GenTypeGet(underlying_type) + " " +
                              EscapeKeyword(field.name) + ") { ";
        auto setter_index =
            is_series
                ? "__p." +
                      (IsArray(field.value.type)
                           ? "bb_pos + " + NumToString(field.value.offset)
                           : "__vector(o)") +
                      +" + j * " + NumToString(InlineSize(underlying_type))
                : (struct_def.fixed
                       ? "__p.bb_pos + " + NumToString(field.value.offset)
                       : "o + __p.bb_pos");
        if (IsScalar(underlying_type.base_type) && !IsUnion(field.value.type)) {
          code += "  public ";
          code += struct_def.fixed ? "void " : "bool ";
          code += mutator_prefix + Name(field);
          code += mutator_params;
          if (struct_def.fixed) {
            code += GenSetter(underlying_type) + "(" + setter_index + ", ";
            code += src_cast + setter_parameter + "); }\n";
          } else {
            code += "int o = __p.__offset(";
            code += NumToString(field.value.offset) + ");";
            code += " if (o != 0) { " + GenSetter(underlying_type);
            code += "(" + setter_index + ", " + src_cast + setter_parameter +
                    "); return true; } else { return false; } }\n";
          }
        }
      }
      if (parser_.opts.java_primitive_has_method &&
          IsScalar(field.value.type.base_type) && !struct_def.fixed) {
        auto vt_offset_constant =
            "  public static final int VT_" +
            ConvertCase(field.name, Case::kScreamingSnake) + " = " +
            NumToString(field.value.offset) + ";";

        code += vt_offset_constant;
        code += "\n";
      }
    }
    code += "\n";
    auto struct_has_create = false;
    std::set<flatbuffers::FieldDef *> field_has_create_set;
    flatbuffers::FieldDef *key_field = nullptr;
    if (struct_def.fixed) {
      struct_has_create = true;
      // create a struct constructor function
      code += "  public static " + GenOffsetType(struct_def) + " ";
      code += "Create";
      code += struct_def.name + "(FlatBufferBuilder builder";
      GenStructArgs(struct_def, code_ptr, "");
      code += ") {\n";
      GenStructBody(struct_def, code_ptr, "");
      code += "    return ";
      code += GenOffsetConstruct(struct_def, "builder.Offset");
      code += ";\n  }\n";
    } else {
      // Generate a method that creates a table in one go. This is only possible
      // when the table has no struct fields, since those have to be created
      // inline, and there's no way to do so in Java.
      bool has_no_struct_fields = true;
      int num_fields = 0;
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        if (IsStruct(field.value.type)) {
          has_no_struct_fields = false;
        } else {
          num_fields++;
        }
      }
      // JVM specifications restrict default constructor params to be < 255.
      // Longs and doubles take up 2 units, so we set the limit to be < 127.
      if ((has_no_struct_fields || opts.generate_object_based_api) &&
          num_fields && num_fields < 127) {
        struct_has_create = true;
        // Generate a table constructor of the form:
        // public static int createName(FlatBufferBuilder builder, args...)
        code += "  public static " + GenOffsetType(struct_def) + " ";
        code += "Create" + struct_def.name;
        code += "(FlatBufferBuilder builder";
        for (auto it = struct_def.fields.vec.begin();
             it != struct_def.fields.vec.end(); ++it) {
          auto &field = **it;
          if (field.deprecated) continue;
          code += ",\n      ";
          if (IsStruct(field.value.type) && opts.generate_object_based_api) {
            code += WrapInNameSpace(
                field.value.type.struct_def->defined_namespace,
                GenTypeName_ObjectAPI(field.value.type.struct_def->name, opts));
            code += " ";
            code += EscapeKeyword(field.name);
            code += " = null";
          } else {
            code += GenTypeBasic(field.value.type);
            if (field.IsScalarOptional()) { code += "?"; }
            code += " ";
            code += EscapeKeyword(field.name);
            if (!IsScalar(field.value.type.base_type)) code += "Offset";

            code += " = ";
            code += GenDefaultValueBasic(field);
          }
        }
        code += ") {\n    builder.";
        code += "StartTable(";
        code += NumToString(struct_def.fields.vec.size()) + ");\n";
        for (size_t size = struct_def.sortbysize ? sizeof(largest_scalar_t) : 1;
             size; size /= 2) {
          for (auto it = struct_def.fields.vec.rbegin();
               it != struct_def.fields.vec.rend(); ++it) {
            auto &field = **it;
            if (!field.deprecated &&
                (!struct_def.sortbysize ||
                 size == SizeOf(field.value.type.base_type))) {
              code += "    " + struct_def.name + ".";
              code += "Add";
              code += Name(field) + "(builder, ";
              if (IsStruct(field.value.type) &&
                  opts.generate_object_based_api) {
                code += GenTypePointer(field.value.type) + ".Pack(builder, " +
                        EscapeKeyword(field.name) + ")";
              } else {
                code += EscapeKeyword(field.name);
                if (!IsScalar(field.value.type.base_type)) code += "Offset";
              }

              code += ");\n";
            }
          }
        }
        code += "    return " + struct_def.name + ".";
        code += "End" + struct_def.name;
        code += "(builder);\n  }\n\n";
      }
      // Generate a set of static methods that allow table construction,
      // of the form:
      // public static void addName(FlatBufferBuilder builder, short name)
      // { builder.addShort(id, name, default); }
      // Unlike the Create function, these always work.
      code += "  public static void Start";
      code += struct_def.name;
      code += "(FlatBufferBuilder builder) { builder.";
      code += "StartTable(";
      code += NumToString(struct_def.fields.vec.size()) + "); }\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        if (field.key) key_field = &field;
        code += "  public static void Add";
        code += Name(field);
        code += "(FlatBufferBuilder builder, ";
        code += GenTypeBasic(field.value.type);
        auto argname = ConvertCase(field.name, Case::kLowerCamel);
        if (!IsScalar(field.value.type.base_type)) argname += "Offset";
        if (field.IsScalarOptional()) { code += "?"; }
        code += " " + EscapeKeyword(argname) + ") { builder.Add";
        code += GenMethod(field.value.type) + "(";
        code += NumToString(it - struct_def.fields.vec.begin()) + ", ";
        code += SourceCastBasic(field.value.type, field.IsScalarOptional());
        code += EscapeKeyword(argname);
        if (!IsScalar(field.value.type.base_type) &&
            field.value.type.base_type != BASE_TYPE_UNION) {
          code += ".Value";
        }
        if (!field.IsScalarOptional()) {
          // When the scalar is optional, use the builder method that doesn't
          // supply a default value. Otherwise, we to continue to use the
          // default value method.
          code += ", ";
          code += GenDefaultValue(field, false);
        }
        code += "); }\n";
        if (IsVector(field.value.type)) {
          auto vector_type = field.value.type.VectorType();
          auto alignment = InlineAlignment(vector_type);
          auto elem_size = InlineSize(vector_type);
          if (!IsStruct(vector_type)) {
            field_has_create_set.insert(&field);
            code += "  public static VectorOffset ";
            code += "Create";
            code += Name(field);
            code += "Vector(FlatBufferBuilder builder, ";
            code += GenTypeBasic(vector_type) + "[] data) ";
            code += "{ builder.StartVector(";
            code += NumToString(elem_size);
            code += ", data.Length, ";
            code += NumToString(alignment);
            code += "); for (int i = data.";
            code += "Length - 1; i >= 0; i--) builder.";
            code += "Add";
            code += GenMethod(vector_type);
            code += "(";
            // At the moment there is no support of the type Vector with
            // optional enum, e.g. if we have enum type SomeEnum there is no way
            // to define `SomeEmum?[] enums` in FlatBuffer schema, so isOptional
            // = false
            code += SourceCastBasic(vector_type, false);
            code += "data[i]";
            if (vector_type.base_type == BASE_TYPE_STRUCT ||
                IsString(vector_type))
              code += ".Value";
            code += "); return ";
            code += "builder.EndVector(); }\n";

            // add Create...VectorBlock() overloads for T[], ArraySegment<T> and
            // IntPtr
            code += "  public static VectorOffset ";
            code += "Create";
            code += Name(field);
            code += "VectorBlock(FlatBufferBuilder builder, ";
            code += GenTypeBasic(vector_type) + "[] data) ";
            code += "{ builder.StartVector(";
            code += NumToString(elem_size);
            code += ", data.Length, ";
            code += NumToString(alignment);
            code += "); builder.Add(data); return builder.EndVector(); }\n";

            code += "  public static VectorOffset ";
            code += "Create";
            code += Name(field);
            code += "VectorBlock(FlatBufferBuilder builder, ";
            code += "ArraySegment<" + GenTypeBasic(vector_type) + "> data) ";
            code += "{ builder.StartVector(";
            code += NumToString(elem_size);
            code += ", data.Count, ";
            code += NumToString(alignment);
            code += "); builder.Add(data); return builder.EndVector(); }\n";

            code += "  public static VectorOffset ";
            code += "Create";
            code += Name(field);
            code += "VectorBlock(FlatBufferBuilder builder, ";
            code += "IntPtr dataPtr, int sizeInBytes) ";
            code += "{ builder.StartVector(1, sizeInBytes, 1); ";
            code += "builder.Add<" + GenTypeBasic(vector_type) +
                    ">(dataPtr, sizeInBytes); return builder.EndVector(); }\n";
          }
          // Generate a method to start a vector, data to be added manually
          // after.
          code += "  public static void Start";
          code += Name(field);
          code += "Vector(FlatBufferBuilder builder, int numElems) ";
          code += "{ builder.StartVector(";
          code += NumToString(elem_size);
          code += ", numElems, " + NumToString(alignment);
          code += "); }\n";
        }
      }
      code += "  public static " + GenOffsetType(struct_def) + " ";
      code += "End" + struct_def.name;
      code += "(FlatBufferBuilder builder) {\n    int o = builder.";
      code += "EndTable();\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (!field.deprecated && field.IsRequired()) {
          code += "    builder.Required(o, ";
          code += NumToString(field.value.offset);
          code += ");  // " + field.name + "\n";
        }
      }
      code += "    return " + GenOffsetConstruct(struct_def, "o") + ";\n  }\n";
      if (parser_.root_struct_def_ == &struct_def) {
        std::string size_prefix[] = { "", "SizePrefixed" };
        for (int i = 0; i < 2; ++i) {
          code += "  public static void ";
          code += "Finish" + size_prefix[i] + struct_def.name;
          code +=
              "Buffer(FlatBufferBuilder builder, " + GenOffsetType(struct_def);
          code += " offset) {";
          code += " builder.Finish" + size_prefix[i] + "(offset";
          code += ".Value";

          if (parser_.file_identifier_.length())
            code += ", \"" + parser_.file_identifier_ + "\"";
          code += "); }\n";
        }
      }
    }
    // Only generate key compare function for table,
    // because `key_field` is not set for struct
    if (struct_def.has_key && !struct_def.fixed) {
      FLATBUFFERS_ASSERT(key_field);
      auto name = Name(*key_field);
      if (name == struct_def.name) { name += "_"; }
      code += "\n  public static VectorOffset ";
      code += "CreateSortedVectorOf" + struct_def.name;
      code += "(FlatBufferBuilder builder, ";
      code += "Offset<" + struct_def.name + ">";
      code += "[] offsets) {\n";
      code += "    Array.Sort(offsets,\n";
      code += "      (Offset<" + struct_def.name + "> o1, Offset<" +
              struct_def.name + "> o2) =>\n";
      code += "        " + GenKeyGetter(struct_def, key_field);
      code += ");\n";
      code += "    return builder.CreateVectorOfTables(offsets);\n  }\n";

      code += "\n  public static " + struct_def.name + "?";
      code += " __lookup_by_key(";
      code += "int vectorLocation, ";
      code += GenTypeGet(key_field->value.type);
      code += " key, ByteBuffer bb) {\n";
      code +=
          "    " + struct_def.name + " obj_ = new " + struct_def.name + "();\n";
      code += "    int span = ";
      code += "bb.GetInt(vectorLocation - 4);\n";
      code += "    int start = 0;\n";
      code += "    while (span != 0) {\n";
      code += "      int middle = span / 2;\n";
      code +=
          "      int tableOffset = Table.__indirect(vectorLocation + 4 * "
          "(start + middle), bb);\n";

      code += "      obj_.__assign(tableOffset, bb);\n";
      code += "      int comp = obj_." + name + ".CompareTo(key);\n";
      code += "      if (comp > 0) {\n";
      code += "        span = middle;\n";
      code += "      } else if (comp < 0) {\n";
      code += "        middle++;\n";
      code += "        start += middle;\n";
      code += "        span -= middle;\n";
      code += "      } else {\n";
      code += "        return obj_;\n";
      code += "      }\n    }\n";
      code += "    return null;\n";
      code += "  }\n";
    }

    if (opts.generate_object_based_api) {
      GenPackUnPack_ObjectAPI(struct_def, code_ptr, opts, struct_has_create,
                              field_has_create_set);
    }
    code += "}\n\n";

    if (opts.generate_object_based_api) {
      GenStruct_ObjectAPI(struct_def, code_ptr, opts);
    }
  }

  void GenVectorAccessObject(StructDef &struct_def,
                             std::string *code_ptr) const {
    auto &code = *code_ptr;
    // Generate a vector of structs accessor class.
    code += "\n";
    code += "  ";
    if (!struct_def.attributes.Lookup("private")) code += "public ";
    code += "static struct Vector : BaseVector\n{\n";

    // Generate the __assign method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    std::string method_indent = "    ";
    code += method_indent + "public Vector ";
    code += "__assign(int _vector, int _element_size, ByteBuffer _bb) { ";
    code += "__reset(_vector, _element_size, _bb); return this; }\n\n";

    auto type_name = struct_def.name;
    auto method_start = method_indent + "public " + type_name + " Get";
    // Generate the accessors that don't do object reuse.
    code += method_start + "(int j) { return Get";
    code += "(new " + type_name + "(), j); }\n";
    code += method_start + "(" + type_name + " obj, int j) { ";
    code += " return obj.__assign(";
    code += struct_def.fixed ? "__p.__element(j)"
                             : "__p.__indirect(__p.__element(j), bb)";
    code += ", __p.bb); }\n";
    // See if we should generate a by-key accessor.
    if (!struct_def.fixed) {
      auto &fields = struct_def.fields.vec;
      for (auto kit = fields.begin(); kit != fields.end(); ++kit) {
        auto &key_field = **kit;
        if (key_field.key) {
          auto nullable_annotation =
              parser_.opts.gen_nullable ? "@Nullable " : "";
          code += method_indent + nullable_annotation;
          code += "public " + type_name + "? ";
          code += "GetByKey(";
          code += GenTypeGet(key_field.value.type) + " key) { ";
          code += " return __lookup_by_key(null, ";
          code += "__p.__vector(), key, ";
          code += "__p.bb); ";
          code += "}\n";
          code += method_indent + nullable_annotation;
          code += "public " + type_name + "?" + " ";
          code += "GetByKey(";
          code += type_name + "? obj, ";
          code += GenTypeGet(key_field.value.type) + " key) { ";
          code += " return __lookup_by_key(obj, ";
          code += "__p.__vector(), key, ";
          code += "__p.bb); ";
          code += "}\n";
          break;
        }
      }
    }
    code += "  }\n";
  }

  std::string GenUnionVerify(const Type &union_type) const {
    if (union_type.enum_def) {
      const auto &enum_def = *union_type.enum_def;

      auto ret = "\n\nstatic public class " + enum_def.name + "Verify\n";
      ret += "{\n";
      ret +=
          "  static public bool Verify(Google.FlatBuffers.Verifier verifier, "
          "byte typeId, uint tablePos)\n";
      ret += "  {\n";
      ret += "    bool result = true;\n";

      const auto union_enum_loop = [&]() {
        ret += "    switch((" + enum_def.name + ")typeId)\n";
        ret += "    {\n";

        for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
             ++it) {
          const auto &ev = **it;
          if (ev.IsZero()) { continue; }

          ret += "      case " + Name(enum_def) + "." + Name(ev) + ":\n";

          if (IsString(ev.union_type)) {
            ret += "       result = verifier.VerifyUnionString(tablePos);\n";
            ret += "        break;";
          } else if (ev.union_type.base_type == BASE_TYPE_STRUCT) {
            if (!ev.union_type.struct_def->fixed) {
              auto type = GenTypeGet(ev.union_type);
              ret += "        result = " + type +
                     "Verify.Verify(verifier, tablePos);\n";
            } else {
              ret += "        result = verifier.VerifyUnionData(tablePos, " +
                     NumToString(InlineSize(ev.union_type)) + ", " +
                     NumToString(InlineAlignment(ev.union_type)) + ");\n";
              ;
            }
            ret += "        break;";
          } else {
            FLATBUFFERS_ASSERT(false);
          }
          ret += "\n";
        }

        ret += "      default: result = true;\n";
        ret += "        break;\n";
        ret += "    }\n";
        ret += "    return result;\n";
      };

      union_enum_loop();
      ret += "  }\n";
      ret += "}\n";
      ret += "\n";

      return ret;
    }
    FLATBUFFERS_ASSERT(0);
    return "";
  }

  void GenEnum_ObjectAPI(EnumDef &enum_def, std::string *code_ptr,
                         const IDLOptions &opts) const {
    auto &code = *code_ptr;
    if (enum_def.generated) return;
    if (!enum_def.is_union) return;
    if (enum_def.attributes.Lookup("private")) {
      code += "internal ";
    } else {
      code += "public ";
    }
    auto union_name = enum_def.name + "Union";
    auto class_member = std::string("Value");
    if (class_member == enum_def.name) { class_member += "_"; };
    code += "class " + union_name + " {\n";
    // Type
    code += "  public " + enum_def.name + " Type { get; set; }\n";
    // Value
    code += "  public object " + class_member + " { get; set; }\n";
    code += "\n";
    // Constructor
    code += "  public " + union_name + "() {\n";
    code += "    this.Type = " + enum_def.name + "." +
            enum_def.Vals()[0]->name + ";\n";
    code += "    this." + class_member + " = null;\n";
    code += "  }\n\n";
    // As<T>
    code += "  public T As<T>() where T : class { return this." + class_member +
            " as T; }\n";
    // As, From
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      if (ev.union_type.base_type == BASE_TYPE_NONE) continue;
      auto type_name = GenTypeGet_ObjectAPI(ev.union_type, opts);
      std::string accessibility =
          (ev.union_type.base_type == BASE_TYPE_STRUCT &&
           ev.union_type.struct_def->attributes.Lookup("private"))
              ? "internal"
              : "public";
      // As
      code += "  " + accessibility + " " + type_name + " As" + ev.name +
              "() { return this.As<" + type_name + ">(); }\n";
      // From
      auto lower_ev_name = ev.name;
      std::transform(lower_ev_name.begin(), lower_ev_name.end(),
                     lower_ev_name.begin(), CharToLower);
      code += "  " + accessibility + " static " + union_name + " From" +
              ev.name + "(" + type_name + " _" + lower_ev_name +
              ") { return new " + union_name + "{ Type = " + Name(enum_def) +
              "." + Name(ev) + ", " + class_member + " = _" + lower_ev_name +
              " }; }\n";
    }
    code += "\n";
    // Pack()
    code +=
        "  public static int Pack(Google.FlatBuffers.FlatBufferBuilder "
        "builder, " +
        union_name + " _o) {\n";
    code += "    switch (_o.Type) {\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      if (ev.union_type.base_type == BASE_TYPE_NONE) {
        code += "      default: return 0;\n";
      } else {
        code += "      case " + Name(enum_def) + "." + Name(ev) + ": return ";
        if (IsString(ev.union_type)) {
          code += "builder.CreateString(_o.As" + ev.name + "()).Value;\n";
        } else {
          code += GenTypeGet(ev.union_type) + ".Pack(builder, _o.As" + ev.name +
                  "()).Value;\n";
        }
      }
    }
    code += "    }\n";
    code += "  }\n";
    code += "}\n\n";

    // JsonConverter
    if (opts.cs_gen_json_serializer) {
      if (enum_def.attributes.Lookup("private")) {
        code += "internal ";
      } else {
        code += "public ";
      }
      code += "class " + union_name +
              "_JsonConverter : Newtonsoft.Json.JsonConverter {\n";
      code += "  public override bool CanConvert(System.Type objectType) {\n";
      code += "    return objectType == typeof(" + union_name +
              ") || objectType == typeof(System.Collections.Generic.List<" +
              union_name + ">);\n";
      code += "  }\n";
      code +=
          "  public override void WriteJson(Newtonsoft.Json.JsonWriter writer, "
          "object value, "
          "Newtonsoft.Json.JsonSerializer serializer) {\n";
      code += "    var _olist = value as System.Collections.Generic.List<" +
              union_name + ">;\n";
      code += "    if (_olist != null) {\n";
      code += "      writer.WriteStartArray();\n";
      code +=
          "      foreach (var _o in _olist) { this.WriteJson(writer, _o, "
          "serializer); }\n";
      code += "      writer.WriteEndArray();\n";
      code += "    } else {\n";
      code += "      this.WriteJson(writer, value as " + union_name +
              ", serializer);\n";
      code += "    }\n";
      code += "  }\n";
      code += "  public void WriteJson(Newtonsoft.Json.JsonWriter writer, " +
              union_name +
              " _o, "
              "Newtonsoft.Json.JsonSerializer serializer) {\n";
      code += "    if (_o == null) return;\n";
      code += "    serializer.Serialize(writer, _o." + class_member + ");\n";
      code += "  }\n";
      code +=
          "  public override object ReadJson(Newtonsoft.Json.JsonReader "
          "reader, "
          "System.Type objectType, "
          "object existingValue, Newtonsoft.Json.JsonSerializer serializer) "
          "{\n";
      code +=
          "    var _olist = existingValue as System.Collections.Generic.List<" +
          union_name + ">;\n";
      code += "    if (_olist != null) {\n";
      code += "      for (var _j = 0; _j < _olist.Count; ++_j) {\n";
      code += "        reader.Read();\n";
      code +=
          "        _olist[_j] = this.ReadJson(reader, _olist[_j], "
          "serializer);\n";
      code += "      }\n";
      code += "      reader.Read();\n";
      code += "      return _olist;\n";
      code += "    } else {\n";
      code += "      return this.ReadJson(reader, existingValue as " +
              union_name + ", serializer);\n";
      code += "    }\n";
      code += "  }\n";
      code += "  public " + union_name +
              " ReadJson(Newtonsoft.Json.JsonReader reader, " + union_name +
              " _o, Newtonsoft.Json.JsonSerializer serializer) {\n";
      code += "    if (_o == null) return null;\n";
      code += "    switch (_o.Type) {\n";
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        auto &ev = **it;
        if (ev.union_type.base_type == BASE_TYPE_NONE) {
          code += "      default: break;\n";
        } else {
          auto type_name = GenTypeGet_ObjectAPI(ev.union_type, opts);
          code += "      case " + Name(enum_def) + "." + Name(ev) + ": _o." +
                  class_member + " = serializer.Deserialize<" + type_name +
                  ">(reader); break;\n";
        }
      }
      code += "    }\n";
      code += "    return _o;\n";
      code += "  }\n";
      code += "}\n\n";
    }
  }

  std::string GenTypeName_ObjectAPI(const std::string &name,
                                    const IDLOptions &opts) const {
    return opts.object_prefix + name + opts.object_suffix;
  }

  void GenUnionUnPack_ObjectAPI(const EnumDef &enum_def, std::string *code_ptr,
                                const std::string &camel_name,
                                const std::string &camel_name_short,
                                bool is_vector) const {
    auto &code = *code_ptr;
    std::string varialbe_name = "_o." + camel_name;
    std::string class_member = "Value";
    if (class_member == enum_def.name) class_member += "_";
    std::string type_suffix = "";
    std::string func_suffix = "()";
    std::string indent = "    ";
    if (is_vector) {
      varialbe_name = "_o_" + camel_name;
      type_suffix = "(_j)";
      func_suffix = "(_j)";
      indent = "      ";
    }
    if (is_vector) {
      code += indent + "var " + varialbe_name + " = new ";
    } else {
      code += indent + varialbe_name + " = new ";
    }
    code += NamespacedName(enum_def) + "Union();\n";
    code += indent + varialbe_name + ".Type = this." + camel_name_short +
            "Type" + type_suffix + ";\n";
    code += indent + "switch (this." + camel_name_short + "Type" + type_suffix +
            ") {\n";
    for (auto eit = enum_def.Vals().begin(); eit != enum_def.Vals().end();
         ++eit) {
      auto &ev = **eit;
      if (ev.union_type.base_type == BASE_TYPE_NONE) {
        code += indent + "  default: break;\n";
      } else {
        code += indent + "  case " + NamespacedName(enum_def) + "." + ev.name +
                ":\n";
        code += indent + "    " + varialbe_name + "." + class_member +
                " = this." + camel_name;
        if (IsString(ev.union_type)) {
          code += "AsString" + func_suffix + ";\n";
        } else {
          code += "<" + GenTypeGet(ev.union_type) + ">" + func_suffix;
          code += ".HasValue ? this." + camel_name;
          code += "<" + GenTypeGet(ev.union_type) + ">" + func_suffix +
                  ".Value.UnPack() : null;\n";
        }
        code += indent + "    break;\n";
      }
    }
    code += indent + "}\n";
    if (is_vector) {
      code += indent + "_o." + camel_name + ".Add(" + varialbe_name + ");\n";
    }
  }

  void GenPackUnPack_ObjectAPI(
      StructDef &struct_def, std::string *code_ptr, const IDLOptions &opts,
      bool struct_has_create,
      const std::set<FieldDef *> &field_has_create) const {
    auto &code = *code_ptr;
    auto struct_name = GenTypeName_ObjectAPI(struct_def.name, opts);
    // UnPack()
    code += "  public " + struct_name + " UnPack() {\n";
    code += "    var _o = new " + struct_name + "();\n";
    code += "    this.UnPackTo(_o);\n";
    code += "    return _o;\n";
    code += "  }\n";
    // UnPackTo()
    code += "  public void UnPackTo(" + struct_name + " _o) {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      auto camel_name = Name(field);
      if (camel_name == struct_def.name) { camel_name += "_"; }
      auto camel_name_short = Name(field);
      auto start = "    _o." + camel_name + " = ";
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          auto fixed = struct_def.fixed && field.value.type.struct_def->fixed;
          if (fixed) {
            code += start + "this." + camel_name + ".UnPack();\n";
          } else {
            code += start + "this." + camel_name + ".HasValue ? this." +
                    camel_name + ".Value.UnPack() : null;\n";
          }
          break;
        }
        case BASE_TYPE_ARRAY: {
          auto type_name = GenTypeGet_ObjectAPI(field.value.type, opts);
          auto length_str = NumToString(field.value.type.fixed_length);
          auto unpack_method = field.value.type.struct_def == nullptr ? ""
                               : field.value.type.struct_def->fixed
                                   ? ".UnPack()"
                                   : "?.UnPack()";
          code += start + "new " + type_name.substr(0, type_name.length() - 1) +
                  length_str + "];\n";
          code += "    for (var _j = 0; _j < " + length_str + "; ++_j) { _o." +
                  camel_name + "[_j] = this." + camel_name + "(_j)" +
                  unpack_method + "; }\n";
          break;
        }
        case BASE_TYPE_VECTOR:
          if (field.value.type.element == BASE_TYPE_UNION) {
            code += start + "new " +
                    GenTypeGet_ObjectAPI(field.value.type, opts) + "();\n";
            code += "    for (var _j = 0; _j < this." + camel_name +
                    "Length; ++_j) {\n";
            GenUnionUnPack_ObjectAPI(*field.value.type.enum_def, code_ptr,
                                     camel_name, camel_name_short, true);
            code += "    }\n";
          } else if (field.value.type.element != BASE_TYPE_UTYPE) {
            auto fixed = field.value.type.struct_def == nullptr;
            code += start + "new " +
                    GenTypeGet_ObjectAPI(field.value.type, opts) + "();\n";
            code += "    for (var _j = 0; _j < this." + camel_name +
                    "Length; ++_j) {";
            code += "_o." + camel_name + ".Add(";
            if (fixed) {
              code += "this." + camel_name + "(_j)";
            } else {
              code += "this." + camel_name + "(_j).HasValue ? this." +
                      camel_name + "(_j).Value.UnPack() : null";
            }
            code += ");}\n";
          }
          break;
        case BASE_TYPE_UTYPE: break;
        case BASE_TYPE_UNION: {
          GenUnionUnPack_ObjectAPI(*field.value.type.enum_def, code_ptr,
                                   camel_name, camel_name_short, false);
          break;
        }
        default: {
          code += start + "this." + camel_name + ";\n";
          break;
        }
      }
    }
    code += "  }\n";
    // Pack()
    code += "  public static " + GenOffsetType(struct_def) +
            " Pack(FlatBufferBuilder builder, " + struct_name + " _o) {\n";
    code += "    if (_o == null) return default(" + GenOffsetType(struct_def) +
            ");\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      auto camel_name = Name(field);
      if (camel_name == struct_def.name) { camel_name += "_"; }
      auto camel_name_short = Name(field);
      // pre
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          if (!field.value.type.struct_def->fixed) {
            code += "    var _" + field.name + " = _o." + camel_name +
                    " == null ? default(" +
                    GenOffsetType(*field.value.type.struct_def) +
                    ") : " + GenTypeGet(field.value.type) +
                    ".Pack(builder, _o." + camel_name + ");\n";
          } else if (struct_def.fixed && struct_has_create) {
            std::vector<FieldArrayLength> array_lengths;
            FieldArrayLength tmp_array_length = {
              field.name,
              field.value.type.fixed_length,
            };
            array_lengths.push_back(tmp_array_length);
            GenStructPackDecl_ObjectAPI(*field.value.type.struct_def, code_ptr,
                                        array_lengths);
          }
          break;
        }
        case BASE_TYPE_STRING: {
          std::string create_string =
              field.shared ? "CreateSharedString" : "CreateString";
          code += "    var _" + field.name + " = _o." + camel_name +
                  " == null ? default(StringOffset) : "
                  "builder." +
                  create_string + "(_o." + camel_name + ");\n";
          break;
        }
        case BASE_TYPE_VECTOR: {
          if (field_has_create.find(&field) != field_has_create.end()) {
            auto property_name = camel_name;
            auto gen_for_loop = true;
            std::string array_name = "__" + field.name;
            std::string array_type = "";
            std::string to_array = "";
            switch (field.value.type.element) {
              case BASE_TYPE_STRING: {
                std::string create_string =
                    field.shared ? "CreateSharedString" : "CreateString";
                array_type = "StringOffset";
                to_array += "builder." + create_string + "(_o." +
                            property_name + "[_j])";
                break;
              }
              case BASE_TYPE_STRUCT:
                array_type = "Offset<" + GenTypeGet(field.value.type) + ">";
                to_array = GenTypeGet(field.value.type) + ".Pack(builder, _o." +
                           property_name + "[_j])";
                break;
              case BASE_TYPE_UTYPE:
                property_name = camel_name.substr(0, camel_name.size() - 4);
                array_type = NamespacedName(*field.value.type.enum_def);
                to_array = "_o." + property_name + "[_j].Type";
                break;
              case BASE_TYPE_UNION:
                array_type = "int";
                to_array = NamespacedName(*field.value.type.enum_def) +
                           "Union.Pack(builder,  _o." + property_name + "[_j])";
                break;
              default: gen_for_loop = false; break;
            }
            code += "    var _" + field.name + " = default(VectorOffset);\n";
            code += "    if (_o." + property_name + " != null) {\n";
            if (gen_for_loop) {
              code += "      var " + array_name + " = new " + array_type +
                      "[_o." + property_name + ".Count];\n";
              code += "      for (var _j = 0; _j < " + array_name +
                      ".Length; ++_j) { ";
              code += array_name + "[_j] = " + to_array + "; }\n";
            } else {
              code += "      var " + array_name + " = _o." + property_name +
                      ".ToArray();\n";
            }
            code += "      _" + field.name + " = Create" + camel_name_short +
                    "Vector(builder, " + array_name + ");\n";
            code += "    }\n";
          } else {
            auto pack_method =
                field.value.type.struct_def == nullptr
                    ? "builder.Add" + GenMethod(field.value.type.VectorType()) +
                          "(_o." + camel_name + "[_j]);"
                    : GenTypeGet(field.value.type) + ".Pack(builder, _o." +
                          camel_name + "[_j]);";
            code += "    var _" + field.name + " = default(VectorOffset);\n";
            code += "    if (_o." + camel_name + " != null) {\n";
            code += "      Start" + camel_name_short + "Vector(builder, _o." +
                    camel_name + ".Count);\n";
            code += "      for (var _j = _o." + camel_name +
                    ".Count - 1; _j >= 0; --_j) { " + pack_method + " }\n";
            code += "      _" + field.name + " = builder.EndVector();\n";
            code += "    }\n";
          }
          break;
        }
        case BASE_TYPE_ARRAY: {
          if (field.value.type.struct_def != nullptr) {
            std::vector<FieldArrayLength> array_lengths;
            FieldArrayLength tmp_array_length = {
              field.name,
              field.value.type.fixed_length,
            };
            array_lengths.push_back(tmp_array_length);
            GenStructPackDecl_ObjectAPI(*field.value.type.struct_def, code_ptr,
                                        array_lengths);
          } else {
            code += "    var _" + field.name + " = _o." + camel_name + ";\n";
          }
          break;
        }
        case BASE_TYPE_UNION: {
          code += "    var _" + field.name + "_type = _o." + camel_name +
                  " == null ? " + NamespacedName(*field.value.type.enum_def) +
                  ".NONE : " + "_o." + camel_name + ".Type;\n";
          code +=
              "    var _" + field.name + " = _o." + camel_name +
              " == null ? 0 : " + GenTypeGet_ObjectAPI(field.value.type, opts) +
              ".Pack(builder, _o." + camel_name + ");\n";
          break;
        }
        default: break;
      }
    }
    if (struct_has_create) {
      // Create
      code += "    return Create" + struct_def.name + "(\n";
      code += "      builder";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        auto camel_name = Name(field);
        if (camel_name == struct_def.name) { camel_name += "_"; }
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            if (struct_def.fixed) {
              GenStructPackCall_ObjectAPI(*field.value.type.struct_def,
                                          code_ptr,
                                          "      _" + field.name + "_");
            } else {
              code += ",\n";
              if (field.value.type.struct_def->fixed) {
                if (opts.generate_object_based_api)
                  code += "      _o." + camel_name;
                else
                  code += "      " + GenTypeGet(field.value.type) +
                          ".Pack(builder, _o." + camel_name + ")";
              } else {
                code += "      _" + field.name;
              }
            }
            break;
          }
          case BASE_TYPE_ARRAY: {
            if (field.value.type.struct_def != nullptr) {
              GenStructPackCall_ObjectAPI(*field.value.type.struct_def,
                                          code_ptr,
                                          "      _" + field.name + "_");
            } else {
              code += ",\n";
              code += "      _" + field.name;
            }
            break;
          }
          case BASE_TYPE_UNION: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_UTYPE: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_STRING: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_VECTOR: {
            code += ",\n";
            code += "      _" + field.name;
            break;
          }
          default:  // scalar
            code += ",\n";
            code += "      _o." + camel_name;
            break;
        }
      }
      code += ");\n";
    } else {
      // Start, End
      code += "    Start" + struct_def.name + "(builder);\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        auto camel_name = Name(field);
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            if (field.value.type.struct_def->fixed) {
              code += "    Add" + camel_name + "(builder, " +
                      GenTypeGet(field.value.type) + ".Pack(builder, _o." +
                      camel_name + "));\n";
            } else {
              code +=
                  "    Add" + camel_name + "(builder, _" + field.name + ");\n";
            }
            break;
          }
          case BASE_TYPE_STRING: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_VECTOR: {
            code +=
                "    Add" + camel_name + "(builder, _" + field.name + ");\n";
            break;
          }
          case BASE_TYPE_UTYPE: break;
          case BASE_TYPE_UNION: {
            code += "    Add" + camel_name + "Type(builder, _" + field.name +
                    "_type);\n";
            code +=
                "    Add" + camel_name + "(builder, _" + field.name + ");\n";
            break;
          }
          // scalar
          default: {
            code +=
                "    Add" + camel_name + "(builder, _o." + camel_name + ");\n";
            break;
          }
        }
      }
      code += "    return End" + struct_def.name + "(builder);\n";
    }
    code += "  }\n";
  }

  void GenStructPackDecl_ObjectAPI(
      const StructDef &struct_def, std::string *code_ptr,
      std::vector<FieldArrayLength> &array_lengths) const {
    auto &code = *code_ptr;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      auto is_array = IsArray(field.value.type);
      const auto &field_type =
          is_array ? field.value.type.VectorType() : field.value.type;
      FieldArrayLength tmp_array_length = {
        field.name,
        field_type.fixed_length,
      };
      array_lengths.push_back(tmp_array_length);
      if (field_type.struct_def != nullptr) {
        GenStructPackDecl_ObjectAPI(*field_type.struct_def, code_ptr,
                                    array_lengths);
      } else {
        std::vector<FieldArrayLength> array_only_lengths;
        for (size_t i = 0; i < array_lengths.size(); ++i) {
          if (array_lengths[i].length > 0) {
            array_only_lengths.push_back(array_lengths[i]);
          }
        }
        std::string name;
        for (size_t i = 0; i < array_lengths.size(); ++i) {
          name += "_" + array_lengths[i].name;
        }
        code += "    var " + name + " = ";
        if (array_only_lengths.size() > 0) {
          code += "new " + GenTypeBasic(field_type) + "[";
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            if (i != 0) { code += ","; }
            code += NumToString(array_only_lengths[i].length);
          }
          code += "];\n";
          code += "    ";
          // initialize array
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            auto idx = "idx" + NumToString(i);
            code += "for (var " + idx + " = 0; " + idx + " < " +
                    NumToString(array_only_lengths[i].length) + "; ++" + idx +
                    ") {";
          }
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            auto idx = "idx" + NumToString(i);
            if (i == 0) {
              code += name + "[" + idx;
            } else {
              code += "," + idx;
            }
          }
          code += "] = _o";
          for (size_t i = 0, j = 0; i < array_lengths.size(); ++i) {
            code += "." + ConvertCase(array_lengths[i].name, Case::kUpperCamel);
            if (array_lengths[i].length <= 0) continue;
            code += "[idx" + NumToString(j++) + "]";
          }
          code += ";";
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            code += "}";
          }
        } else {
          code += "_o";
          for (size_t i = 0; i < array_lengths.size(); ++i) {
            code += "." + ConvertCase(array_lengths[i].name, Case::kUpperCamel);
          }
          code += ";";
        }
        code += "\n";
      }
      array_lengths.pop_back();
    }
  }

  void GenStructPackCall_ObjectAPI(const StructDef &struct_def,
                                   std::string *code_ptr,
                                   std::string prefix) const {
    auto &code = *code_ptr;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      if (field_type.struct_def != nullptr) {
        GenStructPackCall_ObjectAPI(*field_type.struct_def, code_ptr,
                                    prefix + field.name + "_");
      } else {
        code += ",\n";
        code += prefix + field.name;
      }
    }
  }

  std::string GenTypeGet_ObjectAPI(flatbuffers::Type type,
                                   const IDLOptions &opts) const {
    auto type_name = GenTypeGet(type);
    // Replace to ObjectBaseAPI Type Name
    switch (type.base_type) {
      case BASE_TYPE_STRUCT: FLATBUFFERS_FALLTHROUGH();  // fall thru
      case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();   // fall thru
      case BASE_TYPE_VECTOR: {
        if (type.struct_def != nullptr) {
          auto type_name_length = type.struct_def->name.length();
          auto new_type_name =
              GenTypeName_ObjectAPI(type.struct_def->name, opts);
          type_name.replace(type_name.length() - type_name_length,
                            type_name_length, new_type_name);
        } else if (type.element == BASE_TYPE_UNION) {
          type_name = NamespacedName(*type.enum_def) + "Union";
        }
        break;
      }

      case BASE_TYPE_UNION: {
        type_name = NamespacedName(*type.enum_def) + "Union";
        break;
      }
      default: break;
    }

    switch (type.base_type) {
      case BASE_TYPE_ARRAY: {
        type_name = type_name + "[]";
        break;
      }
      case BASE_TYPE_VECTOR: {
        type_name = "List<" + type_name + ">";
        break;
      }
      default: break;
    }
    return type_name;
  }

  void GenStruct_ObjectAPI(StructDef &struct_def, std::string *code_ptr,
                           const IDLOptions &opts) const {
    auto &code = *code_ptr;
    if (struct_def.attributes.Lookup("private")) {
      code += "internal ";
    } else {
      code += "public ";
    }
    if (struct_def.attributes.Lookup("csharp_partial")) {
      // generate a partial class for this C# struct/table
      code += "partial ";
    }
    auto class_name = GenTypeName_ObjectAPI(struct_def.name, opts);
    code += "class " + class_name;
    code += "\n{\n";
    // Generate Properties
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      auto type_name = GenTypeGet_ObjectAPI(field.value.type, opts);
      if (field.IsScalarOptional()) type_name += "?";
      auto camel_name = Name(field);
      if (camel_name == struct_def.name) { camel_name += "_"; }
      if (opts.cs_gen_json_serializer) {
        if (IsUnion(field.value.type)) {
          auto utype_name = NamespacedName(*field.value.type.enum_def);
          code +=
              "  [Newtonsoft.Json.JsonProperty(\"" + field.name + "_type\")]\n";
          if (IsVector(field.value.type)) {
            code += "  private " + utype_name + "[] " + camel_name + "Type {\n";
            code += "    get {\n";
            code += "      if (this." + camel_name + " == null) return null;\n";
            code += "      var _o = new " + utype_name + "[this." + camel_name +
                    ".Count];\n";
            code +=
                "      for (var _j = 0; _j < _o.Length; ++_j) { _o[_j] = "
                "this." +
                camel_name + "[_j].Type; }\n";
            code += "      return _o;\n";
            code += "    }\n";
            code += "    set {\n";
            code += "      this." + camel_name + " = new List<" + utype_name +
                    "Union>();\n";
            code += "      for (var _j = 0; _j < value.Length; ++_j) {\n";
            code += "        var _o = new " + utype_name + "Union();\n";
            code += "        _o.Type = value[_j];\n";
            code += "        this." + camel_name + ".Add(_o);\n";
            code += "      }\n";
            code += "    }\n";
            code += "  }\n";
          } else {
            code += "  private " + utype_name + " " + camel_name + "Type {\n";
            code += "    get {\n";
            code += "      return this." + camel_name + " != null ? this." +
                    camel_name + ".Type : " + utype_name + ".NONE;\n";
            code += "    }\n";
            code += "    set {\n";
            code += "      this." + camel_name + " = new " + utype_name +
                    "Union();\n";
            code += "      this." + camel_name + ".Type = value;\n";
            code += "    }\n";
            code += "  }\n";
          }
        }
        code += "  [Newtonsoft.Json.JsonProperty(\"" + field.name + "\")]\n";
        if (IsUnion(field.value.type)) {
          auto union_name =
              (IsVector(field.value.type))
                  ? GenTypeGet_ObjectAPI(field.value.type.VectorType(), opts)
                  : type_name;
          code += "  [Newtonsoft.Json.JsonConverter(typeof(" + union_name +
                  "_JsonConverter))]\n";
        }
        if (field.attributes.Lookup("hash")) {
          code += "  [Newtonsoft.Json.JsonIgnore()]\n";
        }
      }
      code += "  public " + type_name + " " + camel_name + " { get; set; }\n";
    }
    // Generate Constructor
    code += "\n";
    code += "  public " + class_name + "() {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      auto camel_name = Name(field);
      if (camel_name == struct_def.name) { camel_name += "_"; }
      code += "    this." + camel_name + " = ";
      auto type_name = GenTypeGet_ObjectAPI(field.value.type, opts);
      if (IsScalar(field.value.type.base_type)) {
        code += GenDefaultValue(field) + ";\n";
      } else {
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            if (IsStruct(field.value.type)) {
              code += "new " + type_name + "();\n";
            } else {
              code += "null;\n";
            }
            break;
          }
          case BASE_TYPE_ARRAY: {
            code += "new " + type_name.substr(0, type_name.length() - 1) +
                    NumToString(field.value.type.fixed_length) + "];\n";
            break;
          }
          default: {
            code += "null;\n";
            break;
          }
        }
      }
    }
    code += "  }\n";
    // Generate Serialization
    if (opts.cs_gen_json_serializer &&
        parser_.root_struct_def_ == &struct_def) {
      code += "\n";
      code += "  public static " + class_name +
              " DeserializeFromJson(string jsonText) {\n";
      code += "    return Newtonsoft.Json.JsonConvert.DeserializeObject<" +
              class_name + ">(jsonText);\n";
      code += "  }\n";
      code += "  public string SerializeToJson() {\n";
      code +=
          "    return Newtonsoft.Json.JsonConvert.SerializeObject(this, "
          "Newtonsoft.Json.Formatting.Indented);\n";
      code += "  }\n";
    }
    if (parser_.root_struct_def_ == &struct_def) {
      code += "  public static " + class_name +
              " DeserializeFromBinary(byte[] fbBuffer) {\n";
      code += "    return " + struct_def.name + ".GetRootAs" + struct_def.name +
              "(new ByteBuffer(fbBuffer)).UnPack();\n";
      code += "  }\n";
      code += "  public byte[] SerializeToBinary() {\n";
      code += "    var fbb = new FlatBufferBuilder(0x10000);\n";
      code += "    " + struct_def.name + ".Finish" + struct_def.name +
              "Buffer(fbb, " + struct_def.name + ".Pack(fbb, this));\n";
      code += "    return fbb.DataBuffer.ToSizedArray();\n";
      code += "  }\n";
    }
    code += "}\n\n";
  }

  // This tracks the current namespace used to determine if a type need to be
  // prefixed by its namespace
  const Namespace *cur_name_space_;
};
}  // namespace csharp

static bool GenerateCSharp(const Parser &parser, const std::string &path,
                           const std::string &file_name) {
  csharp::CSharpGenerator generator(parser, path, file_name);
  return generator.generate();
}

namespace {

class CSharpCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateCSharp(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = JavaCSharpMakeRule(false, parser, path, filename);
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

  bool IsSchemaOnly() const override { return true; }

  bool SupportsBfbsGeneration() const override { return false; }

  bool SupportsRootFileGeneration() const override { return false; }

  IDLOptions::Language Language() const override { return IDLOptions::kCSharp; }

  std::string LanguageName() const override { return "CSharp"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewCSharpCodeGenerator() {
  return std::unique_ptr<CSharpCodeGenerator>(new CSharpCodeGenerator());
}

}  // namespace flatbuffers
