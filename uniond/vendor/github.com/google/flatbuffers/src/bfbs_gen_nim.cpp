/*
 * Copyright 2021 Google Inc. All rights reserved.
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

#include "bfbs_gen_nim.h"

#include <cstdint>
#include <map>
#include <memory>
#include <string>
#include <unordered_set>
#include <vector>

// Ensure no includes to flatc internals. bfbs_gen.h and generator.h are OK.
#include "bfbs_gen.h"
#include "bfbs_namer.h"

// The intermediate representation schema.
#include "flatbuffers/code_generator.h"
#include "flatbuffers/reflection.h"
#include "flatbuffers/reflection_generated.h"

namespace flatbuffers {
namespace {

// To reduce typing
namespace r = ::reflection;

std::set<std::string> NimKeywords() {
  return {
    "addr",      "and",     "as",        "asm",      "bind",   "block",
    "break",     "case",    "cast",      "concept",  "const",  "continue",
    "converter", "defer",   "discard",   "distinct", "div",    "do",
    "elif",      "else",    "end",       "enum",     "except", "export",
    "finally",   "for",     "from",      "func",     "if",     "import",
    "in",        "include", "interface", "is",       "isnot",  "iterator",
    "let",       "macro",   "method",    "mixin",    "mod",    "nil",
    "not",       "notin",   "object",    "of",       "or",     "out",
    "proc",      "ptr",     "raise",     "ref",      "return", "shl",
    "shr",       "static",  "template",  "try",      "tuple",  "type",
    "using",     "var",     "when",      "while",    "xor",    "yield",
  };
}

Namer::Config NimDefaultConfig() {
  return { /*types=*/Case::kUpperCamel,
           /*constants=*/Case::kUpperCamel,
           /*methods=*/Case::kLowerCamel,
           /*functions=*/Case::kUpperCamel,
           /*fields=*/Case::kLowerCamel,
           /*variable=*/Case::kLowerCamel,
           /*variants=*/Case::kUpperCamel,
           /*enum_variant_seperator=*/".",
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kKeep,
           /*namespace_seperator=*/"/",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kKeep,
           /*directories=*/Case::kKeep,
           /*output_path=*/"",
           /*filename_suffix=*/"",
           /*filename_extension=*/".nim" };
}

const std::string Export = "*";
const std::set<std::string> builtin_types = {
  "uint8",   "uint8",  "bool",   "int8",  "uint8",   "int16",
  "uint16",  "int32",  "uint32", "int64", "uint64",  "float32",
  "float64", "string", "int",    "uint",  "uoffset", "Builder"
};

class NimBfbsGenerator : public BaseBfbsGenerator {
 public:
  explicit NimBfbsGenerator(const std::string &flatc_version)
      : BaseBfbsGenerator(),
        keywords_(),
        imports_(),
        current_obj_(nullptr),
        current_enum_(nullptr),
        flatc_version_(flatc_version),
        namer_(NimDefaultConfig(), NimKeywords()) {}

  Status GenerateFromSchema(const r::Schema *schema,
                            const CodeGenOptions &options)
      FLATBUFFERS_OVERRIDE {
    options_ = options;
    ForAllEnums(schema->enums(), [&](const r::Enum *enum_def) {
      StartCodeBlock(enum_def);
      GenerateEnum(enum_def);
    });
    ForAllObjects(schema->objects(), [&](const r::Object *object) {
      StartCodeBlock(object);
      GenerateObject(object);
    });
    return OK;
  }

  using BaseBfbsGenerator::GenerateCode;

  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    (void)parser;
    (void)path;
    (void)filename;
    return NOT_IMPLEMENTED;
  }

  Status GenerateMakeRule(const Parser &parser, const std::string &path,
                          const std::string &filename,
                          std::string &output) override {
    (void)parser;
    (void)path;
    (void)filename;
    (void)output;
    return NOT_IMPLEMENTED;
  }

  Status GenerateGrpcCode(const Parser &parser, const std::string &path,
                          const std::string &filename) override {
    (void)parser;
    (void)path;
    (void)filename;
    return NOT_IMPLEMENTED;
  }

  Status GenerateRootFile(const Parser &parser,
                          const std::string &path) override {
    (void)parser;
    (void)path;
    return NOT_IMPLEMENTED;
  }

  bool IsSchemaOnly() const override { return true; }

  bool SupportsBfbsGeneration() const override { return true; }

  bool SupportsRootFileGeneration() const override { return false; }

  IDLOptions::Language Language() const override { return IDLOptions::kNim; }

  std::string LanguageName() const override { return "Nim"; }

  uint64_t SupportedAdvancedFeatures() const FLATBUFFERS_OVERRIDE {
    return r::AdvancedArrayFeatures | r::AdvancedUnionFeatures |
           r::OptionalScalars | r::DefaultVectorsAndStrings;
  }

 protected:
  void GenerateEnum(const r::Enum *enum_def) {
    std::string code;

    std::string ns;
    const std::string enum_name = namer_.Type(namer_.Denamespace(enum_def, ns));
    const std::string enum_type =
        GenerateTypeBasic(enum_def->underlying_type());

    GenerateDocumentation(enum_def->documentation(), "", code);
    code += "type " + enum_name + Export + "{.pure.} = enum\n";

    ForAllEnumValues(enum_def, [&](const reflection::EnumVal *enum_val) {
      GenerateDocumentation(enum_val->documentation(), "  ", code);
      code += "  " + namer_.Variant(enum_val->name()->str()) + " = " +
              NumToString(enum_val->value()) + "." + enum_type + ",\n";
    });

    EmitCodeBlock(code, enum_name, ns, enum_def->declaration_file()->str());
  }

  void GenerateObject(const r::Object *object) {
    // Register the main flatbuffers module.
    RegisterImports("flatbuffers", "");
    std::string code;

    std::string ns;
    const std::string object_name = namer_.Type(namer_.Denamespace(object, ns));

    GenerateDocumentation(object->documentation(), "", code);
    code += "type " + object_name + "* = object of FlatObj\n";

    // Create all the field accessors.
    ForAllFields(object, /*reverse=*/false, [&](const r::Field *field) {
      // Skip writing deprecated fields altogether.
      if (field->deprecated()) { return; }

      const std::string field_name = namer_.Field(*field);
      const r::BaseType base_type = field->type()->base_type();
      std::string field_type = GenerateType(field->type());

      if (field->optional() && !object->is_struct()) {
        RegisterImports("std/options", "");
        field_type = "Option[" + field_type + "]";
      }

      const std::string offset_prefix =
          "let o = self.tab.Offset(" + NumToString(field->offset()) + ")\n";
      const std::string offset_prefix_2 = "if o != 0:\n";

      if (IsScalar(base_type) || base_type == r::String ||
          base_type == r::Obj || base_type == r::Union) {
        GenerateDocumentation(field->documentation(), "", code);

        std::string getter_signature = "func " + namer_.Method(field_name) +
                                       "*(self: " + object_name +
                                       "): " + field_type + " =\n";
        std::string getter_code;
        std::string setter_signature =
            "func `" + namer_.Method(field_name + "=") + "`*(self: var " +
            object_name + ", n: " + field_type + "): bool =\n";
        std::string setter_code;

        if (base_type == r::Obj || base_type == r::Union ||
            field->type()->index() >= 0) {
          RegisterImports(object, field);
        }

        if (object->is_struct()) {
          std::string field_getter =
              GenerateGetter(field->type(), NumToString(field->offset()));
          getter_code += "  return " + field_getter + "\n";

          if (IsScalar(base_type)) {
            setter_code += "  return self.tab.Mutate(self.tab.Pos + " +
                           NumToString(field->offset()) + ", n)\n";
          }
        } else {
          // Table accessors
          getter_code += "  " + offset_prefix;
          getter_code += "  " + offset_prefix_2;
          std::string field_getter = GenerateGetter(field->type(), "o");
          if (field->optional()) {
            field_getter = "some(" + field_getter + ")";
          }
          getter_code += "    return " + field_getter + "\n";
          if (!field->optional()) {
            getter_code += "  return " + DefaultValue(field) + "\n";
          }

          if (IsScalar(base_type)) {
            setter_code += "  return self.tab.MutateSlot(" +
                           NumToString(field->offset()) + ", n)\n";
          }
        }
        code += getter_signature + getter_code;
        if (IsScalar(base_type)) { code += setter_signature + setter_code; }
      } else if (base_type == r::Array || base_type == r::Vector) {
        const r::BaseType vector_base_type = field->type()->element();
        uint32_t element_size = field->type()->element_size();

        if (vector_base_type == r::Obj || vector_base_type == r::Union ||
            field->type()->index() >= 0) {
          RegisterImports(object, field, true);
        }

        // Get vector length:
        code += "func " + namer_.Method(field_name + "Length") +
                "*(self: " + object_name + "): int = \n";
        code += "  " + offset_prefix;
        code += "  " + offset_prefix_2;
        code += "    return self.tab.VectorLen(o)\n";

        // Get single vector field:
        code += "func " + namer_.Method(field_name) + "*(self: " + object_name +
                ", j: int): " + GenerateType(field->type(), true) + " = \n";
        code += "  " + offset_prefix;
        code += "  " + offset_prefix_2;
        code += "    var x = self.tab.Vector(o)\n";
        code +=
            "    x += j.uoffset * " + NumToString(element_size) + ".uoffset\n";
        code += "    return " + GenerateGetter(field->type(), "x", true) + "\n";

        // Get entire vector:
        code += "func " + namer_.Method(field_name) + "*(self: " + object_name +
                "): " + GenerateType(field->type()) + " = \n";
        code += "  let len = self." + field_name + "Length\n";
        code += "  for i in countup(0, len - 1):\n";
        code += "    result.add(self." + field_name + "(i))\n";

        (void)IsSingleByte(vector_base_type);  // unnused function warning
      }
    });

    // Create all the builders
    if (object->is_struct()) {
      code += "proc " + namer_.Function(object_name + "Create") +
              "*(self: var Builder";
      code += GenerateStructBuilderArgs(object);
      code += "): uoffset =\n";
      code += AppendStructBuilderBody(object);
      code += "  return self.Offset()\n";
    } else {
      // Table builders
      code += "proc " + namer_.Function(object_name + "Start") +
              "*(builder: var Builder) =\n";
      code += "  builder.StartObject(" + NumToString(object->fields()->size()) +
              ")\n";

      ForAllFields(object, /*reverse=*/false, [&](const r::Field *field) {
        if (field->deprecated()) { return; }

        const std::string field_name = namer_.Field(*field);
        const std::string variable_name = namer_.Variable(*field);
        const std::string variable_type = GenerateTypeBasic(field->type());

        code += "proc " + namer_.Function(object_name + "Add" + field_name) +
                "*(builder: var Builder, " + variable_name + ": " +
                variable_type + ") =\n";
        code += "  builder.Prepend" + GenerateMethod(field) + "Slot(" +
                NumToString(field->id()) + ", " + variable_name + ", default(" +
                variable_type + "))\n";

        if (IsVector(field->type()->base_type())) {
          code += "proc " +
                  namer_.Function(object_name + "Start" + field_name) +
                  "Vector*(builder: var Builder, numElems: uoffset) =\n";

          const int32_t element_size = field->type()->element_size();
          int32_t alignment = element_size;
          if (IsStruct(field->type(), /*use_element=*/true)) {
            alignment = GetObjectByIndex(field->type()->index())->minalign();
          }

          code += "  builder.StartVector(" + NumToString(element_size) +
                  ", numElems, " + NumToString(alignment) + ")\n";
        }
      });

      code += "proc " + namer_.Function(object_name + "End") +
              "*(builder: var Builder): uoffset =\n";
      code += "  return builder.EndObject()\n";
    }
    EmitCodeBlock(code, object_name, ns, object->declaration_file()->str());
  }

 private:
  void GenerateDocumentation(
      const flatbuffers::Vector<flatbuffers::Offset<flatbuffers::String>>
          *documentation,
      std::string indent, std::string &code) const {
    flatbuffers::ForAllDocumentation(
        documentation, [&](const flatbuffers::String *str) {
          code += indent + "# " + str->str() + "\n";
        });
  }

  std::string GenerateStructBuilderArgs(const r::Object *object,
                                        std::string prefix = "") const {
    std::string signature;
    ForAllFields(object, /*reverse=*/false, [&](const r::Field *field) {
      if (IsStructOrTable(field->type()->base_type())) {
        const r::Object *field_object = GetObject(field->type());
        signature += GenerateStructBuilderArgs(
            field_object, prefix + namer_.Variable(*field) + "_");
      } else {
        signature += ", " + prefix + namer_.Variable(*field) + ": " +
                     GenerateType(field->type());
      }
    });
    return signature;
  }

  std::string AppendStructBuilderBody(const r::Object *object,
                                      std::string prefix = "") const {
    std::string code;
    code += "  self.Prep(" + NumToString(object->minalign()) + ", " +
            NumToString(object->bytesize()) + ")\n";

    // We need to reverse the order we iterate over, since we build the
    // buffer backwards.
    ForAllFields(object, /*reverse=*/true, [&](const r::Field *field) {
      const int32_t num_padding_bytes = field->padding();
      if (num_padding_bytes) {
        code += "  self.Pad(" + NumToString(num_padding_bytes) + ")\n";
      }
      if (IsStructOrTable(field->type()->base_type())) {
        const r::Object *field_object = GetObject(field->type());
        code += AppendStructBuilderBody(field_object,
                                        prefix + namer_.Variable(*field) + "_");
      } else {
        code += "  self.Prepend(" + prefix + namer_.Variable(*field) + ")\n";
      }
    });

    return code;
  }

  std::string GenerateMethod(const r::Field *field) const {
    const r::BaseType base_type = field->type()->base_type();
    if (IsStructOrTable(base_type)) { return "Struct"; }
    return "";
  }

  std::string GenerateGetter(const r::Type *type, const std::string &offsetval,
                             bool element_type = false) const {
    const r::BaseType base_type =
        element_type ? type->element() : type->base_type();
    std::string offset = offsetval;
    if (!element_type) { offset = "self.tab.Pos + " + offset; }
    switch (base_type) {
      case r::String: return "self.tab.String(" + offset + ")";
      case r::Union: return "self.tab.Union(" + offsetval + ")";
      case r::Obj: {
        return GenerateType(type, element_type) +
               "(tab: Vtable(Bytes: self.tab.Bytes, Pos: " + offset + "))";
      }
      case r::Vector: return GenerateGetter(type, offsetval, true);
      default:
        const r::Enum *type_enum = GetEnum(type, element_type);
        if (type_enum != nullptr) {
          return GenerateType(type, element_type) + "(" + "Get[" +
                 GenerateType(base_type) + "](self.tab, " + offset + ")" + ")";
        } else {
          return "Get[" + GenerateType(base_type) + "](self.tab, " + offset +
                 ")";
        }
    }
  }

  std::string Denamespace(const std::string &s, std::string &importns,
                          std::string &ns) const {
    if (builtin_types.find(s) != builtin_types.end()) { return s; }
    std::string type = namer_.Type(namer_.Denamespace(s, ns));
    importns = ns.empty() ? type : ns + "." + type;
    std::replace(importns.begin(), importns.end(), '.', '_');
    return type;
  }

  std::string Denamespace(const std::string &s, std::string &importns) const {
    std::string ns;
    return Denamespace(s, importns, ns);
  }

  std::string Denamespace(const std::string &s) const {
    std::string importns;
    return Denamespace(s, importns);
  }

  std::string GenerateType(const r::Type *type, bool element_type = false,
                           bool enum_inner = false) const {
    const r::BaseType base_type =
        element_type ? type->element() : type->base_type();
    if (IsScalar(base_type) && !enum_inner) {
      const r::Enum *type_enum = GetEnum(type, element_type);
      if (type_enum != nullptr) {
        std::string importns;
        std::string type_name = Denamespace(type_enum->name()->str(), importns);
        return importns + "." + type_name;
      }
    }
    if (IsScalar(base_type)) { return Denamespace(GenerateType(base_type)); }
    switch (base_type) {
      case r::String: return "string";
      case r::Vector: {
        return "seq[" + GenerateType(type, true) + "]";
      }
      case r::Union: return "Vtable";
      case r::Obj: {
        const r::Object *type_obj = GetObject(type, element_type);
        std::string importns;
        std::string type_name = Denamespace(type_obj->name()->str(), importns);
        if (type_obj == current_obj_) {
          return type_name;
        } else {
          return importns + "." + type_name;
        }
      }
      default: return "uoffset";
    }
  }

  std::string GenerateTypeBasic(const r::Type *type,
                                bool element_type = false) const {
    const r::BaseType base_type =
        element_type ? type->element() : type->base_type();
    if (IsScalar(base_type)) {
      return GenerateType(base_type);
    } else {
      return "uoffset";
    }
  }

  std::string GenerateType(const r::BaseType base_type) const {
    switch (base_type) {
      case r::None: return "uint8";
      case r::UType: return "uint8";
      case r::Bool: return "bool";
      case r::Byte: return "int8";
      case r::UByte: return "uint8";
      case r::Short: return "int16";
      case r::UShort: return "uint16";
      case r::Int: return "int32";
      case r::UInt: return "uint32";
      case r::Long: return "int64";
      case r::ULong: return "uint64";
      case r::Float: return "float32";
      case r::Double: return "float64";
      case r::String: return "string";
      default: return r::EnumNameBaseType(base_type);
    }
  }

  std::string DefaultValue(const r::Field *field) const {
    const r::BaseType base_type = field->type()->base_type();
    if (IsFloatingPoint(base_type)) {
      if (field->default_real() != field->default_real()) {
        return "NaN";
      } else if (field->default_real() ==
                 std::numeric_limits<double>::infinity()) {
        return "Inf";
      } else if (field->default_real() ==
                 -std::numeric_limits<double>::infinity()) {
        return "-Inf";
      }
      return NumToString(field->default_real());
    }
    if (IsBool(base_type)) {
      return field->default_integer() ? "true" : "false";
    }
    if (IsScalar(base_type)) {
      const r::Enum *type_enum = GetEnum(field->type());
      if (type_enum != nullptr) {
        return "type(result)(" + NumToString((field->default_integer())) + ")";
      }
      return NumToString((field->default_integer()));
    }
    if (base_type == r::String) { return "\"\""; }
    // represents offsets
    return "0";
  }

  void StartCodeBlock(const reflection::Enum *enum_def) {
    current_enum_ = enum_def;
    current_obj_ = nullptr;
    imports_.clear();
  }

  void StartCodeBlock(const reflection::Object *object) {
    current_enum_ = nullptr;
    current_obj_ = object;
    imports_.clear();
  }

  std::vector<std::string> StringSplit(const std::string orig_str,
                                       const std::string token) {
    std::vector<std::string> result;
    std::string str = orig_str;
    while (str.size()) {
      size_t index = str.find(token);
      if (index != std::string::npos) {
        result.push_back(str.substr(0, index));
        str = str.substr(index + token.size());
        if (str.size() == 0) result.push_back(str);
      } else {
        result.push_back(str);
        str = "";
      }
    }
    return result;
  }

  std::string GetRelativePathFromNamespace(const std::string &relative_to,
                                           const std::string &str2) {
    std::vector<std::string> relative_to_vec = StringSplit(relative_to, ".");
    std::vector<std::string> str2_vec = StringSplit(str2, ".");
    while (relative_to_vec.size() > 0 && str2_vec.size() > 0) {
      if (relative_to_vec[0] == str2_vec[0]) {
        relative_to_vec.erase(relative_to_vec.begin());
        str2_vec.erase(str2_vec.begin());
      } else {
        break;
      }
    }
    relative_to_vec.pop_back();
    for (size_t i = 0; i < relative_to_vec.size(); ++i) {
      str2_vec.insert(str2_vec.begin(), std::string(".."));
    }

    std::string new_path;
    for (size_t i = 0; i < str2_vec.size(); ++i) {
      new_path += str2_vec[i];
      if (i != str2_vec.size() - 1) { new_path += "/"; }
    }
    return new_path;
  }

  void RegisterImports(const r::Object *object, const r::Field *field,
                       bool use_element = false) {
    std::string importns;
    std::string type_name;

    const r::BaseType type =
        use_element ? field->type()->element() : field->type()->base_type();

    if (IsStructOrTable(type)) {
      const r::Object *object_def = GetObjectByIndex(field->type()->index());
      if (object_def == current_obj_) { return; }
      std::string ns;
      type_name = Denamespace(object_def->name()->str(), importns, ns);
      type_name = ns.empty() ? type_name : ns + "." + type_name;
    } else {
      const r::Enum *enum_def = GetEnumByIndex(field->type()->index());
      if (enum_def == current_enum_) { return; }
      std::string ns;
      type_name = Denamespace(enum_def->name()->str(), importns, ns);
      type_name = ns.empty() ? type_name : ns + "." + type_name;
    }

    std::string import_path =
        GetRelativePathFromNamespace(object->name()->str(), type_name);
    std::replace(type_name.begin(), type_name.end(), '.', '_');
    RegisterImports(import_path, importns);
  }

  void RegisterImports(const std::string &local_name,
                       const std::string &imports_name) {
    imports_[local_name] = imports_name;
  }

  void EmitCodeBlock(const std::string &code_block, const std::string &name,
                     const std::string &ns, const std::string &declaring_file) {
    const std::string full_qualified_name = ns.empty() ? name : ns + "." + name;

    std::string code = "#[ " + full_qualified_name + "\n";
    code +=
        "  Automatically generated by the FlatBuffers compiler, do not "
        "modify.\n";
    code += "  Or modify. I'm a message, not a cop.\n";
    code += "\n";
    code += "  flatc version: " + flatc_version_ + "\n";
    code += "\n";
    code += "  Declared by  : " + declaring_file + "\n";
    if (schema_->root_table() != nullptr) {
      const std::string root_type = schema_->root_table()->name()->str();
      const std::string root_file =
          schema_->root_table()->declaration_file()->str();
      code += "  Rooting type : " + root_type + " (" + root_file + ")\n";
    }
    code += "]#\n\n";

    if (!imports_.empty()) {
      for (auto it = imports_.cbegin(); it != imports_.cend(); ++it) {
        if (it->second.empty()) {
          code += "import " + it->first + "\n";
        } else {
          code += "import " + it->first + " as " + it->second + "\n";
        }
      }
      code += "\n";
    }
    code += code_block;

    // Namespaces are '.' deliminted, so replace it with the path separator.
    std::string path = ns;

    if (ns.empty()) {
      path = ".";
    } else {
      std::replace(path.begin(), path.end(), '.', '/');
    }

    // TODO(derekbailey): figure out a save file without depending on util.h
    EnsureDirExists(path);
    const std::string file_name =
        options_.output_path + path + "/" + namer_.File(name);
    SaveFile(file_name.c_str(), code, false);
  }

  std::unordered_set<std::string> keywords_;
  std::map<std::string, std::string> imports_;
  CodeGenOptions options_;

  const r::Object *current_obj_;
  const r::Enum *current_enum_;
  const std::string flatc_version_;
  const BfbsNamer namer_;
};
}  // namespace

std::unique_ptr<CodeGenerator> NewNimBfbsGenerator(
    const std::string &flatc_version) {
  return std::unique_ptr<NimBfbsGenerator>(new NimBfbsGenerator(flatc_version));
}

}  // namespace flatbuffers
