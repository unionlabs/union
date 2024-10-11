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

#include "idl_gen_java.h"

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

namespace flatbuffers {
namespace java {

namespace {

static Namer::Config JavaDefaultConfig() {
  return {
    /*types=*/Case::kKeep,
    /*constants=*/Case::kScreamingSnake,
    /*methods=*/Case::kLowerCamel,
    /*functions=*/Case::kLowerCamel,
    /*fields=*/Case::kLowerCamel,
    /*variables=*/Case::kLowerCamel,
    /*variants=*/Case::kKeep,
    /*enum_variant_seperator=*/".",
    /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
    /*namespaces=*/Case::kKeep,
    /*namespace_seperator=*/".",
    /*object_prefix=*/"",
    /*object_suffix=*/"T",
    /*keyword_prefix=*/"",
    /*keyword_suffix=*/"_",
    /*filenames=*/Case::kKeep,
    /*directories=*/Case::kKeep,
    /*output_path=*/"",
    /*filename_suffix=*/"_generated",
    /*filename_extension=*/".java",
  };
}

static std::set<std::string> JavaKeywords() {
  return {
    "abstract", "continue", "for",        "new",       "switch",
    "assert",   "default",  "goto",       "package",   "synchronized",
    "boolean",  "do",       "if",         "private",   "this",
    "break",    "double",   "implements", "protected", "throw",
    "byte",     "else",     "import",     "public",    "throws",
    "case",     "enum",     "instanceof", "return",    "transient",
    "catch",    "extends",  "int",        "short",     "try",
    "char",     "final",    "interface",  "static",    "void",
    "class",    "finally",  "long",       "strictfp",  "volatile",
    "const",    "float",    "native",     "super",     "while",
  };
}

static const TypedFloatConstantGenerator JavaFloatGen("Double.", "Float.",
                                                      "NaN",
                                                      "POSITIVE_INFINITY",
                                                      "NEGATIVE_INFINITY");

static const CommentConfig comment_config = {
  "/**",
  " *",
  " */",
};

}  // namespace

class JavaGenerator : public BaseGenerator {
  struct FieldArrayLength {
    std::string name;
    int length;
  };

 public:
  JavaGenerator(const Parser &parser, const std::string &path,
                const std::string &file_name, const std::string &package_prefix)
      : BaseGenerator(parser, path, file_name, "", ".", "java"),
        cur_name_space_(nullptr),
        namer_(WithFlagOptions(JavaDefaultConfig(), parser.opts, path),
               JavaKeywords()) {
    if (!package_prefix.empty()) {
      std::istringstream iss(package_prefix);
      std::string component;
      while (std::getline(iss, component, '.')) {
        package_prefix_ns_.components.push_back(component);
      }
      package_prefix_ = package_prefix_ns_.GetFullyQualifiedName("") + ".";
    }
  }

  JavaGenerator &operator=(const JavaGenerator &);
  bool generate() {
    std::string one_file_code;
    cur_name_space_ = parser_.current_namespace_;

    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      std::string enumcode;
      auto &enum_def = **it;
      if (!parser_.opts.one_file) cur_name_space_ = enum_def.defined_namespace;
      GenEnum(enum_def, enumcode);
      if (parser_.opts.one_file) {
        one_file_code += enumcode;
      } else {
        if (!SaveType(enum_def.name, *enum_def.defined_namespace, enumcode,
                      /* needs_includes= */ false))
          return false;
      }

      if (parser_.opts.generate_object_based_api && enum_def.is_union) {
        enumcode = "";
        GenEnum_ObjectAPI(enum_def, enumcode);
        auto class_name = namer_.Type(enum_def) + "Union";
        if (parser_.opts.one_file) {
          one_file_code += enumcode;
        } else {
          if (!SaveType(class_name, *enum_def.defined_namespace, enumcode,
                        /* needs_includes= */ false))
            return false;
        }
      }
    }

    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      std::string declcode;
      auto &struct_def = **it;
      if (!parser_.opts.one_file)
        cur_name_space_ = struct_def.defined_namespace;
      GenStruct(struct_def, declcode, parser_.opts);
      if (parser_.opts.one_file) {
        one_file_code += declcode;
      } else {
        if (!SaveType(struct_def.name, *struct_def.defined_namespace, declcode,
                      /* needs_includes= */ true))
          return false;
      }

      if (parser_.opts.generate_object_based_api) {
        declcode = "";
        GenStruct_ObjectAPI(struct_def, declcode);
        auto class_name = namer_.ObjectType(struct_def);
        if (parser_.opts.one_file) {
          one_file_code += declcode;
        } else {
          if (!SaveType(class_name, *struct_def.defined_namespace, declcode,
                        /* needs_includes= */ true))
            return false;
        }
      }
    }

    if (parser_.opts.one_file) {
      return SaveType(file_name_, *parser_.current_namespace_, one_file_code,
                      /* needs_includes= */ true);
    }
    return true;
  }

  // Save out the generated code for a single class while adding
  // declaration boilerplate.
  bool SaveType(const std::string &defname, const Namespace &ns,
                const std::string &classcode, bool needs_includes) const {
    if (!classcode.length()) return true;

    std::string code;
    code = "// " + std::string(FlatBuffersGeneratedWarning()) + "\n\n";

    Namespace combined_ns = package_prefix_ns_;
    std::copy(ns.components.begin(), ns.components.end(),
              std::back_inserter(combined_ns.components));

    const std::string namespace_name = FullNamespace(".", combined_ns);
    if (!namespace_name.empty()) {
      code += "package " + namespace_name + ";";
      code += "\n\n";
    }
    if (needs_includes) {
      code +=
          "import com.google.flatbuffers.BaseVector;\n"
          "import com.google.flatbuffers.BooleanVector;\n"
          "import com.google.flatbuffers.ByteVector;\n"
          "import com.google.flatbuffers.Constants;\n"
          "import com.google.flatbuffers.DoubleVector;\n"
          "import com.google.flatbuffers.FlatBufferBuilder;\n"
          "import com.google.flatbuffers.FloatVector;\n"
          "import com.google.flatbuffers.IntVector;\n"
          "import com.google.flatbuffers.LongVector;\n"
          "import com.google.flatbuffers.ShortVector;\n"
          "import com.google.flatbuffers.StringVector;\n"
          "import com.google.flatbuffers.Struct;\n"
          "import com.google.flatbuffers.Table;\n"
          "import com.google.flatbuffers.UnionVector;\n"
          "import java.nio.ByteBuffer;\n"
          "import java.nio.ByteOrder;\n";
      if (parser_.opts.gen_nullable) {
        code += "\nimport javax.annotation.Nullable;\n";
      }
      if (parser_.opts.java_checkerframework) {
        code += "\nimport org.checkerframework.dataflow.qual.Pure;\n";
      }
      code += "\n";
    }

    code += classcode;
    if (!namespace_name.empty()) code += "";
    const std::string dirs = namer_.Directories(combined_ns);
    EnsureDirExists(dirs);
    const std::string filename =
        dirs + namer_.File(defname, /*skips=*/SkipFile::Suffix);
    return SaveFile(filename.c_str(), code, false);
  }

  const Namespace *CurrentNameSpace() const { return cur_name_space_; }

  std::string GenNullableAnnotation(const Type &t) const {
    return parser_.opts.gen_nullable &&
                   !IsScalar(DestinationType(t, true).base_type) &&
                   t.base_type != BASE_TYPE_VECTOR
               ? " @Nullable "
               : "";
  }

  std::string GenPureAnnotation(const Type &t) const {
    return parser_.opts.java_checkerframework &&
                   !IsScalar(DestinationType(t, true).base_type)
               ? " @Pure "
               : "";
  }

  std::string GenTypeBasic(const Type &type) const {
    // clang-format off
    static const char * const java_typename[] = {
      #define FLATBUFFERS_TD(ENUM, IDLTYPE, CTYPE, JTYPE, ...) \
        #JTYPE,
        FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
      #undef FLATBUFFERS_TD
    };
    // clang-format on
    return java_typename[type.base_type];
  }

  std::string GenTypePointer(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "String";
      case BASE_TYPE_VECTOR: return GenTypeGet(type.VectorType());
      case BASE_TYPE_STRUCT:
        return Prefixed(namer_.NamespacedType(*type.struct_def));
      case BASE_TYPE_UNION: FLATBUFFERS_FALLTHROUGH();  // else fall thru
      default: return "Table";
    }
  }

  std::string GenTypeGet(const Type &type) const {
    return IsScalar(type.base_type)
               ? GenTypeBasic(type)
               : (IsArray(type) ? GenTypeGet(type.VectorType())
                                : GenTypePointer(type));
  }

  // Find the destination type the user wants to receive the value in (e.g.
  // one size higher signed types for unsigned serialized values in Java).
  Type DestinationType(const Type &type, bool vectorelem) const {
    switch (type.base_type) {
      // We use int for both uchar/ushort, since that generally means less
      // casting than using short for uchar.
      case BASE_TYPE_UCHAR: return Type(BASE_TYPE_INT);
      case BASE_TYPE_USHORT: return Type(BASE_TYPE_INT);
      case BASE_TYPE_UINT: return Type(BASE_TYPE_LONG);
      case BASE_TYPE_ARRAY:
      case BASE_TYPE_VECTOR:
        if (vectorelem) return DestinationType(type.VectorType(), vectorelem);
        FLATBUFFERS_FALLTHROUGH();  // else fall thru
      default: return type;
    }
  }

  std::string GenOffsetType() const { return "int"; }

  std::string GenOffsetConstruct(const std::string &variable_name) const {
    return variable_name;
  }

  std::string GenVectorOffsetType() const { return "int"; }

  // Generate destination type name
  std::string GenTypeNameDest(const Type &type) const {
    return GenTypeGet(DestinationType(type, true));
  }

  // Mask to turn serialized value into destination type value.
  std::string DestinationMask(const Type &type, bool vectorelem) const {
    switch (type.base_type) {
      case BASE_TYPE_UCHAR: return " & 0xFF";
      case BASE_TYPE_USHORT: return " & 0xFFFF";
      case BASE_TYPE_UINT: return " & 0xFFFFFFFFL";
      case BASE_TYPE_VECTOR:
        if (vectorelem) return DestinationMask(type.VectorType(), vectorelem);
        FLATBUFFERS_FALLTHROUGH();  // else fall thru
      default: return "";
    }
  }

  // Casts necessary to correctly read serialized data
  std::string DestinationCast(const Type &type) const {
    if (IsSeries(type)) {
      return DestinationCast(type.VectorType());
    } else {
      // Cast necessary to correctly read serialized unsigned values.
      if (type.base_type == BASE_TYPE_UINT) return "(long)";
    }
    return "";
  }

  // Cast statements for mutator method parameters.
  // In Java, parameters representing unsigned numbers need to be cast down to
  // their respective type. For example, a long holding an unsigned int value
  // would be cast down to int before being put onto the buffer.
  std::string SourceCast(const Type &type, bool castFromDest) const {
    if (IsSeries(type)) {
      return SourceCast(type.VectorType(), castFromDest);
    } else {
      if (castFromDest) {
        if (type.base_type == BASE_TYPE_UINT)
          return "(int) ";
        else if (type.base_type == BASE_TYPE_USHORT)
          return "(short) ";
        else if (type.base_type == BASE_TYPE_UCHAR)
          return "(byte) ";
      }
    }
    return "";
  }

  std::string SourceCast(const Type &type) const {
    return SourceCast(type, true);
  }

  std::string SourceCastBasic(const Type &type, bool castFromDest) const {
    return IsScalar(type.base_type) ? SourceCast(type, castFromDest) : "";
  }

  std::string SourceCastBasic(const Type &type) const {
    return SourceCastBasic(type, true);
  }

  std::string GenEnumDefaultValue(const FieldDef &field) const {
    auto &value = field.value;
    FLATBUFFERS_ASSERT(value.type.enum_def);
    auto &enum_def = *value.type.enum_def;
    auto enum_val = enum_def.FindByValue(value.constant);
    return enum_val
               ? Prefixed(namer_.NamespacedEnumVariant(enum_def, *enum_val))
               : value.constant;
  }

  std::string GenDefaultValue(const FieldDef &field) const {
    auto &value = field.value;
    auto constant = field.IsScalarOptional() ? "0" : value.constant;
    auto longSuffix = "L";
    switch (value.type.base_type) {
      case BASE_TYPE_BOOL: return constant == "0" ? "false" : "true";
      case BASE_TYPE_ULONG: {
        // Converts the ulong into its bits signed equivalent
        uint64_t defaultValue = StringToUInt(constant.c_str());
        return NumToString(static_cast<int64_t>(defaultValue)) + longSuffix;
      }
      case BASE_TYPE_UINT:
      case BASE_TYPE_LONG: return constant + longSuffix;
      default:
        if (IsFloat(value.type.base_type)) {
          if (field.IsScalarOptional()) {
            return value.type.base_type == BASE_TYPE_DOUBLE ? "0.0" : "0f";
          }
          return JavaFloatGen.GenFloatConstant(field);
        } else {
          return constant;
        }
    }
  }

  std::string GenDefaultValueBasic(const FieldDef &field) const {
    auto &value = field.value;
    if (!IsScalar(value.type.base_type)) { return "0"; }
    return GenDefaultValue(field);
  }

  void GenEnum(EnumDef &enum_def, std::string &code) const {
    if (enum_def.generated) return;

    // Generate enum definitions of the form:
    // public static (final) int name = value;
    // In Java, we use ints rather than the Enum feature, because we want them
    // to map directly to how they're used in C/C++ and file formats.
    // That, and Java Enums are expensive, and not universally liked.
    GenComment(enum_def.doc_comment, &code, &comment_config);

    code += "@SuppressWarnings(\"unused\")\n";
    if (enum_def.attributes.Lookup("private")) {
      // For Java, we leave the enum unmarked to indicate package-private
    } else {
      code += "public ";
    }
    code += "final class " + namer_.Type(enum_def);
    code += " {\n";
    code += "  private " + namer_.Type(enum_def) + "() { }\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      GenComment(ev.doc_comment, &code, &comment_config, "  ");
      code += "  public static final ";
      code += GenTypeBasic(DestinationType(enum_def.underlying_type, false));
      code += " ";
      code += namer_.Variant(ev) + " = ";
      code += enum_def.ToString(ev);
      if (enum_def.underlying_type.base_type == BASE_TYPE_LONG ||
          enum_def.underlying_type.base_type == BASE_TYPE_ULONG) {
        code += "L";
      }
      code += ";\n";
    }

    // Generate a string table for enum values.
    // Problem is, if values are very sparse that could generate really big
    // tables. Ideally in that case we generate a map lookup instead, but for
    // the moment we simply don't output a table at all.
    auto range = enum_def.Distance();
    // Average distance between values above which we consider a table
    // "too sparse". Change at will.
    static const uint64_t kMaxSparseness = 5;
    if (range / static_cast<uint64_t>(enum_def.size()) < kMaxSparseness &&
        GenTypeBasic(DestinationType(enum_def.underlying_type, false)) !=
            "long") {
      code += "\n  public static final String";
      code += "[] names = { ";
      const EnumVal *prev = enum_def.Vals().front();
      for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end();
           ++it) {
        const EnumVal &ev = **it;
        for (auto k = enum_def.Distance(prev, &ev); k > 1; --k)
          code += "\"\", ";
        prev = &ev;
        code += "\"" + namer_.Variant(ev) + "\", ";
      }
      code += "};\n\n";
      code += "  public static ";
      code += "String";
      code += " name";
      code += "(int e) { return names[e";
      if (enum_def.MinValue()->IsNonZero())
        code += " - " + namer_.Variant(enum_def.MinValue()->name);
      code += "]; }\n";
    }

    // Close the class
    code += "}\n\n";
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetter(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "__string";
      case BASE_TYPE_STRUCT: return "__struct";
      case BASE_TYPE_UNION: return "__union";
      case BASE_TYPE_VECTOR: return GenGetter(type.VectorType());
      case BASE_TYPE_ARRAY: return GenGetter(type.VectorType());
      default: {
        std::string getter = "bb.get";
        if (type.base_type == BASE_TYPE_BOOL) {
          getter = "0!=" + getter;
        } else if (GenTypeBasic(type) != "byte") {
          getter += ConvertCase(GenTypeBasic(type), Case::kUpperCamel);
        }
        return getter;
      }
    }
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetterForLookupByKey(flatbuffers::FieldDef *key_field,
                                      const std::string &data_buffer,
                                      const char *num = nullptr) const {
    auto type = key_field->value.type;
    auto dest_mask = DestinationMask(type, true);
    auto dest_cast = DestinationCast(type);
    auto getter = data_buffer + ".get";
    if (GenTypeBasic(type) != "byte") {
      getter += ConvertCase(GenTypeBasic(type), Case::kUpperCamel);
    }
    getter = dest_cast + getter + "(" + GenOffsetGetter(key_field, num) + ")" +
             dest_mask;
    return getter;
  }

  // Direct mutation is only allowed for scalar fields.
  // Hence a setter method will only be generated for such fields.
  std::string GenSetter(const Type &type) const {
    if (IsScalar(type.base_type)) {
      std::string setter = "bb.put";
      if (GenTypeBasic(type) != "byte" && type.base_type != BASE_TYPE_BOOL) {
        setter += ConvertCase(GenTypeBasic(type), Case::kUpperCamel);
      }
      return setter;
    } else {
      return "";
    }
  }

  // Returns the method name for use with add/put calls.
  std::string GenMethod(const Type &type) const {
    return IsScalar(type.base_type)
               ? ConvertCase(GenTypeBasic(type), Case::kUpperCamel)
               : (IsStruct(type) ? "Struct" : "Offset");
  }

  // Recursively generate arguments for a constructor, to deal with nested
  // structs.
  void GenStructArgs(const StructDef &struct_def, std::string &code,
                     const char *nameprefix, size_t array_count = 0) const {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      const auto array_field = IsArray(field_type);
      const auto &type = array_field ? field_type.VectorType()
                                     : DestinationType(field_type, false);
      const auto array_cnt = array_field ? (array_count + 1) : array_count;
      if (IsStruct(type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        GenStructArgs(*field_type.struct_def, code,
                      (nameprefix + (field.name + "_")).c_str(), array_cnt);
      } else {
        code += ", ";
        code += GenTypeNameDest(field.value.type);
        for (size_t i = 0; i < array_cnt; i++) code += "[]";
        code += " ";
        code += nameprefix;
        code += namer_.Field(field);
      }
    }
  }

  // Recusively generate struct construction statements of the form:
  // builder.putType(name);
  // and insert manual padding.
  void GenStructBody(const StructDef &struct_def, std::string &code,
                     const char *nameprefix, size_t index = 0,
                     bool in_array = false) const {
    std::string indent((index + 1) * 2, ' ');
    code += indent + "  builder.prep(";
    code += NumToString(struct_def.minalign) + ", ";
    code += NumToString(struct_def.bytesize) + ");\n";
    for (auto it = struct_def.fields.vec.rbegin();
         it != struct_def.fields.vec.rend(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      if (field.padding) {
        code += indent + "  builder.pad(";
        code += NumToString(field.padding) + ");\n";
      }
      if (IsStruct(field_type)) {
        GenStructBody(*field_type.struct_def, code,
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
          GenStructBody(*field_type.struct_def, code,
                        (nameprefix + (field.name + "_")).c_str(), index + 1,
                        in_array);
        } else {
          code += IsArray(field_type) ? "  " : "";
          code += indent + "  builder.put";
          code += GenMethod(type) + "(";
          code += SourceCast(type);
          auto argname = nameprefix + namer_.Variable(field);
          code += argname;
          size_t array_cnt = index + (IsArray(field_type) ? 1 : 0);
          for (size_t i = 0; in_array && i < array_cnt; i++) {
            code += "[_idx" + NumToString(i) + "-1]";
          }
          code += ");\n";
        }
        if (IsArray(field_type)) { code += indent + "  }\n"; }
      }
    }
  }

  std::string GenOffsetGetter(flatbuffers::FieldDef *key_field,
                              const char *num = nullptr) const {
    std::string key_offset = "";
    key_offset += "__offset(" + NumToString(key_field->value.offset) + ", ";
    if (num) {
      key_offset += num;
      key_offset += ", _bb)";
    } else {
      key_offset += "bb.capacity()";
      key_offset += " - tableOffset, bb)";
    }
    return key_offset;
  }

  std::string GenLookupKeyGetter(flatbuffers::FieldDef *key_field) const {
    std::string key_getter = "      ";
    key_getter += "int tableOffset = ";
    key_getter += "__indirect(vectorLocation + 4 * (start + middle)";
    key_getter += ", bb);\n      ";
    if (IsString(key_field->value.type)) {
      key_getter += "int comp = ";
      key_getter += "compareStrings(";
      key_getter += GenOffsetGetter(key_field);
      key_getter += ", byteKey, bb);\n";
    } else {
      auto get_val = GenGetterForLookupByKey(key_field, "bb");
      key_getter += GenTypeNameDest(key_field->value.type) + " val = ";
      key_getter += get_val + ";\n";
      key_getter += "      int comp = val > key ? 1 : val < key ? -1 : 0;\n";
    }
    return key_getter;
  }

  std::string GenKeyGetter(flatbuffers::FieldDef *key_field) const {
    std::string key_getter = "";
    auto data_buffer = "_bb";
    if (IsString(key_field->value.type)) {
      key_getter += " return ";
      key_getter += "";
      key_getter += "compareStrings(";
      key_getter += GenOffsetGetter(key_field, "o1") + ", ";
      key_getter += GenOffsetGetter(key_field, "o2") + ", " + data_buffer + ")";
      key_getter += ";";
    } else {
      auto field_getter = GenGetterForLookupByKey(key_field, data_buffer, "o1");
      key_getter +=
          "\n    " + GenTypeNameDest(key_field->value.type) + " val_1 = ";
      key_getter +=
          field_getter + ";\n    " + GenTypeNameDest(key_field->value.type);
      key_getter += " val_2 = ";
      field_getter = GenGetterForLookupByKey(key_field, data_buffer, "o2");
      key_getter += field_getter + ";\n";
      key_getter += "    return val_1 > val_2 ? 1 : val_1 < val_2 ? -1 : 0;\n ";
    }
    return key_getter;
  }

  void GenStruct(StructDef &struct_def, std::string &code,
                 const IDLOptions &opts) const {
    if (struct_def.generated) return;

    // Generate a struct accessor class, with methods of the form:
    // public type name() { return bb.getType(i + offset); }
    // or for tables of the form:
    // public type name() {
    //   int o = __offset(offset); return o != 0 ? bb.getType(o + i) : default;
    // }
    GenComment(struct_def.doc_comment, &code, &comment_config);

    if (parser_.opts.gen_generated) {
      code += "@javax.annotation.Generated(value=\"flatc\")\n";
    }
    code += "@SuppressWarnings(\"unused\")\n";
    if (struct_def.attributes.Lookup("private")) {
      // For Java, we leave the struct unmarked to indicate package-private
    } else {
      code += "public ";
    }
    const auto struct_class = namer_.Type(struct_def);
    code += "final class " + struct_class;
    code += " extends ";
    code += struct_def.fixed ? "Struct" : "Table";
    code += " {\n";

    if (!struct_def.fixed) {
      // Generate version check method.
      // Force compile time error if not using the same version runtime.
      code += "  public static void ValidateVersion() {";
      code += " Constants.";
      code += "FLATBUFFERS_24_3_25(); ";
      code += "}\n";

      // Generate a special accessor for the table that when used as the root
      // of a FlatBuffer
      const std::string method_name =
          namer_.LegacyJavaMethod2("getRootAs", struct_def, "");
      const std::string method_signature =
          "  public static " + struct_class + " " + method_name;

      // create convenience method that doesn't require an existing object
      code += method_signature + "(ByteBuffer _bb) ";
      code +=
          "{ return " + method_name + "(_bb, new " + struct_class + "()); }\n";

      // create method that allows object reuse
      code +=
          method_signature + "(ByteBuffer _bb, " + struct_class + " obj) { ";
      code += "_bb.order(ByteOrder.LITTLE_ENDIAN); ";
      code += "return (obj.__assign(_bb.getInt(_bb.";
      code += "position()";
      code += ") + _bb.";
      code += "position()";
      code += ", _bb)); }\n";
      if (parser_.root_struct_def_ == &struct_def) {
        if (parser_.file_identifier_.length()) {
          // Check if a buffer has the identifier.
          code += "  public static ";
          code += "boolean " +
                  namer_.LegacyJavaMethod2(
                      "", struct_def, "BufferHasIdentifier(ByteBuffer _bb)") +
                  " { return ";
          code += "__has_identifier(_bb, \"";
          code += parser_.file_identifier_;
          code += "\"); }\n";
        }
      }
    }
    // Generate the __init method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    code += "  public void __init(int _i, ByteBuffer _bb) ";
    code += "{ ";
    code += "__reset(_i, _bb); ";
    code += "}\n";
    code += "  public " + struct_class + " __assign(int _i, ByteBuffer _bb) ";
    code += "{ __init(_i, _bb); return this; }\n\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      GenComment(field.doc_comment, &code, &comment_config, "  ");
      const std::string type_name = GenTypeGet(field.value.type);
      const std::string type_name_dest = GenTypeNameDest(field.value.type);
      const std::string dest_mask = DestinationMask(field.value.type, true);
      const std::string dest_cast = DestinationCast(field.value.type);
      const std::string src_cast = SourceCast(field.value.type);
      const std::string method_start =
          "  public " +
          (field.IsRequired() ? "" : GenNullableAnnotation(field.value.type)) +
          GenPureAnnotation(field.value.type) + type_name_dest + " " +
          namer_.Field(field);
      const std::string obj = "obj";

      // Most field accessors need to retrieve and test the field offset first,
      // this is the prefix code for that:
      auto offset_prefix =
          IsArray(field.value.type)
              ? " { return "
              : (" { int o = __offset(" + NumToString(field.value.offset) +
                 "); return o != 0 ? ");
      // Generate the accessors that don't do object reuse.
      if (field.value.type.base_type == BASE_TYPE_STRUCT) {
        // Calls the accessor that takes an accessor object with a new object.
        code += method_start + "() { return ";
        code += namer_.Field(field);
        code += "(new ";
        code += type_name + "()); }\n";
      } else if (IsSeries(field.value.type) &&
                 field.value.type.element == BASE_TYPE_STRUCT) {
        // Accessors for vectors of structs also take accessor objects, this
        // generates a variant without that argument.
        code += method_start + "(int j) { return ";
        code += namer_.Field(field);
        code += "(new " + type_name + "(), j); }\n";
      }

      if (field.IsScalarOptional()) { code += GenOptionalScalarCheck(field); }
      std::string getter = dest_cast + GenGetter(field.value.type);
      code += method_start;
      std::string member_suffix = "; ";
      if (IsScalar(field.value.type.base_type)) {
        code += "()";
        member_suffix += "";
        if (struct_def.fixed) {
          code += " { return " + getter;
          code += "(bb_pos + ";
          code += NumToString(field.value.offset) + ")";
          code += dest_mask;
        } else {
          code += offset_prefix + getter;
          code += "(o + bb_pos)" + dest_mask;
          code += " : ";
          code += GenDefaultValue(field);
        }
      } else {
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT:
            code += "(" + type_name + " obj)";
            if (struct_def.fixed) {
              code += " { return " + obj + ".__assign(";
              code += "bb_pos + " + NumToString(field.value.offset) + ", ";
              code += "bb)";
            } else {
              code += offset_prefix;
              code += obj + ".__assign(";
              code += field.value.type.struct_def->fixed
                          ? "o + bb_pos"
                          : "__indirect(o + bb_pos)";
              code += ", bb) : null";
            }
            break;
          case BASE_TYPE_STRING:
            code += "()";
            member_suffix += "";
            code += offset_prefix + getter + "(o + ";
            code += "bb_pos) : null";
            break;
          case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_VECTOR: {
            auto vectortype = field.value.type.VectorType();
            code += "(";
            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += type_name + " obj, ";
              getter = obj + ".__assign";
            } else if (vectortype.base_type == BASE_TYPE_UNION) {
              code += type_name + " obj, ";
            }
            code += "int j)";
            const auto body = offset_prefix + getter + "(";
            if (vectortype.base_type == BASE_TYPE_UNION) {
              code += body + "obj, ";
            } else {
              code += body;
            }
            std::string index;
            if (IsArray(field.value.type)) {
              index += "bb_pos + " + NumToString(field.value.offset) + " + ";
            } else {
              index += "__vector(o) + ";
            }
            index += "j * " + NumToString(InlineSize(vectortype));
            if (vectortype.base_type == BASE_TYPE_STRUCT) {
              code += vectortype.struct_def->fixed
                          ? index
                          : "__indirect(" + index + ")";
              code += ", bb";
            } else {
              code += index;
            }
            code += ")" + dest_mask;
            if (!IsArray(field.value.type)) {
              code += " : ";
              code += field.value.type.element == BASE_TYPE_BOOL
                          ? "false"
                          : (IsScalar(field.value.type.element) ? "0" : "null");
            }

            break;
          }
          case BASE_TYPE_UNION:
            code += "(" + type_name + " obj)" + offset_prefix + getter;
            code += "(obj, o + bb_pos) : null";
            break;
          default: FLATBUFFERS_ASSERT(0);
        }
      }
      code += member_suffix;
      code += "}\n";
      if (IsVector(field.value.type)) {
        code += "  public int " + namer_.Field(field);
        code += "Length";
        code += "()";
        code += offset_prefix;
        code += "__vector_len(o) : 0; ";
        code += "";
        code += "}\n";
        // See if we should generate a by-key accessor.
        if (field.value.type.element == BASE_TYPE_STRUCT &&
            !field.value.type.struct_def->fixed) {
          auto &sd = *field.value.type.struct_def;
          auto &fields = sd.fields.vec;
          for (auto kit = fields.begin(); kit != fields.end(); ++kit) {
            auto &key_field = **kit;
            if (key_field.key) {
              auto qualified_name = Prefixed(namer_.NamespacedType(sd));
              code += "  public " + qualified_name + " ";
              code += namer_.Method(field) + "ByKey(";
              code += GenTypeNameDest(key_field.value.type) + " key)";
              code += offset_prefix;
              code += qualified_name + ".__lookup_by_key(";
              code += "null, ";
              code += "__vector(o), key, ";
              code += "bb) : null; ";
              code += "}\n";
              code += "  public " + qualified_name + " ";
              code += namer_.Method(field) + "ByKey(";
              code += qualified_name + " obj, ";
              code += GenTypeNameDest(key_field.value.type) + " key)";
              code += offset_prefix;
              code += qualified_name + ".__lookup_by_key(obj, ";
              code += "__vector(o), key, ";
              code += "bb) : null; ";
              code += "}\n";
              break;
            }
          }
        }
      }
      // Generate the accessors for vector of structs with vector access object
      if (IsVector(field.value.type)) {
        std::string vector_type_name;
        const auto &element_base_type = field.value.type.VectorType().base_type;
        if (IsScalar(element_base_type)) {
          vector_type_name =
              ConvertCase(type_name, Case::kUpperCamel) + "Vector";
        } else if (element_base_type == BASE_TYPE_STRING) {
          vector_type_name = "StringVector";
        } else if (element_base_type == BASE_TYPE_UNION) {
          vector_type_name = "UnionVector";
        } else {
          vector_type_name = type_name + ".Vector";
        }
        auto vector_method_start = GenNullableAnnotation(field.value.type) +
                                   "  public " + vector_type_name + " " +
                                   namer_.Field(field, "vector");
        code += vector_method_start + "() { return ";
        code += namer_.Field(field, "vector");
        code += "(new " + vector_type_name + "()); }\n";
        code += vector_method_start + "(" + vector_type_name + " obj)";
        code += offset_prefix + obj + ".__assign(";
        code += "__vector(o), ";
        if (!IsScalar(element_base_type)) {
          auto vectortype = field.value.type.VectorType();
          code += NumToString(InlineSize(vectortype)) + ", ";
        }
        code += "bb) : null" + member_suffix + "}\n";
      }
      // Generate a ByteBuffer accessor for strings & vectors of scalars.
      if ((IsVector(field.value.type) &&
           IsScalar(field.value.type.VectorType().base_type)) ||
          IsString(field.value.type)) {
        code += "  public ByteBuffer ";
        code += namer_.Field(field);
        code += "AsByteBuffer() { return ";
        code += "__vector_as_bytebuffer(";
        code += NumToString(field.value.offset) + ", ";
        code += NumToString(IsString(field.value.type)
                                ? 1
                                : InlineSize(field.value.type.VectorType()));
        code += "); }\n";
        code += "  public ByteBuffer ";
        code += namer_.Field(field);
        code += "InByteBuffer(ByteBuffer _bb) { return ";
        code += "__vector_in_bytebuffer(_bb, ";
        code += NumToString(field.value.offset) + ", ";
        code += NumToString(IsString(field.value.type)
                                ? 1
                                : InlineSize(field.value.type.VectorType()));
        code += "); }\n";
      }
      // generate object accessors if is nested_flatbuffer
      if (field.nested_flatbuffer) {
        auto nested_type_name =
            Prefixed(namer_.NamespacedType(*field.nested_flatbuffer));
        auto nested_method_name =
            namer_.Field(field) + "As" + field.nested_flatbuffer->name;
        auto get_nested_method_name = nested_method_name;
        code += "  public " + nested_type_name + " ";
        code += nested_method_name + "() { return ";
        code +=
            get_nested_method_name + "(new " + nested_type_name + "()); }\n";
        code += "  public " + nested_type_name + " ";
        code += get_nested_method_name + "(";
        code += nested_type_name + " obj";
        code += ") { int o = __offset(";
        code += NumToString(field.value.offset) + "); ";
        code += "return o != 0 ? " + obj + ".__assign(";
        code += "";
        code += "__indirect(__vector(o)), ";
        code += "bb) : null; }\n";
      }
      // Generate mutators for scalar fields or vectors of scalars.
      if (parser_.opts.mutable_buffer) {
        auto is_series = (IsSeries(field.value.type));
        const auto &underlying_type =
            is_series ? field.value.type.VectorType() : field.value.type;
        // Boolean parameters have to be explicitly converted to byte
        // representation.
        auto setter_parameter = underlying_type.base_type == BASE_TYPE_BOOL
                                    ? "(byte)(" + field.name + " ? 1 : 0)"
                                    : field.name;
        // A vector mutator also needs the index of the vector element it should
        // mutate.
        auto mutator_params = (is_series ? "(int j, " : "(") +
                              GenTypeNameDest(underlying_type) + " " +
                              field.name + ") { ";
        auto setter_index =
            is_series
                ? (IsArray(field.value.type)
                       ? "bb_pos + " + NumToString(field.value.offset)
                       : "__vector(o)") +
                      +" + j * " + NumToString(InlineSize(underlying_type))
                : (struct_def.fixed
                       ? "bb_pos + " + NumToString(field.value.offset)
                       : "o + bb_pos");
        if (IsScalar(underlying_type.base_type) && !IsUnion(field.value.type)) {
          code += "  public ";
          code += struct_def.fixed ? "void " : "boolean ";
          code += namer_.Method("mutate", field);
          code += mutator_params;
          if (struct_def.fixed) {
            code += GenSetter(underlying_type) + "(" + setter_index + ", ";
            code += src_cast + setter_parameter + "); }\n";
          } else {
            code += "int o = __offset(";
            code += NumToString(field.value.offset) + ");";
            code += " if (o != 0) { " + GenSetter(underlying_type);
            code += "(" + setter_index + ", " + src_cast + setter_parameter +
                    "); return true; } else { return false; } }\n";
          }
        }
      }
      if (parser_.opts.java_primitive_has_method &&
          IsScalar(field.value.type.base_type) && !struct_def.fixed) {
        auto vt_offset_constant = "  public static final int VT_" +
                                  namer_.Constant(field) + " = " +
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
      code += "  public static " + GenOffsetType() + " ";
      code += "create";
      code += struct_class + "(FlatBufferBuilder builder";
      GenStructArgs(struct_def, code, "");
      code += ") {\n";
      GenStructBody(struct_def, code, "");
      code += "    return ";
      code += GenOffsetConstruct("builder." + std::string("offset()"));
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
      if (has_no_struct_fields && num_fields && num_fields < 127) {
        struct_has_create = true;
        // Generate a table constructor of the form:
        // public static int createName(FlatBufferBuilder builder, args...)
        code += "  public static " + GenOffsetType() + " ";
        code += namer_.LegacyJavaMethod2("create", struct_def, "");
        code += "(FlatBufferBuilder builder";
        for (auto it = struct_def.fields.vec.begin();
             it != struct_def.fields.vec.end(); ++it) {
          auto &field = **it;
          auto field_name = namer_.Field(field);
          if (field.deprecated) continue;
          code += ",\n      ";
          code += GenTypeBasic(DestinationType(field.value.type, false));
          code += " ";
          code += field_name;
          if (!IsScalar(field.value.type.base_type)) code += "Offset";
        }
        code += ") {\n    builder.";
        code += "startTable(";
        code += NumToString(struct_def.fields.vec.size()) + ");\n";
        for (size_t size = struct_def.sortbysize ? sizeof(largest_scalar_t) : 1;
             size; size /= 2) {
          for (auto it = struct_def.fields.vec.rbegin();
               it != struct_def.fields.vec.rend(); ++it) {
            auto &field = **it;
            auto field_name = namer_.Field(field);
            if (!field.deprecated &&
                (!struct_def.sortbysize ||
                 size == SizeOf(field.value.type.base_type))) {
              code += "    " + struct_class + ".";
              code += namer_.Method("add", field) + "(builder, " + field_name;
              if (!IsScalar(field.value.type.base_type)) code += "Offset";
              code += ");\n";
            }
          }
        }
        code += "    return " + struct_class + ".";
        code += namer_.LegacyJavaMethod2("end", struct_def, "");
        code += "(builder);\n  }\n\n";
      }
      // Generate a set of static methods that allow table construction,
      // of the form:
      // public static void addName(FlatBufferBuilder builder, short name)
      // { builder.addShort(id, name, default); }
      // Unlike the Create function, these always work.
      code += "  public static void start";
      code += struct_class;
      code += "(FlatBufferBuilder builder) { builder.";
      code += "startTable(";
      code += NumToString(struct_def.fields.vec.size()) + "); }\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;

        code += "  public static void " + namer_.Method("add", field);
        code += "(FlatBufferBuilder builder, ";
        code += GenTypeBasic(DestinationType(field.value.type, false));
        auto argname = namer_.Field(field);
        if (!IsScalar(field.value.type.base_type)) argname += "Offset";
        code += " " + argname + ") { builder.add";
        code += GenMethod(field.value.type) + "(";

        if (field.key) {
          // field has key attribute, so always need to exist
          // even if its value is equal to default.
          // Generated code will bypass default checking
          // resulting in { builder.addShort(name); slot(id); }
          key_field = &field;
          code += SourceCastBasic(field.value.type);
          code += argname;
          code += "); builder.slot(" +
                  NumToString(it - struct_def.fields.vec.begin()) + "); }\n";
        } else {
          code += NumToString(it - struct_def.fields.vec.begin()) + ", ";
          code += SourceCastBasic(field.value.type);
          code += argname;
          code += ", ";
          code += SourceCastBasic(field.value.type);
          code += GenDefaultValue(field);
          code += "); }\n";
        }
        if (IsVector(field.value.type)) {
          auto vector_type = field.value.type.VectorType();
          auto alignment = InlineAlignment(vector_type);
          auto elem_size = InlineSize(vector_type);
          if (!IsStruct(vector_type)) {
            field_has_create_set.insert(&field);
            // generate a method to create a vector from a java array.
            if ((vector_type.base_type == BASE_TYPE_CHAR ||
                 vector_type.base_type == BASE_TYPE_UCHAR)) {
              // Handle byte[] and ByteBuffers separately for Java
              code += "  public static " + GenVectorOffsetType() + " ";
              code += namer_.Method("create", field);
              code += "Vector(FlatBufferBuilder builder, byte[] data) ";
              code += "{ return builder.createByteVector(data); }\n";

              code += "  public static " + GenVectorOffsetType() + " ";
              code += namer_.Method("create", field);
              code += "Vector(FlatBufferBuilder builder, ByteBuffer data) ";
              code += "{ return builder.createByteVector(data); }\n";
            } else {
              code += "  public static " + GenVectorOffsetType() + " ";
              code += namer_.Method("create", field);
              code += "Vector(FlatBufferBuilder builder, ";
              code += GenTypeBasic(DestinationType(vector_type, false)) +
                      "[] data) ";
              code += "{ builder.startVector(";
              code += NumToString(elem_size);
              code += ", data.length, ";
              code += NumToString(alignment);
              code += "); for (int i = data.";
              code += "length - 1; i >= 0; i--) builder.";
              code += "add";
              code += GenMethod(vector_type);
              code += "(";
              code += SourceCastBasic(vector_type);
              code += "data[i]";
              code += "); return ";
              code += "builder.endVector(); }\n";
            }
          }
          // Generate a method to start a vector, data to be added manually
          // after.
          code += "  public static void " + namer_.Method("start", field);
          code += "Vector(FlatBufferBuilder builder, int numElems) ";
          code += "{ builder.startVector(";
          code += NumToString(elem_size);
          code += ", numElems, " + NumToString(alignment);
          code += "); }\n";
        }
      }
      code += "  public static " + GenOffsetType() + " ";
      code += namer_.LegacyJavaMethod2("end", struct_def, "");
      code += "(FlatBufferBuilder builder) {\n    int o = builder.";
      code += "endTable();\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (!field.deprecated && field.IsRequired()) {
          code += "    builder.required(o, ";
          code += NumToString(field.value.offset);
          code += ");  // " + field.name + "\n";
        }
      }
      code += "    return " + GenOffsetConstruct("o") + ";\n  }\n";
      if (parser_.root_struct_def_ == &struct_def) {
        std::string size_prefix[] = { "", "SizePrefixed" };
        for (int i = 0; i < 2; ++i) {
          code += "  public static void ";
          code += namer_.LegacyJavaMethod2("finish" + size_prefix[i],
                                           struct_def, "Buffer");
          code += "(FlatBufferBuilder builder, " + GenOffsetType();
          code += " offset) {";
          code += " builder.finish" + size_prefix[i] + "(offset";

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
      code += "\n  @Override\n  protected int keysCompare(";
      code += "Integer o1, Integer o2, ByteBuffer _bb) {";
      code += GenKeyGetter(key_field);
      code += " }\n";

      code += "\n  public static " + struct_class;
      code += " __lookup_by_key(";
      code += struct_class + " obj, ";
      code += "int vectorLocation, ";
      code += GenTypeNameDest(key_field->value.type);
      code += " key, ByteBuffer bb) {\n";
      if (IsString(key_field->value.type)) {
        code += "    byte[] byteKey = ";
        code += "key.getBytes(java.nio.charset.StandardCharsets.UTF_8);\n";
      }
      code += "    int span = ";
      code += "bb.getInt(vectorLocation - 4);\n";
      code += "    int start = 0;\n";
      code += "    while (span != 0) {\n";
      code += "      int middle = span / 2;\n";
      code += GenLookupKeyGetter(key_field);
      code += "      if (comp > 0) {\n";
      code += "        span = middle;\n";
      code += "      } else if (comp < 0) {\n";
      code += "        middle++;\n";
      code += "        start += middle;\n";
      code += "        span -= middle;\n";
      code += "      } else {\n";
      code += "        return ";
      code += "(obj == null ? new " + struct_class + "() : obj)";
      code += ".__assign(tableOffset, bb);\n";
      code += "      }\n    }\n";
      code += "    return null;\n";
      code += "  }\n";
    }
    GenVectorAccessObject(struct_def, code);
    if (opts.generate_object_based_api) {
      GenPackUnPack_ObjectAPI(struct_def, code, opts, struct_has_create,
                              field_has_create_set);
    }
    code += "}\n\n";
  }

  std::string GenOptionalScalarCheck(FieldDef &field) const {
    if (!field.IsScalarOptional()) return "";
    return "  public boolean " + namer_.Method("has", field) +
           "() { return 0 != __offset(" + NumToString(field.value.offset) +
           "); }\n";
  }

  void GenVectorAccessObject(StructDef &struct_def, std::string &code) const {
    // Generate a vector of structs accessor class.
    code += "\n";
    code += "  ";
    if (!struct_def.attributes.Lookup("private")) code += "public ";
    code += "static ";
    code += "final ";
    code += "class Vector extends ";
    code += "BaseVector {\n";

    // Generate the __assign method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    std::string method_indent = "    ";
    code += method_indent + "public Vector ";
    code += "__assign(int _vector, int _element_size, ByteBuffer _bb) { ";
    code += "__reset(_vector, _element_size, _bb); return this; }\n\n";

    auto type_name = namer_.Type(struct_def);
    auto method_start = method_indent + "public " + type_name + " get";
    // Generate the accessors that don't do object reuse.
    code += method_start + "(int j) { return get";
    code += "(new " + type_name + "(), j); }\n";
    code += method_start + "(" + type_name + " obj, int j) { ";
    code += " return obj.__assign(";
    std::string index = "__element(j)";
    code += struct_def.fixed ? index : "__indirect(" + index + ", bb)";
    code += ", bb); }\n";
    // See if we should generate a by-key accessor.
    if (!struct_def.fixed) {
      auto &fields = struct_def.fields.vec;
      for (auto kit = fields.begin(); kit != fields.end(); ++kit) {
        auto &key_field = **kit;
        if (key_field.key) {
          auto nullable_annotation =
              parser_.opts.gen_nullable ? "@Nullable " : "";
          code += method_indent + nullable_annotation;
          code += "public " + type_name + " ";
          code += "getByKey(";
          code += GenTypeNameDest(key_field.value.type) + " key) { ";
          code += " return __lookup_by_key(null, ";
          code += "__vector(), key, ";
          code += "bb); ";
          code += "}\n";
          code += method_indent + nullable_annotation;
          code += "public " + type_name + " ";
          code += "getByKey(";
          code += type_name + " obj, ";
          code += GenTypeNameDest(key_field.value.type) + " key) { ";
          code += " return __lookup_by_key(obj, ";
          code += "__vector(), key, ";
          code += "bb); ";
          code += "}\n";
          break;
        }
      }
    }
    code += "  }\n";
  }

  void GenEnum_ObjectAPI(EnumDef &enum_def, std::string &code) const {
    if (enum_def.generated) return;
    code += "import com.google.flatbuffers.FlatBufferBuilder;\n\n";

    if (!enum_def.attributes.Lookup("private")) { code += "public "; }
    auto union_name = namer_.Type(enum_def) + "Union";
    auto union_type =
        GenTypeBasic(DestinationType(enum_def.underlying_type, false));
    code += "class " + union_name + " {\n";
    // Type
    code += "  private " + union_type + " type;\n";
    // Value
    code += "  private Object value;\n";
    code += "\n";
    // getters and setters
    code += "  public " + union_type + " getType() { return type; }\n\n";
    code += "  public void setType(" + union_type +
            " type) { this.type = type; }\n\n";
    code += "  public Object getValue() { return value; }\n\n";
    code += "  public void setValue(Object value) { this.value = value; }\n\n";
    // Constructor
    code += "  public " + union_name + "() {\n";
    code +=
        "    this.type = " + namer_.EnumVariant(enum_def, *enum_def.Vals()[0]) +
        ";\n";
    code += "    this.value = null;\n";
    code += "  }\n\n";
    // As
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      if (ev.union_type.base_type == BASE_TYPE_NONE) continue;
      auto type_name = GenTypeGet_ObjectAPI(ev.union_type, false, true);
      if (ev.union_type.base_type == BASE_TYPE_STRUCT &&
          ev.union_type.struct_def->attributes.Lookup("private")) {
        code += "  ";
      } else {
        code += "  public ";
      }
      code += type_name + " as" + ev.name + "() { return (" + type_name +
              ") value; }\n";
    }
    code += "\n";
    // pack()
    code += "  public static int pack(FlatBufferBuilder builder, " +
            union_name + " _o) {\n";
    code += "    switch (_o.type) {\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      if (ev.union_type.base_type == BASE_TYPE_NONE) {
        continue;
      } else {
        code += "      case " + namer_.EnumVariant(enum_def, ev) + ": return ";
        if (IsString(ev.union_type)) {
          code += "builder.createString(_o.as" + ev.name + "());\n";
        } else {
          code += GenTypeGet(ev.union_type) + ".pack(builder, _o.as" + ev.name +
                  "());\n";
        }
      }
    }
    code += "      default: return 0;\n";
    code += "    }\n";
    code += "  }\n";
    code += "}\n\n";
  }

  void GenUnionUnPack_ObjectAPI(const EnumDef &enum_def, std::string &code,
                                const std::string &type_name,
                                const std::string &field_name,
                                bool is_vector) const {
    const std::string variable_type =
        is_vector ? type_name.substr(0, type_name.length() - 2) : type_name;
    const std::string variable_name =
        "_" + namer_.Variable("o", field_name) + (is_vector ? "Element" : "");
    const std::string type_params = is_vector ? "_j" : "";
    const std::string value_params = is_vector ? ", _j" : "";
    const std::string indent = (is_vector ? "      " : "    ");

    code += indent + variable_type + " " + variable_name + " = new " +
            variable_type + "();\n";
    code += indent +
            GenTypeBasic(DestinationType(enum_def.underlying_type, false)) +
            " " + variable_name + "Type = " + field_name + "Type(" +
            type_params + ");\n";
    code += indent + variable_name + ".setType(" + variable_name + "Type);\n";
    code += indent + "Table " + variable_name + "Value;\n";
    code += indent + "switch (" + variable_name + "Type) {\n";
    for (auto eit = enum_def.Vals().begin(); eit != enum_def.Vals().end();
         ++eit) {
      auto &ev = **eit;
      if (ev.union_type.base_type == BASE_TYPE_NONE) {
        continue;
      } else {
        if (ev.union_type.base_type == BASE_TYPE_STRING ||
            (ev.union_type.base_type == BASE_TYPE_STRUCT &&
             ev.union_type.struct_def->fixed)) {
          continue;  // This branch is due to bad implemantation of Unions in
                     // Java which doesn't handle non Table types. Should be
                     // deleted when issue #6561 is fixed.
        }
        code += indent + "  case " +
                Prefixed(namer_.NamespacedEnumVariant(enum_def, ev)) + ":\n";
        auto actual_type = GenTypeGet(ev.union_type);
        code += indent + "    " + variable_name + "Value = " + field_name +
                "(new " + actual_type + "()" + value_params + ");\n";
        code += indent + "    " + variable_name + ".setValue(" + variable_name +
                "Value != null ? ((" + actual_type + ") " + variable_name +
                "Value).unpack() : null);\n";
        code += indent + "    break;\n";
      }
    }
    code += indent + "  default: break;\n";
    code += indent + "}\n";
    if (is_vector) {
      code += indent + "_" + namer_.Variable("o", field_name) +
              "[_j] = " + variable_name + ";\n";
    }
  }

  void GenPackUnPack_ObjectAPI(
      StructDef &struct_def, std::string &code, const IDLOptions &opts,
      bool struct_has_create,
      const std::set<FieldDef *> &field_has_create) const {
    auto struct_name = namer_.ObjectType(struct_def);
    // unpack()
    code += "  public " + struct_name + " unpack() {\n";
    code += "    " + struct_name + " _o = new " + struct_name + "();\n";
    code += "    unpackTo(_o);\n";
    code += "    return _o;\n";
    code += "  }\n";
    // unpackTo()
    code += "  public void unpackTo(" + struct_name + " _o) {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      const auto accessor = namer_.Method(field);
      const auto variable = "_" + namer_.Variable("o", field);
      const auto get_field = namer_.Method("get", field);
      const auto set_field = namer_.Method("set", field);

      auto type_name = GenTypeGet_ObjectAPI(field.value.type, false, true);
      if (field.IsScalarOptional())
        type_name = ConvertPrimitiveTypeToObjectWrapper_ObjectAPI(type_name);
      auto start = "    " + type_name + " " + variable + " = ";
      auto call_setter = true;
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          auto fixed = struct_def.fixed && field.value.type.struct_def->fixed;
          if (fixed) {
            code +=
                "    " + accessor + "().unpackTo(_o." + get_field + "());\n";
          } else {
            code += "    if (" + accessor + "() != null) ";
            if (field.value.type.struct_def->fixed) {
              code += accessor + "().unpackTo(_o." + get_field + "());\n";
            } else {
              code += "_o." + set_field + "(" + accessor + "().unpack());\n";
            }
            code += "    else _o." + set_field + "(null);\n";
          }
          call_setter = false;
          break;
        }
        case BASE_TYPE_ARRAY: {
          auto length_str = NumToString(field.value.type.fixed_length);
          auto unpack_method =
              field.value.type.struct_def == nullptr ? "" : ".unpack()";
          code += start + "_o." + get_field + "();\n";
          code += "    for (int _j = 0; _j < " + length_str + "; ++_j) { " +
                  variable + "[_j] = " + accessor + "(_j)" + unpack_method +
                  "; }\n";
          call_setter = false;
          break;
        }
        case BASE_TYPE_VECTOR:
          if (field.value.type.element == BASE_TYPE_UNION) {
            code += start + "new " +
                    GenConcreteTypeGet_ObjectAPI(field.value.type)
                        .substr(0, type_name.length() - 1) +
                    accessor + "Length()];\n";
            code +=
                "    for (int _j = 0; _j < " + accessor + "Length(); ++_j) {\n";
            GenUnionUnPack_ObjectAPI(*field.value.type.enum_def, code,
                                     type_name, accessor, true);
            code += "    }\n";
          } else if (field.value.type.element != BASE_TYPE_UTYPE) {
            auto fixed = field.value.type.struct_def == nullptr;
            const auto length_accessor = namer_.Method(field, "length");
            code += start + "new " +
                    GenConcreteTypeGet_ObjectAPI(field.value.type)
                        .substr(0, type_name.length() - 1) +
                    length_accessor + "()];\n";
            code +=
                "    for (int _j = 0; _j < " + length_accessor + "(); ++_j) {";
            code += variable + "[_j] = ";
            if (fixed) {
              code += accessor + "(_j)";
            } else {
              code += "(" + accessor + "(_j) != null ? " + accessor +
                      "(_j).unpack() : null)";
            }
            code += ";}\n";
          }
          break;
        case BASE_TYPE_UTYPE: break;
        case BASE_TYPE_UNION: {
          GenUnionUnPack_ObjectAPI(*field.value.type.enum_def, code, type_name,
                                   accessor, false);
          break;
        }
        default: {
          if (field.IsScalarOptional()) {
            code += start + namer_.Method("has", field) + "() ? " + accessor +
                    "() : null;\n";
          } else {
            code += start + accessor + "();\n";
          }
          break;
        }
      }
      if (call_setter) {
        code += "    _o." + set_field + "(" + variable + ");\n";
      }
    }
    code += "  }\n";
    // pack()
    code += "  public static " + GenOffsetType() +
            " pack(FlatBufferBuilder builder, " + struct_name + " _o) {\n";
    code += "    if (_o == null) return 0;\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;
      const auto field_name = namer_.Field(field);
      const auto variable = "_" + namer_.Variable("o", field);
      const auto get_field = namer_.Method("get", field);
      // pre
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          if (!field.value.type.struct_def->fixed) {
            code += "    " + GenOffsetType() + " _" + namer_.Variable(field) +
                    " = _o." + get_field +
                    "() == null ? 0 : " + GenTypeGet(field.value.type) +
                    ".pack(builder, _o." + get_field + "());\n";
          } else if (struct_def.fixed && struct_has_create) {
            std::vector<FieldArrayLength> array_lengths;
            FieldArrayLength tmp_array_length = {
              field.name,
              field.value.type.fixed_length,
            };
            array_lengths.push_back(tmp_array_length);
            GenStructPackDecl_ObjectAPI(*field.value.type.struct_def,
                                        array_lengths, code);
          }
          break;
        }
        case BASE_TYPE_STRING: {
          code += "    int _" + field_name + " = _o." + get_field +
                  "() == null ? 0 : "
                  "builder.createString(_o." +
                  get_field + "());\n";
          break;
        }
        case BASE_TYPE_VECTOR: {
          if (field_has_create.find(&field) != field_has_create.end()) {
            auto property_name = field_name;
            auto gen_for_loop = true;
            std::string array_name = "__" + field_name;
            std::string array_type = "";
            std::string element_type = "";
            std::string to_array = "";
            switch (field.value.type.element) {
              case BASE_TYPE_STRING: {
                array_type = "int";
                element_type = "String";
                to_array = "builder.createString(_e)";
                break;
              }
              case BASE_TYPE_STRUCT:
                array_type = "int";
                element_type =
                    GenTypeGet_ObjectAPI(field.value.type, true, true);
                ;
                to_array = GenTypeGet(field.value.type) + ".pack(builder, _e)";
                break;
              case BASE_TYPE_UTYPE:
                property_name = field_name.substr(0, field_name.size() - 4);
                array_type = GenTypeBasic(DestinationType(
                    field.value.type.enum_def->underlying_type, false));
                element_type = field.value.type.enum_def->name + "Union";
                to_array = "_o." + namer_.Method("get", property_name) +
                           "()[_j].getType()";
                break;
              case BASE_TYPE_UNION:
                array_type = "int";
                element_type = Prefixed(namer_.NamespacedType(
                                   *field.value.type.enum_def)) +
                               "Union";
                to_array = element_type + ".pack(builder,  _o." +
                           namer_.Method("get", property_name) + "()[_j])";
                break;
              case BASE_TYPE_UCHAR:  // TODO this branch of the switch is due to
                                     // inconsistent behavior in unsigned byte.
                                     // Read further at Issue #6574.
                array_type = "byte";
                element_type = "int";
                to_array = "(byte) _e";
                break;
              default:
                gen_for_loop = false;
                array_name = "_o." + namer_.Method("get", property_name) + "()";
                array_type = GenTypeNameDest(field.value.type);
                element_type = array_type;
                to_array = "_e";
                break;
            }
            code += "    int _" + field_name + " = 0;\n";
            code += "    if (_o." + namer_.Method("get", property_name) +
                    "() != null) {\n";
            if (gen_for_loop) {
              code += "      " + array_type + "[] " + array_name + " = new " +
                      array_type + "[_o." +
                      namer_.Method("get", property_name) + "().length];\n";
              code += "      int _j = 0;\n";
              code += "      for (" + element_type + " _e : _o." +
                      namer_.Method("get", property_name) + "()) { ";
              code += array_name + "[_j] = " + to_array + "; _j++;}\n";
            }
            code += "      _" + field_name + " = " +
                    namer_.Method("create", field) + "Vector(builder, " +
                    array_name + ");\n";
            code += "    }\n";
          } else {
            auto type_name = GenTypeGet(field.value.type);
            auto element_type_name =
                GenTypeGet_ObjectAPI(field.value.type, true, true);
            auto pack_method =
                field.value.type.struct_def == nullptr
                    ? "builder.add" + GenMethod(field.value.type.VectorType()) +
                          "(" + variable + "[_j]);"
                    : "_unused_offset = " + type_name + ".pack(builder, " +
                          variable + "[_j]);";
            code += "    int _" + field_name + " = 0;\n";
            code += "    " + element_type_name + "[] " + variable + " = _o." +
                    get_field + "();\n";
            code += "    if (" + variable + " != null) {\n";
            if (field.value.type.struct_def != nullptr) {
              code += "      int _unused_offset = 0;\n";
            }
            code += "      " + namer_.Method("start", field) +
                    "Vector(builder, " + variable + ".length);\n";
            code += "      for (int _j = " + variable +
                    ".length - 1; _j >=0; _j--) { ";
            code += pack_method + "}\n";
            code += "      _" + field_name + " = builder.endVector();\n";
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
            GenStructPackDecl_ObjectAPI(*field.value.type.struct_def,
                                        array_lengths, code);
          } else {
            code += "    " +
                    GenTypeGet_ObjectAPI(field.value.type, false, true) + " _" +
                    field_name + " = _o." + get_field + "();\n";
          }
          break;
        }
        case BASE_TYPE_UNION: {
          code += "    " +
                  GenTypeBasic(DestinationType(
                      field.value.type.enum_def->underlying_type, false)) +
                  " _" + field_name + "Type = _o." + get_field +
                  "() == null ? " +
                  Prefixed(namer_.NamespacedType(*field.value.type.enum_def)) +
                  ".NONE : " + "_o." + get_field + "().getType();\n";
          code += "    " + GenOffsetType() + " _" + field_name + " = _o." +
                  get_field + "() == null ? 0 : " +
                  Prefixed(namer_.NamespacedType(*field.value.type.enum_def)) +
                  "Union.pack(builder, _o." + get_field + "());\n";
          break;
        }
        default: break;
      }
    }
    if (struct_has_create) {
      // Create
      code += "    return " +
              namer_.LegacyJavaMethod2("create", struct_def, "") + "(\n";
      code += "      builder";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        const auto field_name = namer_.Field(field);
        const auto get_field = namer_.Method("get", field);
        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            if (struct_def.fixed) {
              GenStructPackCall_ObjectAPI(*field.value.type.struct_def, code,
                                          "      _" + field_name + "_");
            } else {
              code += ",\n";
              if (field.value.type.struct_def->fixed) {
                if (opts.generate_object_based_api)
                  code += "      _o." + field_name;
                else
                  // Seems like unreachable code
                  code += "      " + GenTypeGet(field.value.type) +
                          ".Pack(builder, _o." + field_name + ")";
              } else {
                code += "      _" + field_name;
              }
            }
            break;
          }
          case BASE_TYPE_ARRAY: {
            if (field.value.type.struct_def != nullptr) {
              GenStructPackCall_ObjectAPI(*field.value.type.struct_def, code,
                                          "      _" + field_name + "_");
            } else {
              code += ",\n";
              code += "      _" + field_name;
            }
            break;
          }
          case BASE_TYPE_UNION: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_UTYPE: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_STRING: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_VECTOR: {
            code += ",\n";
            code += "      _" + field_name;
            break;
          }
          default:  // scalar
            code += ",\n";
            code += "      _o." + get_field + "()";
            break;
        }
      }
      code += ");\n";
    } else {
      // Start, End
      code += "    " + namer_.LegacyJavaMethod2("start", struct_def, "") +
              "(builder);\n";
      for (auto it = struct_def.fields.vec.begin();
           it != struct_def.fields.vec.end(); ++it) {
        auto &field = **it;
        if (field.deprecated) continue;
        const auto arg = "_" + namer_.Variable(field);
        const auto get_field = namer_.Method("get", field);
        const auto add_field = namer_.Method("add", field);

        switch (field.value.type.base_type) {
          case BASE_TYPE_STRUCT: {
            if (field.value.type.struct_def->fixed) {
              code += "    " + add_field + "(builder, " +
                      GenTypeGet(field.value.type) + ".pack(builder, _o." +
                      get_field + "()));\n";
            } else {
              code += "    " + add_field + "(builder, " + arg + ");\n";
            }
            break;
          }
          case BASE_TYPE_STRING: FLATBUFFERS_FALLTHROUGH();  // fall thru
          case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();   // fall thru
          case BASE_TYPE_VECTOR: {
            code += "    " + add_field + "(builder, " + arg + ");\n";
            break;
          }
          case BASE_TYPE_UTYPE: break;
          case BASE_TYPE_UNION: {
            code += "    " + add_field + "Type(builder, " + arg + "Type);\n";
            code += "    " + add_field + "(builder, " + arg + ");\n";
            break;
          }
          // scalar
          default: {
            if (field.IsScalarOptional()) {
              code += "    if (_o." + get_field + "() != null) { " + add_field +
                      "(builder, _o." + get_field + "()); }\n";
            } else {
              code +=
                  "    " + add_field + "(builder, _o." + get_field + "());\n";
            }
            break;
          }
        }
      }
      code += "    return " + namer_.LegacyJavaMethod2("end", struct_def, "") +
              "(builder);\n";
    }
    code += "  }\n";
  }

  void GenStructPackDecl_ObjectAPI(const StructDef &struct_def,
                                   std::vector<FieldArrayLength> &array_lengths,
                                   std::string &code) const {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      const bool is_array = IsArray(field.value.type);
      const Type &field_type =
          is_array ? field.value.type.VectorType() : field.value.type;
      FieldArrayLength tmp_array_length = {
        field.name,
        field_type.fixed_length,
      };
      array_lengths.push_back(tmp_array_length);
      if (field_type.struct_def != nullptr) {
        GenStructPackDecl_ObjectAPI(*field_type.struct_def, array_lengths,
                                    code);
      } else {
        std::vector<FieldArrayLength> array_only_lengths;
        for (size_t i = 0; i < array_lengths.size(); ++i) {
          if (array_lengths[i].length > 0) {
            array_only_lengths.push_back(array_lengths[i]);
          }
        }
        std::string name;
        for (size_t i = 0; i < array_lengths.size(); ++i) {
          name += "_" + namer_.Variable(array_lengths[i].name);
        }
        code += "    " + GenTypeBasic(field_type);
        if (array_only_lengths.size() > 0) {
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            code += "[]";
          }
          code += " " + name + " = ";
          code += "new " + GenTypeBasic(field_type) + "[";
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            if (i != 0) { code += "]["; }
            code += NumToString(array_only_lengths[i].length);
          }
          code += "];\n";
          code += "    ";
          // initialize array
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            auto idx = "idx" + NumToString(i);
            code += "for (int " + idx + " = 0; " + idx + " < " +
                    NumToString(array_only_lengths[i].length) + "; ++" + idx +
                    ") {";
          }
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            auto idx = "idx" + NumToString(i);
            if (i == 0) {
              code += name + "[" + idx;
            } else {
              code += "][" + idx;
            }
          }
          code += "] = _o";
          for (size_t i = 0, j = 0; i < array_lengths.size(); ++i) {
            code += "." + namer_.Method("get", array_lengths[i].name) + "()";
            if (array_lengths[i].length <= 0) continue;
            code += "[idx" + NumToString(j++) + "]";
          }
          code += ";";
          for (size_t i = 0; i < array_only_lengths.size(); ++i) {
            code += "}";
          }
        } else {
          code += " " + name + " = ";
          code += SourceCast(field_type);
          code += "_o";
          for (size_t i = 0; i < array_lengths.size(); ++i) {
            code += "." + namer_.Method("get", array_lengths[i].name) + "()";
          }
          code += ";";
        }
        code += "\n";
      }
      array_lengths.pop_back();
    }
  }

  void GenStructPackCall_ObjectAPI(const StructDef &struct_def,
                                   std::string &code,
                                   std::string prefix) const {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      if (field_type.struct_def != nullptr) {
        GenStructPackCall_ObjectAPI(*field_type.struct_def, code,
                                    prefix + namer_.Field(field) + "_");
      } else {
        code += ",\n";
        code += prefix + namer_.Field(field);
      }
    }
  }

  std::string ConvertPrimitiveTypeToObjectWrapper_ObjectAPI(
      const std::string &type_name) const {
    if (type_name == "boolean")
      return "Boolean";
    else if (type_name == "byte")
      return "Byte";
    else if (type_name == "char")
      return "Character";
    else if (type_name == "short")
      return "Short";
    else if (type_name == "int")
      return "Integer";
    else if (type_name == "long")
      return "Long";
    else if (type_name == "float")
      return "Float";
    else if (type_name == "double")
      return "Double";
    return type_name;
  }

  std::string GenTypeGet_ObjectAPI(const flatbuffers::Type &type,
                                   bool vectorelem,
                                   bool wrap_in_namespace) const {
    auto type_name = GenTypeNameDest(type);
    // Replace to ObjectBaseAPI Type Name
    switch (type.base_type) {
      case BASE_TYPE_STRUCT: FLATBUFFERS_FALLTHROUGH();  // fall thru
      case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();   // fall thru
      case BASE_TYPE_VECTOR: {
        if (type.struct_def != nullptr) {
          auto type_name_length = type.struct_def->name.length();
          auto new_type_name = namer_.ObjectType(*type.struct_def);
          type_name.replace(type_name.length() - type_name_length,
                            type_name_length, new_type_name);
        } else if (type.element == BASE_TYPE_UNION) {
          if (wrap_in_namespace) {
            type_name =
                Prefixed(namer_.NamespacedType(*type.enum_def)) + "Union";
          } else {
            type_name = namer_.Type(*type.enum_def) + "Union";
          }
        }
        break;
      }

      case BASE_TYPE_UNION: {
        if (wrap_in_namespace) {
          type_name = Prefixed(namer_.NamespacedType(*type.enum_def)) + "Union";
        } else {
          type_name = namer_.Type(*type.enum_def) + "Union";
        }
        break;
      }
      default: break;
    }
    if (vectorelem) { return type_name; }
    switch (type.base_type) {
      case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();  // fall thru
      case BASE_TYPE_VECTOR: {
        type_name = type_name + "[]";
        break;
      }
      default: break;
    }
    return type_name;
  }

  std::string GenConcreteTypeGet_ObjectAPI(
      const flatbuffers::Type &type) const {
    auto type_name = GenTypeNameDest(type);
    // Replace to ObjectBaseAPI Type Name
    switch (type.base_type) {
      case BASE_TYPE_STRUCT: FLATBUFFERS_FALLTHROUGH();  // fall thru
      case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();   // fall thru
      case BASE_TYPE_VECTOR: {
        if (type.struct_def != nullptr) {
          auto type_name_length = type.struct_def->name.length();
          auto new_type_name = namer_.ObjectType(*type.struct_def);
          type_name.replace(type_name.length() - type_name_length,
                            type_name_length, new_type_name);
        } else if (type.element == BASE_TYPE_UNION) {
          type_name = Prefixed(namer_.NamespacedType(*type.enum_def)) + "Union";
        }
        break;
      }

      case BASE_TYPE_UNION: {
        type_name = Prefixed(namer_.NamespacedType(*type.enum_def)) + "Union";
        break;
      }
      default: break;
    }

    switch (type.base_type) {
      case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();  // fall thru
      case BASE_TYPE_VECTOR: {
        type_name = type_name + "[]";
        break;
      }
      default: break;
    }
    return type_name;
  }

  void GenStruct_ObjectAPI(const StructDef &struct_def,
                           std::string &code) const {
    if (struct_def.generated) return;
    if (struct_def.attributes.Lookup("private")) {
      // For Java, we leave the enum unmarked to indicate package-private
    } else {
      code += "public ";
    }

    const auto class_name = namer_.ObjectType(struct_def);
    code += "class " + class_name;
    code += " {\n";
    // Generate Properties
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      auto type_name = GenTypeGet_ObjectAPI(field.value.type, false, true);
      if (field.IsScalarOptional())
        type_name = ConvertPrimitiveTypeToObjectWrapper_ObjectAPI(type_name);
      const auto field_name = namer_.Field(field);
      code += "  private " + type_name + " " + field_name + ";\n";
    }
    // Generate Java getters and setters
    code += "\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      const auto field_name = namer_.Field(field);
      const auto get_field = namer_.Method("get", field);
      auto type_name = GenTypeGet_ObjectAPI(field.value.type, false, true);
      if (field.IsScalarOptional())
        type_name = ConvertPrimitiveTypeToObjectWrapper_ObjectAPI(type_name);

      code += "  public " + type_name + " " + get_field + "() { return " +
              field_name + "; }\n\n";
      std::string array_validation = "";
      if (field.value.type.base_type == BASE_TYPE_ARRAY) {
        array_validation =
            "if (" + field_name + " != null && " + field_name +
            ".length == " + NumToString(field.value.type.fixed_length) + ") ";
      }
      code += "  public void " + namer_.Method("set", field) + "(" + type_name +
              " " + field_name + ") { " + array_validation + "this." +
              field_name + " = " + field_name + "; }\n\n";
    }
    // Generate Constructor
    code += "\n";
    code += "  public " + class_name + "() {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const auto &field = **it;
      if (field.deprecated) continue;
      if (field.value.type.base_type == BASE_TYPE_UTYPE) continue;
      if (field.value.type.element == BASE_TYPE_UTYPE) continue;
      const auto get_field = namer_.Method("get", field);

      code += "    this." + namer_.Field(field) + " = ";
      const auto type_name =
          GenTypeGet_ObjectAPI(field.value.type, false, true);
      if (IsScalar(field.value.type.base_type)) {
        if (field.IsScalarOptional()) {
          code += "null;\n";
        } else {
          code += GenDefaultValue(field) + ";\n";
        }
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
    if (parser_.root_struct_def_ == &struct_def) {
      const std::string struct_type = namer_.Type(struct_def);
      code += "  public static " + class_name +
              " deserializeFromBinary(byte[] fbBuffer) {\n";
      code += "    return " + struct_type + "." +
              namer_.LegacyJavaMethod2("getRootAs", struct_def, "") +
              "(ByteBuffer.wrap(fbBuffer)).unpack();\n";
      code += "  }\n";
      code += "  public byte[] serializeToBinary() {\n";
      code += "    FlatBufferBuilder fbb = new FlatBufferBuilder();\n";
      code += "    " + struct_type + "." +
              namer_.LegacyJavaMethod2("finish", struct_def, "Buffer") +
              "(fbb, " + struct_type + ".pack(fbb, this));\n";
      code += "    return fbb.sizedByteArray();\n";
      code += "  }\n";
    }
    code += "}\n\n";
  }

  // This tracks the current namespace used to determine if a type need to be
  // prefixed by its namespace
  const Namespace *cur_name_space_;
  const IdlNamer namer_;

 private:
  std::string Prefixed(const std::string &str) const {
    return package_prefix_ + str;
  }

  std::string package_prefix_;
  Namespace package_prefix_ns_;
};
}  // namespace java

static bool GenerateJava(const Parser &parser, const std::string &path,
                         const std::string &file_name) {
  java::JavaGenerator generator(parser, path, file_name,
                                parser.opts.java_package_prefix);
  return generator.generate();
}

namespace {

class JavaCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateJava(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = JavaCSharpMakeRule(true, parser, path, filename);
    return Status::OK;
  }

  Status GenerateGrpcCode(const Parser &parser, const std::string &path,
                          const std::string &filename) override {
    if (!GenerateJavaGRPC(parser, path, filename)) { return Status::ERROR; }
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

  IDLOptions::Language Language() const override { return IDLOptions::kJava; }

  std::string LanguageName() const override { return "Java"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewJavaCodeGenerator() {
  return std::unique_ptr<JavaCodeGenerator>(new JavaCodeGenerator());
}

}  // namespace flatbuffers
