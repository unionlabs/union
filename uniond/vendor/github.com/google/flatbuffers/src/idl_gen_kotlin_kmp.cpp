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

#include <functional>
#include <unordered_set>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_gen_kotlin.h"
#include "idl_namer.h"

namespace flatbuffers {

namespace kotlin {

namespace {

typedef std::map<std::string, std::pair<std::string, std::string> > FbbParamMap;
static TypedFloatConstantGenerator KotlinFloatGen("Double.", "Float.", "NaN",
                                                  "POSITIVE_INFINITY",
                                                  "NEGATIVE_INFINITY");

static const CommentConfig comment_config = { "/**", " *", " */" };
static const std::string ident_pad = "    ";
static std::set<std::string> KotlinKeywords() {
  return { "package",  "as",     "typealias", "class",  "this",   "super",
           "val",      "var",    "fun",       "for",    "null",   "true",
           "false",    "is",     "in",        "throw",  "return", "break",
           "continue", "object", "if",        "try",    "else",   "while",
           "do",       "when",   "interface", "typeof", "Any",    "Character" };
}

static Namer::Config KotlinDefaultConfig() {
  return { /*types=*/Case::kKeep,
           /*constants=*/Case::kUpperCamel,
           /*methods=*/Case::kLowerCamel,
           /*functions=*/Case::kKeep,
           /*fields=*/Case::kLowerCamel,
           /*variables=*/Case::kLowerCamel,
           /*variants=*/Case::kUpperCamel,
           /*enum_variant_seperator=*/"",  // I.e. Concatenate.
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kLowerCamel,
           /*namespace_seperator=*/".",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"E",
           /*filenames=*/Case::kUpperCamel,
           /*directories=*/Case::kLowerCamel,
           /*output_path=*/"",
           /*filename_suffix=*/"",
           /*filename_extension=*/".kt" };
}
}  // namespace

class KotlinKMPGenerator : public BaseGenerator {
 public:
  KotlinKMPGenerator(const Parser &parser, const std::string &path,
                     const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "", ".", "kt"),
        namer_(WithFlagOptions(KotlinDefaultConfig(), parser.opts, path),
               KotlinKeywords()) {}

  KotlinKMPGenerator &operator=(const KotlinKMPGenerator &);
  bool generate() FLATBUFFERS_OVERRIDE {
    std::string one_file_code;

    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      CodeWriter enumWriter(ident_pad);
      auto &enum_def = **it;

      GenEnum(enum_def, enumWriter);
      enumWriter += "";
      GenEnumOffsetAlias(enum_def, enumWriter);

      if (parser_.opts.one_file) {
        one_file_code += enumWriter.ToString();
      } else {
        if (!SaveType(namer_.EscapeKeyword(enum_def.name),
                      *enum_def.defined_namespace, enumWriter.ToString(), true))
          return false;
      }
    }

    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      CodeWriter structWriter(ident_pad);
      auto &struct_def = **it;

      GenStruct(struct_def, structWriter, parser_.opts);
      structWriter += "";
      GenStructOffsetAlias(struct_def, structWriter);

      if (parser_.opts.one_file) {
        one_file_code += structWriter.ToString();
      } else {
        if (!SaveType(namer_.EscapeKeyword(struct_def.name),
                      *struct_def.defined_namespace, structWriter.ToString(),
                      true))
          return false;
      }
    }

    if (parser_.opts.one_file) {
      return SaveType(file_name_, *parser_.current_namespace_, one_file_code,
                      true);
    }
    return true;
  }

  std::string TypeInNameSpace(const Namespace *ns,
                              const std::string &name = "") const {
    auto qualified = namer_.Namespace(*ns);
    return qualified.empty() ? name : qualified + qualifying_separator_ + name;
  }

  std::string TypeInNameSpace(const Definition &def,
                              const std::string &suffix = "") const {
    return TypeInNameSpace(def.defined_namespace, def.name + suffix);
  }

  // Save out the generated code for a single class while adding
  // declaration boilerplate.
  bool SaveType(const std::string &defname, const Namespace &ns,
                const std::string &classcode, bool needs_includes) const {
    if (!classcode.length()) return true;

    std::string code =
        "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n";
    auto qualified = ns.GetFullyQualifiedName("");
    std::string namespace_name = namer_.Namespace(ns);
    if (!namespace_name.empty()) {
      code += "package " + namespace_name;
      code += "\n\n";
    }
    if (needs_includes) { code += "import com.google.flatbuffers.kotlin.*\n"; }
    code += "import kotlin.jvm.JvmInline\n";
    code += classcode;
    const std::string dirs =
        namer_.Directories(ns, SkipDir::None, Case::kUnknown);
    EnsureDirExists(dirs);
    const std::string filename =
        dirs + namer_.File(defname, /*skips=*/SkipFile::Suffix);
    return SaveFile(filename.c_str(), code, false);
  }

  static bool IsEnum(const Type &type) {
    return type.enum_def != nullptr && IsInteger(type.base_type);
  }

  std::string GenerateKotlinPrimiteArray(const Type &type) const {
    if (IsScalar(type.base_type) && !IsEnum(type)) { return GenType(type); }

    if (IsEnum(type) || type.base_type == BASE_TYPE_UTYPE) {
      return TypeInNameSpace(type.enum_def->defined_namespace,
                             namer_.Type(*type.enum_def));
    }
    switch (type.base_type) {
      case BASE_TYPE_STRUCT:
        return "Offset<" + TypeInNameSpace(*type.struct_def) + ">";
      case BASE_TYPE_UNION: return "UnionOffset";
      case BASE_TYPE_STRING: return "Offset<String>";
      case BASE_TYPE_UTYPE: return "Offset<UByte>";
      default: return "Offset<" + GenTypeBasic(type.element) + ">";
    }
  }

  std::string GenerateKotlinOffsetArray(const Type &type) const {
    if (IsScalar(type.base_type) && !IsEnum(type)) {
      return GenType(type) + "Array";
    }

    if (IsEnum(type) || type.base_type == BASE_TYPE_UTYPE) {
      return TypeInNameSpace(type.enum_def->defined_namespace,
                             namer_.Type(*type.enum_def) + "Array");
    }
    switch (type.base_type) {
      case BASE_TYPE_STRUCT:
        return TypeInNameSpace(*type.struct_def) + "OffsetArray";
      case BASE_TYPE_UNION: return "UnionOffsetArray";
      case BASE_TYPE_STRING: return "StringOffsetArray";
      case BASE_TYPE_UTYPE: return "UByteArray";
      default: return GenTypeBasic(type.element) + "OffsetArray";
    }
  }

  std::string GenTypeBasic(const BaseType &type) const {
    switch (type) {
      case BASE_TYPE_NONE:
      case BASE_TYPE_UTYPE: return "UByte";
      case BASE_TYPE_BOOL: return "Boolean";
      case BASE_TYPE_CHAR: return "Byte";
      case BASE_TYPE_UCHAR: return "UByte";
      case BASE_TYPE_SHORT: return "Short";
      case BASE_TYPE_USHORT: return "UShort";
      case BASE_TYPE_INT: return "Int";
      case BASE_TYPE_UINT: return "UInt";
      case BASE_TYPE_LONG: return "Long";
      case BASE_TYPE_ULONG: return "ULong";
      case BASE_TYPE_FLOAT: return "Float";
      case BASE_TYPE_DOUBLE: return "Double";
      case BASE_TYPE_STRING:
      case BASE_TYPE_STRUCT: return "Offset";
      case BASE_TYPE_UNION: return "UnionOffset";
      case BASE_TYPE_VECTOR:
      case BASE_TYPE_ARRAY: return "VectorOffset";
      // VECTOR64 not supported
      case BASE_TYPE_VECTOR64: FLATBUFFERS_ASSERT(0);
    }
    return "Int";
  }

  std::string GenType(const Type &type) const {
    auto base_type = GenTypeBasic(type.base_type);

    if (IsEnum(type) || type.base_type == BASE_TYPE_UTYPE) {
      return TypeInNameSpace(type.enum_def->defined_namespace,
                             namer_.Type(*type.enum_def));
    }
    switch (type.base_type) {
      case BASE_TYPE_ARRAY:
      case BASE_TYPE_VECTOR: {
        switch (type.element) {
          case BASE_TYPE_STRUCT:
            return base_type + "<" + TypeInNameSpace(*type.struct_def) + ">";
          case BASE_TYPE_UNION:
            return base_type + "<" + GenTypeBasic(type.element) + ">";
          case BASE_TYPE_STRING: return base_type + "<String>";
          case BASE_TYPE_UTYPE: return base_type + "<UByte>";
          default: return base_type + "<" + GenTypeBasic(type.element) + ">";
        }
      }
      case BASE_TYPE_STRUCT:
        return base_type + "<" + TypeInNameSpace(*type.struct_def) + ">";
      case BASE_TYPE_STRING: return base_type + "<String>";
      case BASE_TYPE_UNION: return base_type;
      default: return base_type;
    }
    // clang-format on
  }

  std::string GenTypePointer(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "String";
      case BASE_TYPE_VECTOR: return GenTypeGet(type.VectorType());
      case BASE_TYPE_STRUCT: return TypeInNameSpace(*type.struct_def);
      default: return "Table";
    }
  }

  // with the addition of optional scalar types,
  // we are adding the nullable '?' operator to return type of a field.
  std::string GetterReturnType(const FieldDef &field) const {
    auto base_type = field.value.type.base_type;

    auto r_type = GenTypeGet(field.value.type);
    if (field.IsScalarOptional() ||
        // string, structs and unions
        (base_type == BASE_TYPE_STRING || base_type == BASE_TYPE_STRUCT ||
         base_type == BASE_TYPE_UNION) ||
        // vector of anything not scalar
        (base_type == BASE_TYPE_VECTOR &&
         !IsScalar(field.value.type.VectorType().base_type))) {
      r_type += "?";
    }
    return r_type;
  }

  std::string GenTypeGet(const Type &type) const {
    return IsScalar(type.base_type) ? GenType(type) : GenTypePointer(type);
  }

  std::string GenEnumDefaultValue(const FieldDef &field) const {
    auto &value = field.value;
    FLATBUFFERS_ASSERT(value.type.enum_def);
    auto &enum_def = *value.type.enum_def;
    auto enum_val = enum_def.FindByValue(value.constant);
    return enum_val ? (TypeInNameSpace(enum_def) + "." + enum_val->name)
                    : value.constant;
  }

  // differently from GenDefaultValue, the default values are meant
  // to be inserted in the buffer as the object is building.
  std::string GenDefaultBufferValue(const FieldDef &field) const {
    auto &value = field.value;
    auto base_type = value.type.base_type;
    auto field_name = field.name;
    std::string suffix = IsScalar(base_type) ? LiteralSuffix(value.type) : "";
    if (field.IsScalarOptional()) { return "null"; }
    if (IsFloat(base_type)) {
      auto val = KotlinFloatGen.GenFloatConstant(field);
      if (base_type == BASE_TYPE_DOUBLE && val.back() == 'f') {
        val.pop_back();
      }
      return val;
    }

    if (base_type == BASE_TYPE_BOOL) {
      return value.constant == "0" ? "false" : "true";
    }

    if (IsEnum(field.value.type)) {
      return value.constant + suffix;
    } else if ((IsVector(field.value.type) &&
                field.value.type.element == BASE_TYPE_UTYPE) ||
               (IsVector(field.value.type) &&
                field.value.type.VectorType().base_type == BASE_TYPE_UNION)) {
      return value.constant;
    } else {
      return value.constant + suffix;
    }
  }

  std::string GenDefaultValue(const FieldDef &field) const {
    auto &value = field.value;
    auto base_type = value.type.base_type;
    auto field_name = field.name;
    std::string suffix = LiteralSuffix(value.type);
    if (field.IsScalarOptional()) { return "null"; }
    if (IsFloat(base_type)) {
      auto val = KotlinFloatGen.GenFloatConstant(field);
      if (base_type == BASE_TYPE_DOUBLE && val.back() == 'f') {
        val.pop_back();
      }
      return val;
    }

    if (base_type == BASE_TYPE_BOOL) {
      return value.constant == "0" ? "false" : "true";
    }

    if (IsEnum(field.value.type) ||
        (IsVector(field.value.type) && IsEnum(field.value.type.VectorType()))) {
      return WrapEnumValue(field.value.type, value.constant + suffix);
    }

    if (IsVector(field.value.type) &&
        (field.value.type.VectorType().base_type == BASE_TYPE_UNION ||
         field.value.type.VectorType().base_type == BASE_TYPE_STRUCT ||
         field.value.type.VectorType().base_type == BASE_TYPE_STRING)) {
      return "null";
    }
    if (IsVector(field.value.type)) {
      switch (field.value.type.element) {
        case BASE_TYPE_UTYPE:
          return namer_.Type(*field.value.type.enum_def) + "(" +
                 value.constant + suffix + ")";
        case BASE_TYPE_UNION:
        case BASE_TYPE_STRUCT:
        case BASE_TYPE_STRING: return "null";
        case BASE_TYPE_BOOL: return value.constant == "0" ? "false" : "true";
        case BASE_TYPE_FLOAT: return value.constant + "f";
        case BASE_TYPE_DOUBLE: {
          return value.constant + ".toDouble()";
        }
        default: return value.constant + suffix;
      }
    }
    return value.constant + suffix;
  }

  void GenEnum(EnumDef &enum_def, CodeWriter &writer) const {
    if (enum_def.generated) return;

    GenerateComment(enum_def.doc_comment, writer, &comment_config);
    auto enum_type = namer_.Type(enum_def);
    auto field_type = GenTypeBasic(enum_def.underlying_type.base_type);
    writer += "@Suppress(\"unused\")";
    writer += "@JvmInline";
    writer += "value class " + enum_type + " (val value: " + field_type + ") {";
    writer.IncrementIdentLevel();

    GenerateCompanionObject(writer, [&]() {
      // Write all properties
      auto vals = enum_def.Vals();

      for (auto it = vals.begin(); it != vals.end(); ++it) {
        auto &ev = **it;
        auto val = enum_def.ToString(ev);
        auto suffix = LiteralSuffix(enum_def.underlying_type);
        writer.SetValue("name", namer_.Variant(ev));
        writer.SetValue("type", enum_type);
        writer.SetValue("val", val + suffix);
        GenerateComment(ev.doc_comment, writer, &comment_config);
        writer += "val {{name}} = {{type}}({{val}})";
      }

      // Generate a generate string table for enum values.
      // Problem is, if values are very sparse that could generate really
      // big tables. Ideally in that case we generate a map lookup
      // instead, but for the moment we simply don't output a table at all.
      auto range = enum_def.Distance();
      // Average distance between values above which we consider a table
      // "too sparse". Change at will.
      static const uint64_t kMaxSparseness = 5;
      if (range / static_cast<uint64_t>(enum_def.size()) < kMaxSparseness) {
        GeneratePropertyOneLine(writer, "names", "Array<String>", [&]() {
          writer += "arrayOf(\\";
          auto val = enum_def.Vals().front();
          for (auto it = vals.begin(); it != vals.end(); ++it) {
            auto ev = *it;
            for (auto k = enum_def.Distance(val, ev); k > 1; --k)
              writer += "\"\", \\";
            val = ev;
            writer += "\"" + (*it)->name + "\"\\";
            if (it + 1 != vals.end()) { writer += ", \\"; }
          }
          writer += ")";
        });
        std::string e_param = "e: " + enum_type;
        GenerateFunOneLine(
            writer, "name", e_param, "String",
            [&]() {
              writer += "names[e.value.toInt()\\";
              if (enum_def.MinValue()->IsNonZero())
                writer += " - " + namer_.Variant(*enum_def.MinValue()) +
                          ".value.toInt()\\";
              writer += "]";
            },
            parser_.opts.gen_jvmstatic);
      }
    });
    writer.DecrementIdentLevel();
    writer += "}";
  }

  // Returns the function name that is able to read a value of the given type.
  std::string ByteBufferGetter(const Type &type,
                               std::string bb_var_name) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "string";
      case BASE_TYPE_STRUCT: return "__struct";
      case BASE_TYPE_UNION: return "union";
      case BASE_TYPE_VECTOR:
        return ByteBufferGetter(type.VectorType(), bb_var_name);
      case BASE_TYPE_INT: return bb_var_name + ".getInt";
      case BASE_TYPE_UINT: return bb_var_name + ".getUInt";
      case BASE_TYPE_SHORT: return bb_var_name + ".getShort";
      case BASE_TYPE_USHORT: return bb_var_name + ".getUShort";
      case BASE_TYPE_ULONG: return bb_var_name + ".getULong";
      case BASE_TYPE_LONG: return bb_var_name + ".getLong";
      case BASE_TYPE_FLOAT: return bb_var_name + ".getFloat";
      case BASE_TYPE_DOUBLE: return bb_var_name + ".getDouble";
      case BASE_TYPE_UTYPE:
      case BASE_TYPE_UCHAR: return bb_var_name + ".getUByte";
      case BASE_TYPE_CHAR:
      case BASE_TYPE_NONE: return bb_var_name + ".get";
      case BASE_TYPE_BOOL: return "0.toByte() != " + bb_var_name + ".get";
      default: return bb_var_name + "." + namer_.Method("get", GenType(type));
    }
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenLookupByKey(flatbuffers::FieldDef *key_field,
                             const std::string &bb_var_name,
                             const char *num = nullptr) const {
    auto type = key_field->value.type;
    return ByteBufferGetter(type, bb_var_name) + "(" +
           GenOffsetGetter(key_field, num) + ")";
  }

  // Returns the method name for use with add/put calls.
  static std::string GenMethod(const Type &type) {
    return IsStruct(type) ? "Struct" : "";
  }

  // Recursively generate arguments for a constructor, to deal with nested
  // structs.
  void GenStructArgs(const StructDef &struct_def, CodeWriter &writer,
                     const char *nameprefix) const {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (IsStruct(field.value.type)) {
        // Generate arguments for a struct inside a struct. To ensure
        // names don't clash, and to make it obvious these arguments are
        // constructing a nested struct, prefix the name with the field
        // name.
        GenStructArgs(*field.value.type.struct_def, writer,
                      (nameprefix + (field.name + "_")).c_str());
      } else {
        writer += std::string(", ") + nameprefix + "\\";
        writer += namer_.Field(field) + ": \\";
        writer += GenType(field.value.type) + "\\";
      }
    }
  }

  // Recusively generate struct construction statements of the form:
  // builder.putType(name);
  // and insert manual padding.
  void GenStructBody(const StructDef &struct_def, CodeWriter &writer,
                     const char *nameprefix) const {
    writer.SetValue("align", NumToString(struct_def.minalign));
    writer.SetValue("size", NumToString(struct_def.bytesize));
    writer += "builder.prep({{align}}, {{size}})";
    auto fields_vec = struct_def.fields.vec;
    for (auto it = fields_vec.rbegin(); it != fields_vec.rend(); ++it) {
      auto &field = **it;

      if (field.padding) {
        writer.SetValue("pad", NumToString(field.padding));
        writer += "builder.pad({{pad}})";
      }
      if (IsStruct(field.value.type)) {
        GenStructBody(*field.value.type.struct_def, writer,
                      (nameprefix + (field.name + "_")).c_str());
      } else {
        auto suffix = IsEnum(field.value.type) ? ".value" : "";
        writer.SetValue("type", GenMethod(field.value.type));
        writer.SetValue("argname",
                        nameprefix + namer_.Variable(field) + suffix);
        writer += "builder.put{{type}}({{argname}})";
      }
    }
  }

  std::string GenOffsetGetter(flatbuffers::FieldDef *key_field,
                              const char *num = nullptr) const {
    std::string key_offset =
        "offset(" + NumToString(key_field->value.offset) + ", ";
    if (num) {
      key_offset += num;
      key_offset += ", buffer)";
    } else {
      key_offset += "(bb.capacity - tableOffset).toOffset<Int>(), bb)";
    }
    return key_offset;
  }

  bool StructHasUnsignedField(StructDef &struct_def) {
    auto vec = struct_def.fields.vec;
    for (auto it = vec.begin(); it != vec.end(); ++it) {
      auto &field = **it;
      if (IsUnsigned(field.value.type.base_type)) { return true; }
    }
    return false;
  }

  // This method generate alias types for offset arrays. We need it
  // to avoid unboxing/boxing of offsets when put into an array.
  // e.g:
  // Array<Offset<Monster>> generates boxing.
  // So we creates a new type to avoid it:
  // typealias MonterOffsetArray = IntArray
  void GenStructOffsetAlias(StructDef &struct_def, CodeWriter &writer) const {
    if (struct_def.generated) return;
    auto name = namer_.Type(struct_def);
    // This assumes offset as Ints always.
    writer += "typealias " + name + "OffsetArray = OffsetArray<" + name + ">";

    // public inline fun <T> MonsterOffsetArray(size: Int, crossinline call:
    // (Int) -> Offset<T>): OffsetArray<T> {
    //  return OffsetArray(IntArray(size) { call(it).value })
    // }
    writer += "";
    writer += "inline fun " + name +
              "OffsetArray(size: Int, crossinline call: (Int) -> Offset<" +
              name + ">): " + name + "OffsetArray =";
    writer.IncrementIdentLevel();
    writer += name + "OffsetArray(IntArray(size) { call(it).value })";
  }

  // This method generate alias types for offset arrays. We need it
  // to avoid unboxing/boxing of offsets when put into an array.
  // e.g:
  // Array<Offset<Monster>> generates boxing.
  // So we creates a new type to avoid it:
  // typealias MonterOffsetArray = IntArray
  void GenEnumOffsetAlias(EnumDef &enum_def, CodeWriter &writer) const {
    if (enum_def.generated) return;
    // This assumes offset as Ints always.
    writer += "typealias " + namer_.Type(enum_def) +
              "Array = " + GenTypeBasic(enum_def.underlying_type.base_type) +
              "Array";
  }

  void GenStruct(StructDef &struct_def, CodeWriter &writer,
                 IDLOptions options) const {
    if (struct_def.generated) return;

    GenerateComment(struct_def.doc_comment, writer, &comment_config);
    auto fixed = struct_def.fixed;

    writer.SetValue("struct_name", namer_.Type(struct_def));
    writer.SetValue("superclass", fixed ? "Struct" : "Table");

    writer += "@Suppress(\"unused\")";
    writer += "class {{struct_name}} : {{superclass}}() {\n";

    writer.IncrementIdentLevel();

    {
      auto esc_type = namer_.EscapeKeyword(struct_def.name);
      // Generate the init() method that sets the field in a pre-existing
      // accessor object. This is to allow object reuse.
      GenerateFunOneLine(writer, "init", "i: Int, buffer: ReadWriteBuffer",
                         esc_type, [&]() { writer += "reset(i, buffer)"; });
      writer += "";  // line break

      // Generate all getters
      GenerateStructGetters(struct_def, writer);

      // Generate Static Fields
      GenerateCompanionObject(writer, [&]() {
        if (!struct_def.fixed) {
          FieldDef *key_field = nullptr;

          // Generate version check method.
          // Force compile time error if not using the same version
          // runtime.
          GenerateFunOneLine(
              writer, "validateVersion", "", "",
              [&]() { writer += "VERSION_2_0_8"; }, options.gen_jvmstatic);

          writer += "";
          GenerateGetRootAsAccessors(namer_.Type(struct_def), writer, options);

          writer += "";
          GenerateBufferHasIdentifier(struct_def, writer, options);

          writer += "";
          GenerateTableCreator(struct_def, writer, options);

          GenerateStartStructMethod(struct_def, writer, options);

          // Static Add for fields
          auto fields = struct_def.fields.vec;
          int field_pos = -1;
          for (auto it = fields.begin(); it != fields.end(); ++it) {
            auto &field = **it;
            field_pos++;
            if (field.deprecated) continue;
            if (field.key) key_field = &field;
            writer += "";
            GenerateAddField(NumToString(field_pos), field, writer, options);
            if (IsVector(field.value.type)) {
              auto vector_type = field.value.type.VectorType();
              if (!IsStruct(vector_type)) {
                writer += "";
                GenerateCreateVectorField(field, writer, options);
              }
              writer += "";
              GenerateStartVectorField(field, writer, options);
            }
          }

          writer += "";
          GenerateEndStructMethod(struct_def, writer, options);
          auto file_identifier = parser_.file_identifier_;
          if (parser_.root_struct_def_ == &struct_def) {
            writer += "";
            GenerateFinishStructBuffer(struct_def, file_identifier, writer,
                                       options);
            writer += "";
            GenerateFinishSizePrefixed(struct_def, file_identifier, writer,
                                       options);
          }

          if (struct_def.has_key) {
            writer += "";
            GenerateLookupByKey(key_field, struct_def, writer, options);
          }
        } else {
          writer += "";
          GenerateStaticConstructor(struct_def, writer, options);
        }
      });
    }

    // class closing
    writer.DecrementIdentLevel();
    writer += "}";
  }

  // TODO: move key_field to reference instead of pointer
  void GenerateLookupByKey(FieldDef *key_field, StructDef &struct_def,
                           CodeWriter &writer, const IDLOptions options) const {
    std::stringstream params;
    params << "obj: " << namer_.Type(struct_def) << "?"
           << ", ";
    params << "vectorLocation: Int, ";
    params << "key: " << GenTypeGet(key_field->value.type) << ", ";
    params << "bb: ReadWriteBuffer";

    auto statements = [&]() {
      auto base_type = key_field->value.type.base_type;
      writer.SetValue("struct_name", namer_.Type(struct_def));
      if (base_type == BASE_TYPE_STRING) {
        writer += "val byteKey = key.encodeToByteArray()";
      }
      writer += "var span = bb.getInt(vectorLocation - 4)";
      writer += "var start = 0";
      writer += "while (span != 0) {";
      writer.IncrementIdentLevel();
      writer += "var middle = span / 2";
      writer +=
          "val tableOffset = indirect(vector"
          "Location + 4 * (start + middle), bb)";
      if (IsString(key_field->value.type)) {
        writer += "val comp = compareStrings(\\";
        writer += GenOffsetGetter(key_field) + "\\";
        writer += ", byteKey, bb)";
      } else {
        auto get_val = GenLookupByKey(key_field, "bb");
        writer += "val value = " + get_val;
        writer += "val comp = value.compareTo(key)";
      }
      writer += "when {";
      writer.IncrementIdentLevel();
      writer += "comp > 0 -> span = middle";
      writer += "comp < 0 -> {";
      writer.IncrementIdentLevel();
      writer += "middle++";
      writer += "start += middle";
      writer += "span -= middle";
      writer.DecrementIdentLevel();
      writer += "}";  // end comp < 0
      writer += "else -> {";
      writer.IncrementIdentLevel();
      writer += "return (obj ?: {{struct_name}}()).init(tableOffset, bb)";
      writer.DecrementIdentLevel();
      writer += "}";  // end else
      writer.DecrementIdentLevel();
      writer += "}";  // end when
      writer.DecrementIdentLevel();
      writer += "}";  // end while
      writer += "return null";
    };
    GenerateFun(writer, "lookupByKey", params.str(),
                namer_.Type(struct_def) + "?", statements,
                options.gen_jvmstatic);
  }

  void GenerateFinishSizePrefixed(StructDef &struct_def,
                                  const std::string &identifier,
                                  CodeWriter &writer,
                                  const IDLOptions options) const {
    auto id = identifier.length() > 0 ? ", \"" + identifier + "\"" : "";
    auto gen_type = "Offset<" + namer_.Type(struct_def.name) + ">";
    auto params = "builder: FlatBufferBuilder, offset: " + gen_type;
    auto method_name =
        namer_.LegacyJavaMethod2("finishSizePrefixed", struct_def, "Buffer");
    GenerateFunOneLine(
        writer, method_name, params, "",
        [&]() { writer += "builder.finishSizePrefixed(offset" + id + ")"; },
        options.gen_jvmstatic);
  }
  void GenerateFinishStructBuffer(StructDef &struct_def,
                                  const std::string &identifier,
                                  CodeWriter &writer,
                                  const IDLOptions options) const {
    auto id = identifier.length() > 0 ? ", \"" + identifier + "\"" : "";
    auto gen_type = "Offset<" + namer_.Type(struct_def.name) + ">";
    auto params = "builder: FlatBufferBuilder, offset: " + gen_type;
    auto method_name =
        namer_.LegacyKotlinMethod("finish", struct_def, "Buffer");
    GenerateFunOneLine(
        writer, method_name, params, "",
        [&]() { writer += "builder.finish(offset" + id + ")"; },
        options.gen_jvmstatic);
  }

  void GenerateEndStructMethod(StructDef &struct_def, CodeWriter &writer,
                               const IDLOptions options) const {
    // Generate end{{TableName}}(builder: FlatBufferBuilder) method
    auto name = namer_.Method("end", struct_def.name);
    auto params = "builder: FlatBufferBuilder";
    auto returns = "Offset<" + namer_.Type(struct_def) + '>';
    auto field_vec = struct_def.fields.vec;

    GenerateFun(
        writer, name, params, returns,
        [&]() {
          writer += "val o: " + returns + " = builder.endTable()";
          writer.IncrementIdentLevel();
          for (auto it = field_vec.begin(); it != field_vec.end(); ++it) {
            auto &field = **it;
            if (field.deprecated || !field.IsRequired()) { continue; }
            writer.SetValue("offset", NumToString(field.value.offset));
            writer.SetValue("field_name", field.name);
            writer += "builder.required(o, {{offset}}, \"{{field_name}}\")";
          }
          writer.DecrementIdentLevel();
          writer += "return o";
        },
        options.gen_jvmstatic);
  }

  // Generate a method to create a vector from a Kotlin array.
  void GenerateCreateVectorField(FieldDef &field, CodeWriter &writer,
                                 const IDLOptions options) const {
    auto vector_type = field.value.type.VectorType();
    auto method_name = namer_.Method("create", field, "vector");
    auto array_param = GenerateKotlinOffsetArray(vector_type);
    auto params = "builder: FlatBufferBuilder, vector:" + array_param;
    auto return_type = GenType(field.value.type);
    writer.SetValue("size", NumToString(InlineSize(vector_type)));
    writer.SetValue("align", NumToString(InlineAlignment(vector_type)));
    writer.SetValue("root", GenMethod(vector_type));

    GenerateFun(
        writer, method_name, params, return_type,
        [&]() {
          writer += "builder.startVector({{size}}, vector.size, {{align}})";
          writer += "for (i in vector.size - 1 downTo 0) {";
          writer.IncrementIdentLevel();
          writer += "builder.add{{root}}(vector[i])";
          writer.DecrementIdentLevel();
          writer += "}";
          writer += "return builder.endVector()";
        },
        options.gen_jvmstatic);
  }

  void GenerateStartVectorField(FieldDef &field, CodeWriter &writer,
                                const IDLOptions options) const {
    // Generate a method to start a vector, data to be added manually
    // after.
    auto vector_type = field.value.type.VectorType();
    auto params = "builder: FlatBufferBuilder, numElems: Int";
    writer.SetValue("size", NumToString(InlineSize(vector_type)));
    writer.SetValue("align", NumToString(InlineAlignment(vector_type)));

    GenerateFunOneLine(
        writer, namer_.Method("start", field, "Vector"), params, "",
        [&]() {
          writer += "builder.startVector({{size}}, numElems, {{align}})";
        },
        options.gen_jvmstatic);
  }

  void GenerateAddField(std::string field_pos, FieldDef &field,
                        CodeWriter &writer, const IDLOptions options) const {
    auto field_type = GenType(field.value.type);
    auto secondArg = namer_.Variable(field.name) + ": " + field_type;

    auto content = [&]() {
      auto method = GenMethod(field.value.type);
      auto default_value = GenDefaultBufferValue(field);
      auto field_param = namer_.Field(field);
      if (IsEnum(field.value.type) || IsStruct(field.value.type)) {
        field_param += ".value";
      }

      writer.SetValue("field_name", namer_.Field(field));
      writer.SetValue("field_param", field_param);
      writer.SetValue("method_name", method);
      writer.SetValue("pos", field_pos);
      writer.SetValue("default", default_value);

      if (field.key) {
        // field has key attribute, so always need to exist
        // even if its value is equal to default.
        // Generated code will bypass default checking
        // resulting in { builder.addShort(name); slot(id); }
        writer += "builder.add{{method_name}}({{field_name}})";
        writer += "builder.slot({{pos}})";
      } else {
        writer += "builder.add{{method_name}}({{pos}}, \\";
        writer += "{{field_param}}, {{default}})";
      }
    };
    auto signature = namer_.LegacyKotlinMethod("add", field, "");
    auto params = "builder: FlatBufferBuilder, " + secondArg;
    if (field.key) {
      GenerateFun(writer, signature, params, "", content,
                  options.gen_jvmstatic);
    } else {
      GenerateFunOneLine(writer, signature, params, "", content,
                         options.gen_jvmstatic);
    }
  }

  // fun startMonster(builder: FlatBufferBuilder) = builder.startTable(11)
  void GenerateStartStructMethod(StructDef &struct_def, CodeWriter &code,
                                 const IDLOptions options) const {
    GenerateFunOneLine(
        code, namer_.LegacyJavaMethod2("start", struct_def, ""),
        "builder: FlatBufferBuilder", "",
        [&]() {
          code += "builder.startTable(" +
                  NumToString(struct_def.fields.vec.size()) + ")";
        },
        options.gen_jvmstatic);
  }

  void GenerateTableCreator(StructDef &struct_def, CodeWriter &writer,
                            const IDLOptions options) const {
    // Generate a method that creates a table in one go. This is only possible
    // when the table has no struct fields, since those have to be created
    // inline, and there's no way to do so in Java.
    bool has_no_struct_fields = true;
    int num_fields = 0;
    auto fields_vec = struct_def.fields.vec;

    for (auto it = fields_vec.begin(); it != fields_vec.end(); ++it) {
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
    if (has_no_struct_fields && num_fields && num_fields < 127) {
      // Generate a table constructor of the form:
      // public static int createName(FlatBufferBuilder builder, args...)

      auto name = namer_.LegacyJavaMethod2("create", struct_def, "");
      std::stringstream params;
      params << "builder: FlatBufferBuilder";
      for (auto it = fields_vec.begin(); it != fields_vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        params << ", " << namer_.Variable(field);
        if (!IsScalar(field.value.type.base_type)) {
          params << "Offset: ";
        } else {
          params << ": ";
        }
        auto optional = field.IsScalarOptional() ? "?" : "";
        params << GenType(field.value.type) << optional;
      }

      GenerateFun(
          writer, name, params.str(), "Offset<" + namer_.Type(struct_def) + '>',
          [&]() {
            writer.SetValue("vec_size", NumToString(fields_vec.size()));
            writer.SetValue("end_method",
                            namer_.Method("end", struct_def.name));
            writer += "builder.startTable({{vec_size}})";

            auto sortbysize = struct_def.sortbysize;
            auto largest = sortbysize ? sizeof(largest_scalar_t) : 1;
            for (size_t size = largest; size; size /= 2) {
              for (auto it = fields_vec.rbegin(); it != fields_vec.rend();
                   ++it) {
                auto &field = **it;
                auto base_type_size = SizeOf(field.value.type.base_type);
                if (!field.deprecated &&
                    (!sortbysize || size == base_type_size)) {
                  writer.SetValue("field_name", namer_.Field(field));

                  // we wrap on null check for scalar optionals
                  writer += field.IsScalarOptional()
                                ? "{{field_name}}?.run { \\"
                                : "\\";

                  writer += namer_.LegacyKotlinMethod("add", field, "") +
                            "(builder, {{field_name}}\\";
                  if (!IsScalar(field.value.type.base_type)) {
                    writer += "Offset\\";
                  }
                  // we wrap on null check for scalar optionals
                  writer += field.IsScalarOptional() ? ") }" : ")";
                }
              }
            }
            writer += "return {{end_method}}(builder)";
          },
          options.gen_jvmstatic);
    }
  }
  void GenerateBufferHasIdentifier(StructDef &struct_def, CodeWriter &writer,
                                   IDLOptions options) const {
    auto file_identifier = parser_.file_identifier_;
    // Check if a buffer has the identifier.
    if (parser_.root_struct_def_ != &struct_def || !file_identifier.length())
      return;
    auto name = namer_.Function(struct_def);
    GenerateFunOneLine(
        writer, name + "BufferHasIdentifier", "buffer: ReadWriteBuffer",
        "Boolean",
        [&]() {
          writer += "hasIdentifier(buffer, \"" + file_identifier + "\")";
        },
        options.gen_jvmstatic);
  }

  void GenerateStructGetters(StructDef &struct_def, CodeWriter &writer) const {
    auto fields_vec = struct_def.fields.vec;
    FieldDef *key_field = nullptr;
    for (auto it = fields_vec.begin(); it != fields_vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      if (field.key) key_field = &field;

      GenerateComment(field.doc_comment, writer, &comment_config);

      auto field_name = namer_.Field(field);
      auto field_type = GenTypeGet(field.value.type);
      auto field_default_value = GenDefaultValue(field);
      auto return_type = GetterReturnType(field);
      auto bbgetter = ByteBufferGetter(field.value.type, "bb");
      auto offset_val = NumToString(field.value.offset);
      auto offset_prefix =
          "val o = offset(" + offset_val + "); return o != 0 ? ";
      auto value_base_type = field.value.type.base_type;
      // Most field accessors need to retrieve and test the field offset
      // first, this is the offset value for that:
      writer.SetValue("offset", NumToString(field.value.offset));
      writer.SetValue("return_type", return_type);
      writer.SetValue("field_type", field_type);
      writer.SetValue("field_name", field_name);
      writer.SetValue("field_default", field_default_value);
      writer.SetValue("bbgetter", bbgetter);
      // Generate the accessors that don't do object reuse.
      if (value_base_type == BASE_TYPE_STRUCT) {
        // Calls the accessor that takes an accessor object with a
        // new object.
        // val pos
        //     get() = pos(Vec3())
        GenerateGetterOneLine(writer, field_name, return_type, [&]() {
          writer += "{{field_name}}({{field_type}}())";
        });
      } else if (value_base_type == BASE_TYPE_VECTOR &&
                 field.value.type.element == BASE_TYPE_STRUCT) {
        // Accessors for vectors of structs also take accessor objects,
        // this generates a variant without that argument.
        // ex: fun weapons(j: Int) = weapons(Weapon(), j)
        GenerateFunOneLine(writer, field_name, "j: Int", return_type, [&]() {
          writer += "{{field_name}}({{field_type}}(), j)";
        });
      }

      if (IsScalar(value_base_type)) {
        if (struct_def.fixed) {
          GenerateGetterOneLine(writer, field_name, return_type, [&]() {
            std::string found = "{{bbgetter}}(bufferPos + {{offset}})";
            writer += WrapEnumValue(field.value.type, found);
          });
        } else {
          GenerateGetterOneLine(writer, field_name, return_type, [&]() {
            std::string found = "{{bbgetter}}(it + bufferPos)";
            writer += LookupFieldOneLine(offset_val,
                                         WrapEnumValue(field.value.type, found),
                                         "{{field_default}}");
          });
        }
      } else {
        switch (value_base_type) {
          case BASE_TYPE_STRUCT:
            if (struct_def.fixed) {
              // create getter with object reuse
              // ex:
              // fun pos(obj: Vec3) : Vec3? = obj.init(bufferPos + 4, bb)
              // ? adds nullability annotation
              GenerateFunOneLine(
                  writer, field_name, "obj: " + field_type, return_type, [&]() {
                    writer += "obj.init(bufferPos + {{offset}}, bb)";
                  });
            } else {
              // create getter with object reuse
              // ex:
              //  fun pos(obj: Vec3) : Vec3? {
              //      val o = offset(4)
              //      return if(o != 0) {
              //          obj.init(o + bufferPos, bb)
              //      else {
              //          null
              //      }
              //  }
              // ? adds nullability annotation
              GenerateFunOneLine(
                  writer, field_name, "obj: " + field_type, return_type, [&]() {
                    auto fixed = field.value.type.struct_def->fixed;

                    writer.SetValue("seek", Indirect("it + bufferPos", fixed));
                    writer += LookupFieldOneLine(
                        offset_val, "obj.init({{seek}}, bb)", "null");
                  });
            }
            break;
          case BASE_TYPE_STRING:
            // create string getter
            // e.g.
            // val Name : String?
            //     get() = {
            //         val o = offset(10)
            //         return if (o != 0) string(o + bufferPos) else null
            //     }
            // ? adds nullability annotation
            GenerateGetterOneLine(writer, field_name, return_type, [&]() {
              writer += LookupFieldOneLine(offset_val, "string(it + bufferPos)",
                                           "null");
            });
            break;
          case BASE_TYPE_VECTOR: {
            // e.g.
            // fun inventory(j: Int) : UByte {
            //     val o = offset(14)
            //     return if (o != 0) {
            //         bb.get(vector(it) + j * 1).toUByte()
            //     } else {
            //        0
            //     }
            // }

            auto vectortype = field.value.type.VectorType();
            std::string params = "j: Int";

            if (vectortype.base_type == BASE_TYPE_STRUCT ||
                vectortype.base_type == BASE_TYPE_UNION) {
              params = "obj: " + field_type + ", j: Int";
            }

            GenerateFunOneLine(writer, field_name, params, return_type, [&]() {
              auto inline_size = NumToString(InlineSize(vectortype));
              auto index = "vector(it) + j * " + inline_size;
              std::string found = "";
              writer.SetValue("index", index);

              if (IsEnum(vectortype)) {
                found = "{{field_type}}({{bbgetter}}({{index}}))";
              } else {
                switch (vectortype.base_type) {
                  case BASE_TYPE_STRUCT: {
                    bool fixed = vectortype.struct_def->fixed;
                    writer.SetValue("index", Indirect(index, fixed));
                    found = "obj.init({{index}}, bb)";
                    break;
                  }
                  case BASE_TYPE_UNION:
                    found = "{{bbgetter}}(obj, {{index}})";
                    break;
                  case BASE_TYPE_UTYPE:
                    found = "{{field_type}}({{bbgetter}}({{index}}))";
                    break;
                  default: found = "{{bbgetter}}({{index}})";
                }
              }
              writer +=
                  LookupFieldOneLine(offset_val, found, "{{field_default}}");
            });
            break;
          }
          case BASE_TYPE_UNION:
            GenerateFunOneLine(
                writer, field_name, "obj: " + field_type, return_type, [&]() {
                  writer += LookupFieldOneLine(
                      offset_val, bbgetter + "(obj, it + bufferPos)", "null");
                });
            break;
          default: FLATBUFFERS_ASSERT(0);
        }
      }

      if (value_base_type == BASE_TYPE_VECTOR) {
        // Generate Lenght functions for vectors
        GenerateGetterOneLine(writer, field_name + "Length", "Int", [&]() {
          writer += LookupFieldOneLine(offset_val, "vectorLength(it)", "0");
        });

        // See if we should generate a by-key accessor.
        if (field.value.type.element == BASE_TYPE_STRUCT &&
            !field.value.type.struct_def->fixed) {
          auto &sd = *field.value.type.struct_def;
          auto &fields = sd.fields.vec;
          for (auto kit = fields.begin(); kit != fields.end(); ++kit) {
            auto &kfield = **kit;
            if (kfield.key) {
              auto qualified_name = TypeInNameSpace(sd);
              auto name = namer_.Method(field, "ByKey");
              auto params = "key: " + GenTypeGet(kfield.value.type);
              auto rtype = qualified_name + "?";
              GenerateFunOneLine(writer, name, params, rtype, [&]() {
                writer += LookupFieldOneLine(
                    offset_val,
                    qualified_name + ".lookupByKey(null, vector(it), key, bb)",
                    "null");
              });

              auto param2 = "obj: " + qualified_name +
                            ", key: " + GenTypeGet(kfield.value.type);
              GenerateFunOneLine(writer, name, param2, rtype, [&]() {
                writer += LookupFieldOneLine(
                    offset_val,
                    qualified_name + ".lookupByKey(obj, vector(it), key, bb)",
                    "null");
              });

              break;
            }
          }
        }
      }

      if ((value_base_type == BASE_TYPE_VECTOR &&
           IsScalar(field.value.type.VectorType().base_type)) ||
          value_base_type == BASE_TYPE_STRING) {
        auto end_idx =
            NumToString(value_base_type == BASE_TYPE_STRING
                            ? 1
                            : InlineSize(field.value.type.VectorType()));

        // Generate a ByteBuffer accessor for strings & vectors of scalars.
        // e.g.
        // fun inventoryInByteBuffer(buffer: Bytebuffer):
        //     ByteBuffer = vectorAsBuffer(buffer, 14, 1)
        GenerateFunOneLine(
            writer, field_name + "AsBuffer", "", "ReadBuffer", [&]() {
              writer.SetValue("end", end_idx);
              writer += "vectorAsBuffer(bb, {{offset}}, {{end}})";
            });
      }

      // generate object accessors if is nested_flatbuffer
      // fun testnestedflatbufferAsMonster() : Monster?
      //{ return testnestedflatbufferAsMonster(new Monster()); }

      if (field.nested_flatbuffer) {
        auto nested_type_name = TypeInNameSpace(*field.nested_flatbuffer);
        auto nested_method_name =
            field_name + "As" + field.nested_flatbuffer->name;

        GenerateGetterOneLine(
            writer, nested_method_name, nested_type_name + "?", [&]() {
              writer += nested_method_name + "(" + nested_type_name + "())";
            });

        GenerateFunOneLine(
            writer, nested_method_name, "obj: " + nested_type_name,
            nested_type_name + "?", [&]() {
              writer += LookupFieldOneLine(
                  offset_val, "obj.init(indirect(vector(it)), bb)", "null");
            });
      }

      writer += "";  // Initial line break between fields
    }
    if (struct_def.has_key && !struct_def.fixed) {
      // Key Comparison method
      GenerateOverrideFun(
          writer, "keysCompare",
          "o1: Offset<*>, o2: Offset<*>, buffer: ReadWriteBuffer", "Int",
          [&]() {
            if (IsString(key_field->value.type)) {
              writer.SetValue("offset", NumToString(key_field->value.offset));
              writer +=
                  " return compareStrings(offset({{offset}}, o1, "
                  "buffer), offset({{offset}}, o2, buffer), buffer)";

            } else {
              auto getter1 = GenLookupByKey(key_field, "buffer", "o1");
              auto getter2 = GenLookupByKey(key_field, "buffer", "o2");
              writer += "val a = " + getter1;
              writer += "val b = " + getter2;
              writer += "return (a - b).toInt().sign()";
            }
          });
    }
  }

  static std::string LiteralSuffix(const Type &type) {
    auto base = IsVector(type) ? type.element : type.base_type;
    switch (base) {
      case BASE_TYPE_UINT:
      case BASE_TYPE_UCHAR:
      case BASE_TYPE_UTYPE:
      case BASE_TYPE_USHORT: return "u";
      case BASE_TYPE_ULONG: return "UL";
      case BASE_TYPE_LONG: return "L";
      default: return "";
    }
  }

  std::string WrapEnumValue(const Type &type, const std::string value) const {
    if (IsEnum(type)) { return GenType(type) + "(" + value + ")"; }
    if (IsVector(type) && IsEnum(type.VectorType())) {
      return GenType(type.VectorType()) + "(" + value + ")";
    }
    return value;
  }

  void GenerateCompanionObject(CodeWriter &code,
                               const std::function<void()> &callback) const {
    code += "companion object {";
    code.IncrementIdentLevel();
    callback();
    code.DecrementIdentLevel();
    code += "}";
  }

  // Generate a documentation comment, if available.
  void GenerateComment(const std::vector<std::string> &dc, CodeWriter &writer,
                       const CommentConfig *config) const {
    if (dc.begin() == dc.end()) {
      // Don't output empty comment blocks with 0 lines of comment content.
      return;
    }

    if (config != nullptr && config->first_line != nullptr) {
      writer += std::string(config->first_line);
    }
    std::string line_prefix =
        ((config != nullptr && config->content_line_prefix != nullptr)
             ? config->content_line_prefix
             : "///");
    for (auto it = dc.begin(); it != dc.end(); ++it) {
      writer += line_prefix + *it;
    }
    if (config != nullptr && config->last_line != nullptr) {
      writer += std::string(config->last_line);
    }
  }

  void GenerateGetRootAsAccessors(const std::string &struct_name,
                                  CodeWriter &writer,
                                  IDLOptions options) const {
    // Generate a special accessor for the table that when used as the root
    // ex: fun getRootAsMonster(buffer: ByteBuffer): Monster {...}
    writer.SetValue("gr_name", struct_name);

    // create convenience method that doesn't require an existing object
    GenerateJvmStaticAnnotation(writer, options.gen_jvmstatic);
    GenerateFunOneLine(writer, "asRoot", "buffer: ReadWriteBuffer", struct_name,
                       [&]() { writer += "asRoot(buffer, {{gr_name}}())"; });

    // create method that allows object reuse
    // ex: fun Monster getRootAsMonster(buffer: ByteBuffer, obj: Monster) {...}
    GenerateJvmStaticAnnotation(writer, options.gen_jvmstatic);
    GenerateFunOneLine(
        writer, "asRoot", "buffer: ReadWriteBuffer, obj: {{gr_name}}",
        struct_name, [&]() {
          writer +=
              "obj.init(buffer.getInt(buffer.limit) + buffer.limit, buffer)";
        });
  }

  void GenerateStaticConstructor(const StructDef &struct_def, CodeWriter &code,
                                 const IDLOptions options) const {
    // create a struct constructor function
    auto params = StructConstructorParams(struct_def);
    GenerateFun(
        code, namer_.LegacyJavaMethod2("create", struct_def, ""), params,
        "Offset<" + namer_.Type(struct_def) + '>',
        [&]() {
          GenStructBody(struct_def, code, "");
          code += "return Offset(builder.offset())";
        },
        options.gen_jvmstatic);
  }

  std::string StructConstructorParams(const StructDef &struct_def,
                                      const std::string &prefix = "") const {
    // builder: FlatBufferBuilder
    std::stringstream out;
    auto field_vec = struct_def.fields.vec;
    if (prefix.empty()) { out << "builder: FlatBufferBuilder"; }
    for (auto it = field_vec.begin(); it != field_vec.end(); ++it) {
      auto &field = **it;
      if (IsStruct(field.value.type)) {
        // Generate arguments for a struct inside a struct. To ensure
        // names don't clash, and to make it obvious these arguments are
        // constructing a nested struct, prefix the name with the field
        // name.
        out << StructConstructorParams(*field.value.type.struct_def,
                                       prefix + (namer_.Variable(field) + "_"));
      } else {
        out << ", " << prefix << namer_.Variable(field) << ": "
            << GenType(field.value.type);
      }
    }
    return out.str();
  }

  static void GenerateVarGetterSetterOneLine(CodeWriter &writer,
                                             const std::string &name,
                                             const std::string &type,
                                             const std::string &getter,
                                             const std::string &setter) {
    // Generates Kotlin getter for properties
    // e.g.:
    // val prop: Mytype
    //     get() = {
    //       return x
    //     }
    writer.SetValue("name", name);
    writer.SetValue("type", type);
    writer += "var {{name}} : {{type}}";
    writer.IncrementIdentLevel();
    writer += "get() = " + getter;
    writer += "set(value) = " + setter;
    writer.DecrementIdentLevel();
  }

  static void GeneratePropertyOneLine(CodeWriter &writer,
                                      const std::string &name,
                                      const std::string &type,
                                      const std::function<void()> &body) {
    // Generates Kotlin getter for properties
    // e.g.:
    // val prop: Mytype = x
    writer.SetValue("_name", name);
    writer.SetValue("_type", type);
    writer += "val {{_name}} : {{_type}} = \\";
    body();
  }
  static void GenerateGetterOneLine(CodeWriter &writer, const std::string &name,
                                    const std::string &type,
                                    const std::function<void()> &body) {
    // Generates Kotlin getter for properties
    // e.g.:
    // val prop: Mytype get() = x
    writer.SetValue("_name", name);
    writer.SetValue("_type", type);
    writer += "val {{_name}} : {{_type}} get() = \\";
    body();
  }

  static void GenerateGetter(CodeWriter &writer, const std::string &name,
                             const std::string &type,
                             const std::function<void()> &body) {
    // Generates Kotlin getter for properties
    // e.g.:
    // val prop: Mytype
    //     get() = {
    //       return x
    //     }
    writer.SetValue("name", name);
    writer.SetValue("type", type);
    writer += "val {{name}} : {{type}}";
    writer.IncrementIdentLevel();
    writer += "get() {";
    writer.IncrementIdentLevel();
    body();
    writer.DecrementIdentLevel();
    writer += "}";
    writer.DecrementIdentLevel();
  }

  static void GenerateFun(CodeWriter &writer, const std::string &name,
                          const std::string &params,
                          const std::string &returnType,
                          const std::function<void()> &body,
                          bool gen_jvmstatic = false) {
    // Generates Kotlin function
    // e.g.:
    // fun path(j: Int): Vec3 {
    //     return path(Vec3(), j)
    // }
    auto noreturn = returnType.empty();
    writer.SetValue("name", name);
    writer.SetValue("params", params);
    writer.SetValue("return_type", noreturn ? "" : ": " + returnType);
    GenerateJvmStaticAnnotation(writer, gen_jvmstatic);
    writer += "fun {{name}}({{params}}) {{return_type}} {";
    writer.IncrementIdentLevel();
    body();
    writer.DecrementIdentLevel();
    writer += "}";
  }

  static void GenerateFunOneLine(CodeWriter &writer, const std::string &name,
                                 const std::string &params,
                                 const std::string &returnType,
                                 const std::function<void()> &body,
                                 bool gen_jvmstatic = false) {
    // Generates Kotlin function
    // e.g.:
    // fun path(j: Int): Vec3 = return path(Vec3(), j)
    auto ret = returnType.empty() ? "" : " : " + returnType;
    GenerateJvmStaticAnnotation(writer, gen_jvmstatic);
    writer += "fun " + name + "(" + params + ")" + ret + " = \\";
    body();
  }

  static void GenerateOverrideFun(CodeWriter &writer, const std::string &name,
                                  const std::string &params,
                                  const std::string &returnType,
                                  const std::function<void()> &body) {
    // Generates Kotlin function
    // e.g.:
    // override fun path(j: Int): Vec3 = return path(Vec3(), j)
    writer += "override \\";
    GenerateFun(writer, name, params, returnType, body);
  }

  static void GenerateOverrideFunOneLine(CodeWriter &writer,
                                         const std::string &name,
                                         const std::string &params,
                                         const std::string &returnType,
                                         const std::string &statement) {
    // Generates Kotlin function
    // e.g.:
    // override fun path(j: Int): Vec3 = return path(Vec3(), j)
    writer.SetValue("name", name);
    writer.SetValue("params", params);
    writer.SetValue("return_type",
                    returnType.empty() ? "" : " : " + returnType);
    writer += "override fun {{name}}({{params}}){{return_type}} = \\";
    writer += statement;
  }

  static std::string LookupFieldOneLine(const std::string &offset,
                                        const std::string &found,
                                        const std::string &not_found) {
    return "lookupField(" + offset + ", " + not_found + " ) { " + found + " }";
  }

  static std::string Indirect(const std::string &index, bool fixed) {
    // We apply indirect() and struct is not fixed.
    if (!fixed) return "indirect(" + index + ")";
    return index;
  }

  static std::string NotFoundReturn(BaseType el) {
    switch (el) {
      case BASE_TYPE_FLOAT: return "0.0f";
      case BASE_TYPE_DOUBLE: return "0.0";
      case BASE_TYPE_BOOL: return "false";
      case BASE_TYPE_LONG:
      case BASE_TYPE_INT:
      case BASE_TYPE_CHAR:
      case BASE_TYPE_SHORT: return "0";
      case BASE_TYPE_UINT:
      case BASE_TYPE_UCHAR:
      case BASE_TYPE_USHORT:
      case BASE_TYPE_UTYPE: return "0u";
      case BASE_TYPE_ULONG: return "0uL";
      default: return "null";
    }
  }

  // Prepend @JvmStatic to methods in companion object.
  static void GenerateJvmStaticAnnotation(CodeWriter &code,
                                          bool gen_jvmstatic) {
    if (gen_jvmstatic) { code += "@JvmStatic"; }
  }

  const IdlNamer namer_;
};
}  // namespace kotlin

static bool GenerateKotlinKMP(const Parser &parser, const std::string &path,
                       const std::string &file_name) {
  kotlin::KotlinKMPGenerator generator(parser, path, file_name);
  return generator.generate();
}

namespace {

class KotlinKMPCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateKotlinKMP(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    (void)parser;
    (void)path;
    (void)filename;
    (void)output;
    return Status::NOT_IMPLEMENTED;
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

  IDLOptions::Language Language() const override {
    return IDLOptions::kKotlinKmp;
  }

  std::string LanguageName() const override { return "Kotlin"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewKotlinKMPCodeGenerator() {
  return std::unique_ptr<KotlinKMPCodeGenerator>(new KotlinKMPCodeGenerator());
}

}  // namespace flatbuffers
