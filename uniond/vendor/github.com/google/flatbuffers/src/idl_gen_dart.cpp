/*
 * Copyright 2018 Dan Field
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
#include "idl_gen_dart.h"

#include <cassert>
#include <cmath>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

namespace flatbuffers {

namespace dart {

namespace {

static Namer::Config DartDefaultConfig() {
  return { /*types=*/Case::kUpperCamel,
           /*constants=*/Case::kScreamingSnake,
           /*methods=*/Case::kLowerCamel,
           /*functions=*/Case::kUnknown,  // unused.
           /*fields=*/Case::kLowerCamel,
           /*variables=*/Case::kLowerCamel,
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/".",
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kSnake2,
           /*namespace_seperator=*/".",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"$",
           /*keyword_suffix=*/"",
           /*filenames=*/Case::kKeep,
           /*directories=*/Case::kKeep,
           /*output_path=*/"",
           /*filename_suffix=*/"_generated",
           /*filename_extension=*/".dart" };
}

static std::set<std::string> DartKeywords() {
  // see https://www.dartlang.org/guides/language/language-tour#keywords
  // yield*, async*, and sync* shouldn't be proble
  return {
    "abstract", "else",       "import",    "show",     "as",        "enum",
    "in",       "static",     "assert",    "export",   "interface", "super",
    "async",    "extends",    "is",        "switch",   "await",     "extension",
    "late",     "sync",       "break",     "external", "library",   "this",
    "case",     "factory",    "mixin",     "throw",    "catch",     "false",
    "new",      "true",       "class",     "final",    "null",      "try",
    "const",    "finally",    "on",        "typedef",  "continue",  "for",
    "operator", "var",        "covariant", "Function", "part",      "void",
    "default",  "get",        "required",  "while",    "deferred",  "hide",
    "rethrow",  "with",       "do",        "if",       "return",    "yield",
    "dynamic",  "implements", "set",
  };
}
}  // namespace

const std::string _kFb = "fb";

// Iterate through all definitions we haven't generate code for (enums, structs,
// and tables) and output them to a single file.
class DartGenerator : public BaseGenerator {
 public:
  typedef std::map<std::string, std::string> namespace_code_map;

  DartGenerator(const Parser &parser, const std::string &path,
                const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "", ".", "dart"),
        namer_(WithFlagOptions(DartDefaultConfig(), parser.opts, path),
               DartKeywords()) {}

  template<typename T>
  void import_generator(const std::vector<T *> &definitions,
                        const std::string &included,
                        std::set<std::string> &imports) {
    for (const auto &item : definitions) {
      if (item->file == included) {
        std::string component = namer_.Namespace(*item->defined_namespace);
        std::string filebase =
            flatbuffers::StripPath(flatbuffers::StripExtension(item->file));
        std::string filename =
            namer_.File(filebase + (component.empty() ? "" : "_" + component));

        imports.emplace("import './" + filename + "'" +
                        (component.empty()
                             ? ";\n"
                             : " as " + ImportAliasName(component) + ";\n"));
      }
    }
  }

  // Iterate through all definitions we haven't generate code for (enums,
  // structs, and tables) and output them to a single file.
  bool generate() {
    std::string code;
    namespace_code_map namespace_code;
    GenerateEnums(namespace_code);
    GenerateStructs(namespace_code);

    std::set<std::string> imports;

    for (const auto &included_file : parser_.GetIncludedFiles()) {
      if (included_file.filename == parser_.file_being_parsed_) continue;

      import_generator(parser_.structs_.vec, included_file.filename, imports);
      import_generator(parser_.enums_.vec, included_file.filename, imports);
    }

    std::string import_code = "";
    for (const auto &file : imports) { import_code += file; }

    import_code += import_code.empty() ? "" : "\n";

    for (auto kv = namespace_code.begin(); kv != namespace_code.end(); ++kv) {
      code.clear();
      code = code + "// " + FlatBuffersGeneratedWarning() + "\n";
      code = code +
             "// ignore_for_file: unused_import, unused_field, unused_element, "
             "unused_local_variable\n\n";

      if (!kv->first.empty()) { code += "library " + kv->first + ";\n\n"; }

      code += "import 'dart:typed_data' show Uint8List;\n";
      code += "import 'package:flat_buffers/flat_buffers.dart' as " + _kFb +
              ";\n\n";

      for (auto kv2 = namespace_code.begin(); kv2 != namespace_code.end();
           ++kv2) {
        if (kv2->first != kv->first) {
          code += "import './" + Filename(kv2->first, /*path=*/false) +
                  "' as " + ImportAliasName(kv2->first) + ";\n";
        }
      }

      code += "\n";
      code += import_code;

      code += kv->second;

      if (!SaveFile(Filename(kv->first).c_str(), code, false)) { return false; }
    }
    return true;
  }

  std::string Filename(const std::string &suffix, bool path = true) const {
    return (path ? path_ : "") +
           namer_.File(file_name_ + (suffix.empty() ? "" : "_" + suffix));
  }

 private:
  static std::string ImportAliasName(const std::string &ns) {
    std::string ret;
    ret.assign(ns);
    size_t pos = ret.find('.');
    while (pos != std::string::npos) {
      ret.replace(pos, 1, "_");
      pos = ret.find('.', pos + 1);
    }

    return ret;
  }

  void GenerateEnums(namespace_code_map &namespace_code) {
    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      auto &enum_def = **it;
      GenEnum(enum_def, namespace_code);
    }
  }

  void GenerateStructs(namespace_code_map &namespace_code) {
    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      auto &struct_def = **it;
      GenStruct(struct_def, namespace_code);
    }
  }

  // Generate a documentation comment, if available.
  static void GenDocComment(const std::vector<std::string> &dc,
                            const char *indent, std::string &code) {
    for (auto it = dc.begin(); it != dc.end(); ++it) {
      if (indent) code += indent;
      code += "/// " + *it + "\n";
    }
  }

  // Generate an enum declaration and an enum string lookup table.
  void GenEnum(EnumDef &enum_def, namespace_code_map &namespace_code) {
    if (enum_def.generated) return;
    std::string &code =
        namespace_code[namer_.Namespace(*enum_def.defined_namespace)];
    GenDocComment(enum_def.doc_comment, "", code);

    const std::string enum_type =
        namer_.Type(enum_def) + (enum_def.is_union ? "TypeId" : "");
    const bool is_bit_flags =
        enum_def.attributes.Lookup("bit_flags") != nullptr;
    // The flatbuffer schema language allows bit flag enums to potentially have
    // a default value of zero, even if it's not a valid enum value...
    const bool permit_zero = is_bit_flags;

    code += "class " + enum_type + " {\n";
    code += "  final int value;\n";
    code += "  const " + enum_type + "._(this.value);\n\n";
    code += "  factory " + enum_type + ".fromValue(int value) {\n";
    code += "    final result = values[value];\n";
    code += "    if (result == null) {\n";
    if (permit_zero) {
      code += "      if (value == 0) {\n";
      code += "        return " + enum_type + "._(0);\n";
      code += "      } else {\n";
    }
    code += "        throw StateError('Invalid value $value for bit flag enum ";
    code += enum_type + "');\n";
    if (permit_zero) { code += "      }\n"; }
    code += "    }\n";

    code += "    return result;\n";
    code += "  }\n\n";

    code += "  static " + enum_type + "? _createOrNull(int? value) => \n";
    code +=
        "      value == null ? null : " + enum_type + ".fromValue(value);\n\n";

    // this is meaningless for bit_flags
    // however, note that unlike "regular" dart enums this enum can still have
    // holes.
    if (!is_bit_flags) {
      code += "  static const int minValue = " +
              enum_def.ToString(*enum_def.MinValue()) + ";\n";
      code += "  static const int maxValue = " +
              enum_def.ToString(*enum_def.MaxValue()) + ";\n";
    }

    code +=
        "  static bool containsValue(int value) =>"
        " values.containsKey(value);\n\n";

    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      const auto enum_var = namer_.Variant(ev);

      if (!ev.doc_comment.empty()) {
        if (it != enum_def.Vals().begin()) { code += '\n'; }
        GenDocComment(ev.doc_comment, "  ", code);
      }
      code += "  static const " + enum_type + " " + enum_var + " = " +
              enum_type + "._(" + enum_def.ToString(ev) + ");\n";
    }

    code += "  static const Map<int, " + enum_type + "> values = {\n";
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      const auto enum_var = namer_.Variant(ev);
      if (it != enum_def.Vals().begin()) code += ",\n";
      code += "    " + enum_def.ToString(ev) + ": " + enum_var;
    }
    code += "};\n\n";

    code += "  static const " + _kFb + ".Reader<" + enum_type + "> reader = _" +
            enum_type + "Reader();\n\n";
    code += "  @override\n";
    code += "  String toString() {\n";
    code += "    return '" + enum_type + "{value: $value}';\n";
    code += "  }\n";
    code += "}\n\n";

    GenEnumReader(enum_def, enum_type, code);
  }

  void GenEnumReader(EnumDef &enum_def, const std::string &enum_type,
                     std::string &code) {
    code += "class _" + enum_type + "Reader extends " + _kFb + ".Reader<" +
            enum_type + "> {\n";
    code += "  const _" + enum_type + "Reader();\n\n";
    code += "  @override\n";
    code += "  int get size => " + EnumSize(enum_def.underlying_type) + ";\n\n";
    code += "  @override\n";
    code += "  " + enum_type + " read(" + _kFb +
            ".BufferContext bc, int offset) =>\n";
    code += "      " + enum_type + ".fromValue(const " + _kFb + "." +
            GenType(enum_def.underlying_type) + "Reader().read(bc, offset));\n";
    code += "}\n\n";
  }

  std::string GenType(const Type &type) {
    switch (type.base_type) {
      case BASE_TYPE_BOOL: return "Bool";
      case BASE_TYPE_CHAR: return "Int8";
      case BASE_TYPE_UTYPE:
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
      case BASE_TYPE_STRUCT: return namer_.Type(*type.struct_def);
      case BASE_TYPE_UNION: return namer_.Type(*type.enum_def) + "TypeId";
      default: return "Table";
    }
  }

  static std::string EnumSize(const Type &type) {
    switch (type.base_type) {
      case BASE_TYPE_BOOL:
      case BASE_TYPE_CHAR:
      case BASE_TYPE_UTYPE:
      case BASE_TYPE_UCHAR: return "1";
      case BASE_TYPE_SHORT:
      case BASE_TYPE_USHORT: return "2";
      case BASE_TYPE_INT:
      case BASE_TYPE_UINT:
      case BASE_TYPE_FLOAT: return "4";
      case BASE_TYPE_LONG:
      case BASE_TYPE_ULONG:
      case BASE_TYPE_DOUBLE: return "8";
      default: return "1";
    }
  }

  std::string GenReaderTypeName(const Type &type, Namespace *current_namespace,
                                const FieldDef &def,
                                bool parent_is_vector = false, bool lazy = true,
                                bool constConstruct = true) {
    std::string prefix = (constConstruct ? "const " : "") + _kFb;
    if (type.base_type == BASE_TYPE_BOOL) {
      return prefix + ".BoolReader()";
    } else if (IsVector(type)) {
      if (!type.VectorType().enum_def) {
        if (type.VectorType().base_type == BASE_TYPE_CHAR) {
          return prefix + ".Int8ListReader(" + (lazy ? ")" : "lazy: false)");
        }
        if (type.VectorType().base_type == BASE_TYPE_UCHAR) {
          return prefix + ".Uint8ListReader(" + (lazy ? ")" : "lazy: false)");
        }
      }
      return prefix + ".ListReader<" +
             GenDartTypeName(type.VectorType(), current_namespace, def) + ">(" +
             GenReaderTypeName(type.VectorType(), current_namespace, def, true,
                               true, false) +
             (lazy ? ")" : ", lazy: false)");
    } else if (IsString(type)) {
      return prefix + ".StringReader()";
    }
    if (IsScalar(type.base_type)) {
      if (type.enum_def && parent_is_vector) {
        return GenDartTypeName(type, current_namespace, def) + ".reader";
      }
      return prefix + "." + GenType(type) + "Reader()";
    } else {
      return GenDartTypeName(type, current_namespace, def) + ".reader";
    }
  }

  std::string GenDartTypeName(const Type &type, Namespace *current_namespace,
                              const FieldDef &def,
                              std::string struct_type_suffix = "") {
    if (type.enum_def) {
      if (type.enum_def->is_union && type.base_type != BASE_TYPE_UNION) {
        return namer_.Type(*type.enum_def) + "TypeId";
      } else if (type.enum_def->is_union) {
        return "dynamic";
      } else if (type.base_type != BASE_TYPE_VECTOR) {
        return namer_.Type(*type.enum_def);
      }
    }

    switch (type.base_type) {
      case BASE_TYPE_BOOL: return "bool";
      case BASE_TYPE_LONG:
      case BASE_TYPE_ULONG:
      case BASE_TYPE_INT:
      case BASE_TYPE_UINT:
      case BASE_TYPE_SHORT:
      case BASE_TYPE_USHORT:
      case BASE_TYPE_CHAR:
      case BASE_TYPE_UCHAR: return "int";
      case BASE_TYPE_FLOAT:
      case BASE_TYPE_DOUBLE: return "double";
      case BASE_TYPE_STRING: return "String";
      case BASE_TYPE_STRUCT:
        return MaybeWrapNamespace(
            namer_.Type(*type.struct_def) + struct_type_suffix,
            current_namespace, def);
      case BASE_TYPE_VECTOR:
        return "List<" +
               GenDartTypeName(type.VectorType(), current_namespace, def,
                               struct_type_suffix) +
               ">";
      default: assert(0); return "dynamic";
    }
  }

  std::string GenDartTypeName(const Type &type, Namespace *current_namespace,
                              const FieldDef &def, bool nullable,
                              std::string struct_type_suffix) {
    std::string typeName =
        GenDartTypeName(type, current_namespace, def, struct_type_suffix);
    if (nullable && typeName != "dynamic") typeName += "?";
    return typeName;
  }

  std::string MaybeWrapNamespace(const std::string &type_name,
                                 Namespace *current_ns,
                                 const FieldDef &field) const {
    const std::string current_namespace = namer_.Namespace(*current_ns);
    const std::string field_namespace =
        field.value.type.struct_def
            ? namer_.Namespace(*field.value.type.struct_def->defined_namespace)
        : field.value.type.enum_def
            ? namer_.Namespace(*field.value.type.enum_def->defined_namespace)
            : "";

    if (field_namespace != "" && field_namespace != current_namespace) {
      return ImportAliasName(field_namespace) + "." + type_name;
    } else {
      return type_name;
    }
  }

  // Generate an accessor struct with constructor for a flatbuffers struct.
  void GenStruct(const StructDef &struct_def,
                 namespace_code_map &namespace_code) {
    if (struct_def.generated) return;

    std::string &code =
        namespace_code[namer_.Namespace(*struct_def.defined_namespace)];

    const auto &struct_type = namer_.Type(struct_def);

    // Emit constructor

    GenDocComment(struct_def.doc_comment, "", code);

    auto reader_name = "_" + struct_type + "Reader";
    auto builder_name = struct_type + "Builder";
    auto object_builder_name = struct_type + "ObjectBuilder";

    std::string reader_code, builder_code;

    code += "class " + struct_type + " {\n";

    code += "  " + struct_type + "._(this._bc, this._bcOffset);\n";
    if (!struct_def.fixed) {
      code += "  factory " + struct_type + "(List<int> bytes) {\n";
      code +=
          "    final rootRef = " + _kFb + ".BufferContext.fromBytes(bytes);\n";
      code += "    return reader.read(rootRef, 0);\n";
      code += "  }\n";
    }

    code += "\n";
    code += "  static const " + _kFb + ".Reader<" + struct_type +
            "> reader = " + reader_name + "();\n\n";

    code += "  final " + _kFb + ".BufferContext _bc;\n";
    code += "  final int _bcOffset;\n\n";

    std::vector<std::pair<int, FieldDef *>> non_deprecated_fields;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      FieldDef &field = **it;
      if (field.deprecated) continue;
      auto offset = static_cast<int>(it - struct_def.fields.vec.begin());
      non_deprecated_fields.push_back(std::make_pair(offset, &field));
    }

    GenImplementationGetters(struct_def, non_deprecated_fields, code);

    if (parser_.opts.generate_object_based_api) {
      code +=
          "\n" + GenStructObjectAPIUnpack(struct_def, non_deprecated_fields);

      code += "\n  static int pack(fb.Builder fbBuilder, " +
              namer_.ObjectType(struct_def) + "? object) {\n";
      code += "    if (object == null) return 0;\n";
      code += "    return object.pack(fbBuilder);\n";
      code += "  }\n";
    }

    code += "}\n\n";

    if (parser_.opts.generate_object_based_api) {
      code += GenStructObjectAPI(struct_def, non_deprecated_fields);
    }

    GenReader(struct_def, reader_name, reader_code);
    GenBuilder(struct_def, non_deprecated_fields, builder_name, builder_code);
    GenObjectBuilder(struct_def, non_deprecated_fields, object_builder_name,
                     builder_code);

    code += reader_code;
    code += builder_code;
  }

  // Generate an accessor struct with constructor for a flatbuffers struct.
  std::string GenStructObjectAPI(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields) {
    std::string code;
    GenDocComment(struct_def.doc_comment, "", code);

    std::string object_type = namer_.ObjectType(struct_def);
    code += "class " + object_type + " implements " + _kFb + ".Packable {\n";

    std::string constructor_args;
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;

      const std::string field_name = namer_.Field(field);
      const std::string defaultValue = getDefaultValue(field.value);
      const std::string type_name =
          GenDartTypeName(field.value.type, struct_def.defined_namespace, field,
                          defaultValue.empty() && !struct_def.fixed, "T");

      GenDocComment(field.doc_comment, "  ", code);
      code += "  " + type_name + " " + field_name + ";\n";

      if (!constructor_args.empty()) constructor_args += ",\n";
      constructor_args += "      ";
      constructor_args += (struct_def.fixed ? "required " : "");
      constructor_args += "this." + field_name;
      if (!struct_def.fixed && !defaultValue.empty()) {
        if (IsEnum(field.value.type)) {
          auto &enum_def = *field.value.type.enum_def;
          if (auto val = enum_def.FindByValue(defaultValue)) {
            constructor_args += " = " + namer_.EnumVariant(enum_def, *val);
          } else {
            constructor_args += " = const " + namer_.Type(enum_def) + "._(" +
                                defaultValue + ")";
          }
        } else {
          constructor_args += " = " + defaultValue;
        }
      }
    }

    if (!constructor_args.empty()) {
      code += "\n  " + object_type + "({\n" + constructor_args + "});\n\n";
    }

    code += GenStructObjectAPIPack(struct_def, non_deprecated_fields);
    code += "\n";
    code += GenToString(object_type, non_deprecated_fields);

    code += "}\n\n";
    return code;
  }

  // Generate function `StructNameT unpack()`
  std::string GenStructObjectAPIUnpack(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields) {
    std::string constructor_args;
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;

      const std::string field_name = namer_.Field(field);
      if (!constructor_args.empty()) constructor_args += ",\n";
      constructor_args += "      " + field_name + ": ";

      const Type &type = field.value.type;
      std::string defaultValue = getDefaultValue(field.value);
      bool isNullable = defaultValue.empty() && !struct_def.fixed;
      std::string nullableValueAccessOperator = isNullable ? "?" : "";
      if (type.base_type == BASE_TYPE_STRUCT) {
        constructor_args +=
            field_name + nullableValueAccessOperator + ".unpack()";
      } else if (type.base_type == BASE_TYPE_VECTOR) {
        if (type.VectorType().base_type == BASE_TYPE_STRUCT) {
          constructor_args += field_name + nullableValueAccessOperator +
                              ".map((e) => e.unpack()).toList()";
        } else {
          constructor_args +=
              GenReaderTypeName(field.value.type, struct_def.defined_namespace,
                                field, false, false);
          constructor_args += ".vTableGet";
          std::string offset = NumToString(field.value.offset);
          constructor_args +=
              isNullable
                  ? "Nullable(_bc, _bcOffset, " + offset + ")"
                  : "(_bc, _bcOffset, " + offset + ", " + defaultValue + ")";
        }
      } else {
        constructor_args += field_name;
      }
    }

    const std::string object_type = namer_.ObjectType(struct_def);
    std::string code = "  " + object_type + " unpack() => " + object_type + "(";
    if (!constructor_args.empty()) code += "\n" + constructor_args;
    code += ");\n";
    return code;
  }

  // Generate function `StructNameT pack()`
  std::string GenStructObjectAPIPack(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields) {
    std::string code;

    code += "  @override\n";
    code += "  int pack(fb.Builder fbBuilder) {\n";
    code += GenObjectBuilderImplementation(struct_def, non_deprecated_fields,
                                           false, true);
    code += "  }\n";
    return code;
  }

  std::string NamespaceAliasFromUnionType(Namespace *root_namespace,
                                          const Type &type) {
    const std::vector<std::string> qualified_name_parts =
        type.struct_def->defined_namespace->components;
    if (std::equal(root_namespace->components.begin(),
                   root_namespace->components.end(),
                   qualified_name_parts.begin())) {
      return namer_.Type(*type.struct_def);
    }

    std::string ns;

    for (auto it = qualified_name_parts.begin();
         it != qualified_name_parts.end(); ++it) {
      auto &part = *it;

      for (size_t i = 0; i < part.length(); i++) {
        if (i && !isdigit(part[i]) && part[i] == CharToUpper(part[i])) {
          ns += "_";
          ns += CharToLower(part[i]);
        } else {
          ns += CharToLower(part[i]);
        }
      }
      if (it != qualified_name_parts.end() - 1) { ns += "_"; }
    }

    return ns + "." + namer_.Type(*type.struct_def);
  }

  void GenImplementationGetters(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      std::string &code) {
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;

      const std::string field_name = namer_.Field(field);
      const std::string defaultValue = getDefaultValue(field.value);
      const bool isNullable = defaultValue.empty() && !struct_def.fixed;
      const std::string type_name =
          GenDartTypeName(field.value.type, struct_def.defined_namespace, field,
                          isNullable, "");

      GenDocComment(field.doc_comment, "  ", code);

      code += "  " + type_name + " get " + field_name;
      if (field.value.type.base_type == BASE_TYPE_UNION) {
        code += " {\n";
        code += "    switch (" + field_name + "Type?.value) {\n";
        const auto &enum_def = *field.value.type.enum_def;
        for (auto en_it = enum_def.Vals().begin() + 1;
             en_it != enum_def.Vals().end(); ++en_it) {
          const auto &ev = **en_it;
          const auto enum_name = NamespaceAliasFromUnionType(
              enum_def.defined_namespace, ev.union_type);
          code += "      case " + enum_def.ToString(ev) + ": return " +
                  enum_name + ".reader.vTableGetNullable(_bc, _bcOffset, " +
                  NumToString(field.value.offset) + ");\n";
        }
        code += "      default: return null;\n";
        code += "    }\n";
        code += "  }\n";
      } else {
        code += " => ";
        if (field.value.type.enum_def &&
            field.value.type.base_type != BASE_TYPE_VECTOR) {
          code += GenDartTypeName(field.value.type,
                                  struct_def.defined_namespace, field) +
                  (isNullable ? "._createOrNull(" : ".fromValue(");
        }

        code += GenReaderTypeName(field.value.type,
                                  struct_def.defined_namespace, field);
        if (struct_def.fixed) {
          code +=
              ".read(_bc, _bcOffset + " + NumToString(field.value.offset) + ")";
        } else {
          code += ".vTableGet";
          std::string offset = NumToString(field.value.offset);
          if (isNullable) {
            code += "Nullable(_bc, _bcOffset, " + offset + ")";
          } else {
            code += "(_bc, _bcOffset, " + offset + ", " + defaultValue + ")";
          }
        }
        if (field.value.type.enum_def &&
            field.value.type.base_type != BASE_TYPE_VECTOR) {
          code += ")";
        }
        code += ";\n";
      }
    }

    code += "\n";
    code += GenToString(namer_.Type(struct_def), non_deprecated_fields);
  }

  std::string GenToString(
      const std::string &object_name,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields) {
    std::string code;
    code += "  @override\n";
    code += "  String toString() {\n";
    code += "    return '" + object_name + "{";
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const std::string field = namer_.Field(*it->second);
      // We need to escape the fact that some fields have $ in the name which is
      // also used in symbol/string substitution.
      std::string escaped_field;
      for (size_t i = 0; i < field.size(); i++) {
        if (field[i] == '$') escaped_field.push_back('\\');
        escaped_field.push_back(field[i]);
      }
      code += escaped_field + ": ${" + field + "}";
      if (it != non_deprecated_fields.end() - 1) { code += ", "; }
    }
    code += "}';\n";
    code += "  }\n";
    return code;
  }

  std::string getDefaultValue(const Value &value) const {
    if (!value.constant.empty() && value.constant != "0") {
      if (IsBool(value.type.base_type)) { return "true"; }
      if (IsScalar(value.type.base_type)) {
        if (StringIsFlatbufferNan(value.constant)) {
          return "double.nan";
        } else if (StringIsFlatbufferPositiveInfinity(value.constant)) {
          return "double.infinity";
        } else if (StringIsFlatbufferNegativeInfinity(value.constant)) {
          return "double.negativeInfinity";
        }
      }
      return value.constant;
    } else if (IsBool(value.type.base_type)) {
      return "false";
    } else if (IsScalar(value.type.base_type) && !IsUnion(value.type)) {
      return "0";
    } else {
      return "";
    }
  }

  void GenReader(const StructDef &struct_def, const std::string &reader_name,
                 std::string &code) {
    const auto struct_type = namer_.Type(struct_def);

    code += "class " + reader_name + " extends " + _kFb;
    if (struct_def.fixed) {
      code += ".StructReader<";
    } else {
      code += ".TableReader<";
    }
    code += struct_type + "> {\n";
    code += "  const " + reader_name + "();\n\n";

    if (struct_def.fixed) {
      code += "  @override\n";
      code += "  int get size => " + NumToString(struct_def.bytesize) + ";\n\n";
    }
    code += "  @override\n";
    code += "  " + struct_type +
            " createObject(fb.BufferContext bc, int offset) => \n    " +
            struct_type + "._(bc, offset);\n";
    code += "}\n\n";
  }

  void GenBuilder(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      const std::string &builder_name, std::string &code) {
    if (non_deprecated_fields.size() == 0) { return; }

    code += "class " + builder_name + " {\n";
    code += "  " + builder_name + "(this.fbBuilder);\n\n";
    code += "  final " + _kFb + ".Builder fbBuilder;\n\n";

    if (struct_def.fixed) {
      StructBuilderBody(struct_def, non_deprecated_fields, code);
    } else {
      TableBuilderBody(struct_def, non_deprecated_fields, code);
    }

    code += "}\n\n";
  }

  void StructBuilderBody(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      std::string &code) {
    code += "  int finish(";
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;
      const std::string field_name = namer_.Field(field);

      if (IsStruct(field.value.type)) {
        code += "fb.StructBuilder";
      } else {
        code += GenDartTypeName(field.value.type, struct_def.defined_namespace,
                                field);
      }
      code += " " + field_name;
      if (it != non_deprecated_fields.end() - 1) { code += ", "; }
    }
    code += ") {\n";

    for (auto it = non_deprecated_fields.rbegin();
         it != non_deprecated_fields.rend(); ++it) {
      const FieldDef &field = *it->second;
      const std::string field_name = namer_.Field(field);

      if (field.padding) {
        code += "    fbBuilder.pad(" + NumToString(field.padding) + ");\n";
      }

      if (IsStruct(field.value.type)) {
        code += "    " + field_name + "();\n";
      } else {
        code += "    fbBuilder.put" + GenType(field.value.type) + "(";
        code += field_name;
        if (field.value.type.enum_def) { code += ".value"; }
        code += ");\n";
      }
    }
    code += "    return fbBuilder.offset;\n";
    code += "  }\n\n";
  }

  void TableBuilderBody(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      std::string &code) {
    code += "  void begin() {\n";
    code += "    fbBuilder.startTable(" +
            NumToString(struct_def.fields.vec.size()) + ");\n";
    code += "  }\n\n";

    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const auto &field = *it->second;
      const auto offset = it->first;
      const std::string add_field = namer_.Method("add", field);
      const std::string field_var = namer_.Variable(field);

      if (IsScalar(field.value.type.base_type)) {
        code += "  int " + add_field + "(";
        code += GenDartTypeName(field.value.type, struct_def.defined_namespace,
                                field);
        code += "? " + field_var + ") {\n";
        code += "    fbBuilder.add" + GenType(field.value.type) + "(" +
                NumToString(offset) + ", ";
        code += field_var;
        if (field.value.type.enum_def) { code += "?.value"; }
        code += ");\n";
      } else if (IsStruct(field.value.type)) {
        code += "  int " + add_field + "(int offset) {\n";
        code +=
            "    fbBuilder.addStruct(" + NumToString(offset) + ", offset);\n";
      } else {
        code += "  int " + add_field + "Offset(int? offset) {\n";
        code +=
            "    fbBuilder.addOffset(" + NumToString(offset) + ", offset);\n";
      }
      code += "    return fbBuilder.offset;\n";
      code += "  }\n";
    }

    code += "\n";
    code += "  int finish() {\n";
    code += "    return fbBuilder.endTable();\n";
    code += "  }\n";
  }

  void GenObjectBuilder(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      const std::string &builder_name, std::string &code) {
    code += "class " + builder_name + " extends " + _kFb + ".ObjectBuilder {\n";
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;

      code += "  final " +
              GenDartTypeName(field.value.type, struct_def.defined_namespace,
                              field, !struct_def.fixed, "ObjectBuilder") +
              " _" + namer_.Variable(field) + ";\n";
    }
    code += "\n";
    code += "  " + builder_name + "(";

    if (non_deprecated_fields.size() != 0) {
      code += "{\n";
      for (auto it = non_deprecated_fields.begin();
           it != non_deprecated_fields.end(); ++it) {
        const FieldDef &field = *it->second;

        code += "    ";
        code += (struct_def.fixed ? "required " : "") +
                GenDartTypeName(field.value.type, struct_def.defined_namespace,
                                field, !struct_def.fixed, "ObjectBuilder") +
                " " + namer_.Variable(field) + ",\n";
      }
      code += "  })\n";
      code += "      : ";
      for (auto it = non_deprecated_fields.begin();
           it != non_deprecated_fields.end(); ++it) {
        const FieldDef &field = *it->second;

        code += "_" + namer_.Variable(field) + " = " + namer_.Variable(field);
        if (it == non_deprecated_fields.end() - 1) {
          code += ";\n\n";
        } else {
          code += ",\n        ";
        }
      }
    } else {
      code += ");\n\n";
    }

    code += "  /// Finish building, and store into the [fbBuilder].\n";
    code += "  @override\n";
    code += "  int finish(" + _kFb + ".Builder fbBuilder) {\n";
    code += GenObjectBuilderImplementation(struct_def, non_deprecated_fields);
    code += "  }\n\n";

    code += "  /// Convenience method to serialize to byte list.\n";
    code += "  @override\n";
    code += "  Uint8List toBytes([String? fileIdentifier]) {\n";
    code += "    final fbBuilder = " + _kFb +
            ".Builder(deduplicateTables: false);\n";
    code += "    fbBuilder.finish(finish(fbBuilder), fileIdentifier);\n";
    code += "    return fbBuilder.buffer;\n";
    code += "  }\n";
    code += "}\n";
  }

  std::string GenObjectBuilderImplementation(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      bool prependUnderscore = true, bool pack = false) {
    std::string code;
    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;

      if (IsScalar(field.value.type.base_type) || IsStruct(field.value.type))
        continue;

      std::string offset_name = namer_.Variable(field) + "Offset";
      std::string field_name =
          (prependUnderscore ? "_" : "") + namer_.Variable(field);
      // custom handling for fixed-sized struct in pack()
      if (pack && IsVector(field.value.type) &&
          field.value.type.VectorType().base_type == BASE_TYPE_STRUCT &&
          field.value.type.struct_def->fixed) {
        code += "    int? " + offset_name + ";\n";
        code += "    if (" + field_name + " != null) {\n";
        code +=
            "      for (var e in " + field_name + "!) { e.pack(fbBuilder); }\n";
        code += "      " + namer_.Variable(field) +
                "Offset = fbBuilder.endStructVector(" + field_name +
                "!.length);\n";
        code += "    }\n";
        continue;
      }

      code += "    final int? " + offset_name;
      if (IsVector(field.value.type)) {
        code += " = " + field_name + " == null ? null\n";
        code += "        : fbBuilder.writeList";
        switch (field.value.type.VectorType().base_type) {
          case BASE_TYPE_STRING:
            code +=
                "(" + field_name + "!.map(fbBuilder.writeString).toList());\n";
            break;
          case BASE_TYPE_STRUCT:
            if (field.value.type.struct_def->fixed) {
              code += "OfStructs(" + field_name + "!);\n";
            } else {
              code += "(" + field_name + "!.map((b) => b." +
                      (pack ? "pack" : "getOrCreateOffset") +
                      "(fbBuilder)).toList());\n";
            }
            break;
          default:
            code +=
                GenType(field.value.type.VectorType()) + "(" + field_name + "!";
            if (field.value.type.enum_def) {
              code += ".map((f) => f.value).toList()";
            }
            code += ");\n";
        }
      } else if (IsString(field.value.type)) {
        code += " = " + field_name + " == null ? null\n";
        code += "        : fbBuilder.writeString(" + field_name + "!);\n";
      } else {
        code += " = " + field_name + "?." +
                (pack ? "pack" : "getOrCreateOffset") + "(fbBuilder);\n";
      }
    }

    if (struct_def.fixed) {
      code += StructObjectBuilderBody(non_deprecated_fields, prependUnderscore,
                                      pack);
    } else {
      code += TableObjectBuilderBody(struct_def, non_deprecated_fields,
                                     prependUnderscore, pack);
    }
    return code;
  }

  std::string StructObjectBuilderBody(
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      bool prependUnderscore = true, bool pack = false) {
    std::string code;

    for (auto it = non_deprecated_fields.rbegin();
         it != non_deprecated_fields.rend(); ++it) {
      const FieldDef &field = *it->second;
      const std::string field_name = namer_.Field(field);

      if (field.padding) {
        code += "    fbBuilder.pad(" + NumToString(field.padding) + ");\n";
      }

      if (IsStruct(field.value.type)) {
        code += "    ";
        if (prependUnderscore) { code += "_"; }
        code += field_name + (pack ? ".pack" : ".finish") + "(fbBuilder);\n";
      } else {
        code += "    fbBuilder.put" + GenType(field.value.type) + "(";
        if (prependUnderscore) { code += "_"; }
        code += field_name;
        if (field.value.type.enum_def) { code += ".value"; }
        code += ");\n";
      }
    }

    code += "    return fbBuilder.offset;\n";
    return code;
  }

  std::string TableObjectBuilderBody(
      const StructDef &struct_def,
      const std::vector<std::pair<int, FieldDef *>> &non_deprecated_fields,
      bool prependUnderscore = true, bool pack = false) {
    std::string code;
    code += "    fbBuilder.startTable(" +
            NumToString(struct_def.fields.vec.size()) + ");\n";

    for (auto it = non_deprecated_fields.begin();
         it != non_deprecated_fields.end(); ++it) {
      const FieldDef &field = *it->second;
      auto offset = it->first;

      std::string field_var =
          (prependUnderscore ? "_" : "") + namer_.Variable(field);

      if (IsScalar(field.value.type.base_type)) {
        code += "    fbBuilder.add" + GenType(field.value.type) + "(" +
                NumToString(offset) + ", " + field_var;
        if (field.value.type.enum_def) {
          bool isNullable = getDefaultValue(field.value).empty();
          code += (isNullable || !pack) ? "?.value" : ".value";
        }
        code += ");\n";
      } else if (IsStruct(field.value.type)) {
        code += "    if (" + field_var + " != null) {\n";
        code += "      fbBuilder.addStruct(" + NumToString(offset) + ", " +
                field_var + (pack ? "!.pack" : "!.finish") + "(fbBuilder));\n";
        code += "    }\n";
      } else {
        code += "    fbBuilder.addOffset(" + NumToString(offset) + ", " +
                namer_.Variable(field) + "Offset);\n";
      }
    }
    code += "    return fbBuilder.endTable();\n";
    return code;
  }

  const IdlNamer namer_;
};
}  // namespace dart

static bool GenerateDart(const Parser &parser, const std::string &path,
                         const std::string &file_name) {
  dart::DartGenerator generator(parser, path, file_name);
  return generator.generate();
}

static std::string DartMakeRule(const Parser &parser, const std::string &path,
                                const std::string &file_name) {
  auto filebase =
      flatbuffers::StripPath(flatbuffers::StripExtension(file_name));
  dart::DartGenerator generator(parser, path, file_name);
  auto make_rule = generator.Filename("") + ": ";

  auto included_files = parser.GetIncludedFilesRecursive(file_name);
  for (auto it = included_files.begin(); it != included_files.end(); ++it) {
    make_rule += " " + *it;
  }
  return make_rule;
}

namespace {

class DartCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateDart(parser, path, filename)) { return Status::ERROR; }
    return Status::OK;
  }

  Status GenerateCode(const uint8_t *, int64_t,
                      const CodeGenOptions &) override {
    return Status::NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    output = DartMakeRule(parser, path, filename);
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

  IDLOptions::Language Language() const override { return IDLOptions::kDart; }

  std::string LanguageName() const override { return "Dart"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewDartCodeGenerator() {
  return std::unique_ptr<DartCodeGenerator>(new DartCodeGenerator());
}

}  // namespace flatbuffers
