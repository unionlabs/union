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

#include "bfbs_gen_lua.h"

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

std::set<std::string> LuaKeywords() {
  return { "and",   "break", "do",       "else", "elseif", "end",
           "false", "for",   "function", "goto", "if",     "in",
           "local", "nil",   "not",      "or",   "repeat", "return",
           "then",  "true",  "until",    "while" };
}

Namer::Config LuaDefaultConfig() {
  return { /*types=*/Case::kUpperCamel,
           /*constants=*/Case::kUnknown,
           /*methods=*/Case::kUpperCamel,
           /*functions=*/Case::kUpperCamel,
           /*fields=*/Case::kUpperCamel,
           /*variables=*/Case::kLowerCamel,
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/"",
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kKeep,
           /*namespace_seperator=*/"__",
           /*object_prefix=*/"",
           /*object_suffix=*/"",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kKeep,
           /*directories=*/Case::kKeep,
           /*output_path=*/"",
           /*filename_suffix=*/"",
           /*filename_extension=*/".lua" };
}

class LuaBfbsGenerator : public BaseBfbsGenerator {
 public:
  explicit LuaBfbsGenerator(const std::string &flatc_version)
      : BaseBfbsGenerator(),
        keywords_(),
        requires_(),
        current_obj_(nullptr),
        current_enum_(nullptr),
        flatc_version_(flatc_version),
        namer_(LuaDefaultConfig(), LuaKeywords()) {}

  Status GenerateFromSchema(const r::Schema *schema,
                            const CodeGenOptions &options)
      FLATBUFFERS_OVERRIDE {
    options_ = options;
    if (!GenerateEnums(schema->enums())) { return ERROR; }
    if (!GenerateObjects(schema->objects(), schema->root_table())) {
      return ERROR;
    }
    return OK;
  }

  using BaseBfbsGenerator::GenerateCode;

  Status GenerateCode(const Parser &, const std::string &,
                      const std::string &) override {
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

  bool SupportsBfbsGeneration() const override { return true; }

  bool SupportsRootFileGeneration() const override { return false; }

  IDLOptions::Language Language() const override { return IDLOptions::kLua; }

  std::string LanguageName() const override { return "Lua"; }

  uint64_t SupportedAdvancedFeatures() const FLATBUFFERS_OVERRIDE {
    return 0xF;
  }

 protected:
  bool GenerateEnums(
      const flatbuffers::Vector<flatbuffers::Offset<r::Enum>> *enums) {
    ForAllEnums(enums, [&](const r::Enum *enum_def) {
      std::string code;

      StartCodeBlock(enum_def);

      std::string ns;
      const std::string enum_name =
          namer_.Type(namer_.Denamespace(enum_def, ns));

      GenerateDocumentation(enum_def->documentation(), "", code);
      code += "local " + enum_name + " = {\n";

      ForAllEnumValues(enum_def, [&](const reflection::EnumVal *enum_val) {
        GenerateDocumentation(enum_val->documentation(), "  ", code);
        code += "  " + namer_.Variant(enum_val->name()->str()) + " = " +
                NumToString(enum_val->value()) + ",\n";
      });
      code += "}\n";
      code += "\n";

      EmitCodeBlock(code, enum_name, ns, enum_def->declaration_file()->str());
    });
    return true;
  }

  bool GenerateObjects(
      const flatbuffers::Vector<flatbuffers::Offset<r::Object>> *objects,
      const r::Object *root_object) {
    ForAllObjects(objects, [&](const r::Object *object) {
      std::string code;

      StartCodeBlock(object);

      // Register the main flatbuffers module.
      RegisterRequires("flatbuffers", "flatbuffers");

      std::string ns;
      const std::string object_name =
          namer_.Type(namer_.Denamespace(object, ns));

      GenerateDocumentation(object->documentation(), "", code);

      code += "local " + object_name + " = {}\n";
      code += "local mt = {}\n";
      code += "\n";
      code += "function " + object_name + ".New()\n";
      code += "  local o = {}\n";
      code += "  setmetatable(o, {__index = mt})\n";
      code += "  return o\n";
      code += "end\n";
      code += "\n";

      if (object == root_object) {
        code += "function " + object_name + ".GetRootAs" + object_name +
                "(buf, offset)\n";
        code += "  if type(buf) == \"string\" then\n";
        code += "    buf = flatbuffers.binaryArray.New(buf)\n";
        code += "  end\n";
        code += "\n";
        code += "  local n = flatbuffers.N.UOffsetT:Unpack(buf, offset)\n";
        code += "  local o = " + object_name + ".New()\n";
        code += "  o:Init(buf, n + offset)\n";
        code += "  return o\n";
        code += "end\n";
        code += "\n";
      }

      // Generates a init method that receives a pre-existing accessor object,
      // so that objects can be reused.

      code += "function mt:Init(buf, pos)\n";
      code += "  self.view = flatbuffers.view.New(buf, pos)\n";
      code += "end\n";
      code += "\n";

      // Create all the field accessors.
      ForAllFields(object, /*reverse=*/false, [&](const r::Field *field) {
        // Skip writing deprecated fields altogether.
        if (field->deprecated()) { return; }

        const std::string field_name = namer_.Field(*field);
        const r::BaseType base_type = field->type()->base_type();

        // Generate some fixed strings so we don't repeat outselves later.
        const std::string getter_signature =
            "function mt:" + field_name + "()\n";
        const std::string offset_prefix = "local o = self.view:Offset(" +
                                          NumToString(field->offset()) + ")\n";
        const std::string offset_prefix_2 = "if o ~= 0 then\n";

        GenerateDocumentation(field->documentation(), "", code);

        if (IsScalar(base_type)) {
          code += getter_signature;

          if (object->is_struct()) {
            // TODO(derekbailey): it would be nice to modify the view:Get to
            // just pass in the offset and not have to add it its own
            // self.view.pos.
            code += "  return " + GenerateGetter(field->type()) +
                    "self.view.pos + " + NumToString(field->offset()) + ")\n";
          } else {
            // Table accessors
            code += "  " + offset_prefix;
            code += "  " + offset_prefix_2;

            std::string getter =
                GenerateGetter(field->type()) + "self.view.pos + o)";
            if (IsBool(base_type)) { getter = "(" + getter + " ~=0)"; }
            code += "    return " + getter + "\n";
            code += "  end\n";
            code += "  return " + DefaultValue(field) + "\n";
          }
          code += "end\n";
          code += "\n";
        } else {
          switch (base_type) {
            case r::String: {
              code += getter_signature;
              code += "  " + offset_prefix;
              code += "  " + offset_prefix_2;
              code += "    return " + GenerateGetter(field->type()) +
                      "self.view.pos + o)\n";
              code += "  end\n";
              code += "end\n";
              code += "\n";
              break;
            }
            case r::Obj: {
              if (object->is_struct()) {
                code += "function mt:" + field_name + "(obj)\n";
                code += "  obj:Init(self.view.bytes, self.view.pos + " +
                        NumToString(field->offset()) + ")\n";
                code += "  return obj\n";
                code += "end\n";
                code += "\n";
              } else {
                code += getter_signature;
                code += "  " + offset_prefix;
                code += "  " + offset_prefix_2;

                const r::Object *field_object = GetObject(field->type());
                if (!field_object) {
                  // TODO(derekbailey): this is an error condition. we
                  // should report it better.
                  return;
                }
                code += "    local x = " +
                        std::string(
                            field_object->is_struct()
                                ? "self.view.pos + o\n"
                                : "self.view:Indirect(self.view.pos + o)\n");
                const std::string require_name = RegisterRequires(field);
                code += "    local obj = " + require_name + ".New()\n";
                code += "    obj:Init(self.view.bytes, x)\n";
                code += "    return obj\n";
                code += "  end\n";
                code += "end\n";
                code += "\n";
              }
              break;
            }
            case r::Union: {
              code += getter_signature;
              code += "  " + offset_prefix;
              code += "  " + offset_prefix_2;
              code +=
                  "   local obj = "
                  "flatbuffers.view.New(flatbuffers.binaryArray.New("
                  "0), 0)\n";
              code += "    " + GenerateGetter(field->type()) + "obj, o)\n";
              code += "    return obj\n";
              code += "  end\n";
              code += "end\n";
              code += "\n";
              break;
            }
            case r::Array:
            case r::Vector: {
              const r::BaseType vector_base_type = field->type()->element();
              int32_t element_size = field->type()->element_size();
              code += "function mt:" + field_name + "(j)\n";
              code += "  " + offset_prefix;
              code += "  " + offset_prefix_2;

              if (IsStructOrTable(vector_base_type)) {
                code += "    local x = self.view:Vector(o)\n";
                code +=
                    "    x = x + ((j-1) * " + NumToString(element_size) + ")\n";
                if (IsTable(field->type(), /*use_element=*/true)) {
                  code += "    x = self.view:Indirect(x)\n";
                } else {
                  // Vector of structs are inline, so we need to query the
                  // size of the struct.
                  const reflection::Object *obj =
                      GetObjectByIndex(field->type()->index());
                  element_size = obj->bytesize();
                }

                // Include the referenced type, thus we need to make sure
                // we set `use_element` to true.
                const std::string require_name =
                    RegisterRequires(field, /*use_element=*/true);
                code += "    local obj = " + require_name + ".New()\n";
                code += "    obj:Init(self.view.bytes, x)\n";
                code += "    return obj\n";
              } else {
                code += "    local a = self.view:Vector(o)\n";
                code += "    return " + GenerateGetter(field->type()) +
                        "a + ((j-1) * " + NumToString(element_size) + "))\n";
              }
              code += "  end\n";
              // Only generate a default value for those types that are
              // supported.
              if (!IsStructOrTable(vector_base_type)) {
                code +=
                    "  return " +
                    std::string(vector_base_type == r::String ? "''\n" : "0\n");
              }
              code += "end\n";
              code += "\n";

              // If the vector is composed of single byte values, we
              // generate a helper function to get it as a byte string in
              // Lua.
              if (IsSingleByte(vector_base_type)) {
                code += "function mt:" + field_name + "AsString(start, stop)\n";
                code += "  return self.view:VectorAsString(" +
                        NumToString(field->offset()) + ", start, stop)\n";
                code += "end\n";
                code += "\n";
              }

              // We also make a new accessor to query just the length of the
              // vector.
              code += "function mt:" + field_name + "Length()\n";
              code += "  " + offset_prefix;
              code += "  " + offset_prefix_2;
              code += "    return self.view:VectorLen(o)\n";
              code += "  end\n";
              code += "  return 0\n";
              code += "end\n";
              code += "\n";
              break;
            }
            default: {
              return;
            }
          }
        }
        return;
      });

      // Create all the builders
      if (object->is_struct()) {
        code += "function " + object_name + ".Create" + object_name +
                "(builder" + GenerateStructBuilderArgs(object) + ")\n";
        code += AppendStructBuilderBody(object);
        code += "  return builder:Offset()\n";
        code += "end\n";
        code += "\n";
      } else {
        // Table builders
        code += "function " + object_name + ".Start(builder)\n";
        code += "  builder:StartObject(" +
                NumToString(object->fields()->size()) + ")\n";
        code += "end\n";
        code += "\n";

        ForAllFields(object, /*reverse=*/false, [&](const r::Field *field) {
          if (field->deprecated()) { return; }

          const std::string field_name = namer_.Field(*field);
          const std::string variable_name = namer_.Variable(*field);

          code += "function " + object_name + ".Add" + field_name +
                  "(builder, " + variable_name + ")\n";
          code += "  builder:Prepend" + GenerateMethod(field) + "Slot(" +
                  NumToString(field->id()) + ", " + variable_name + ", " +
                  DefaultValue(field) + ")\n";
          code += "end\n";
          code += "\n";

          if (IsVector(field->type()->base_type())) {
            code += "function " + object_name + ".Start" + field_name +
                    "Vector(builder, numElems)\n";

            const int32_t element_size = field->type()->element_size();
            int32_t alignment = 0;
            if (IsStruct(field->type(), /*use_element=*/true)) {
              alignment = GetObjectByIndex(field->type()->index())->minalign();
            } else {
              alignment = element_size;
            }

            code += "  return builder:StartVector(" +
                    NumToString(element_size) + ", numElems, " +
                    NumToString(alignment) + ")\n";
            code += "end\n";
            code += "\n";
          }
        });

        code += "function " + object_name + ".End(builder)\n";
        code += "  return builder:EndObject()\n";
        code += "end\n";
        code += "\n";
      }

      EmitCodeBlock(code, object_name, ns, object->declaration_file()->str());
    });
    return true;
  }

 private:
  void GenerateDocumentation(
      const flatbuffers::Vector<flatbuffers::Offset<flatbuffers::String>>
          *documentation,
      std::string indent, std::string &code) const {
    flatbuffers::ForAllDocumentation(
        documentation, [&](const flatbuffers::String *str) {
          code += indent + "--" + str->str() + "\n";
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
        signature += ", " + prefix + namer_.Variable(*field);
      }
    });
    return signature;
  }

  std::string AppendStructBuilderBody(const r::Object *object,
                                      std::string prefix = "") const {
    std::string code;
    code += "  builder:Prep(" + NumToString(object->minalign()) + ", " +
            NumToString(object->bytesize()) + ")\n";

    // We need to reverse the order we iterate over, since we build the
    // buffer backwards.
    ForAllFields(object, /*reverse=*/true, [&](const r::Field *field) {
      const int32_t num_padding_bytes = field->padding();
      if (num_padding_bytes) {
        code += "  builder:Pad(" + NumToString(num_padding_bytes) + ")\n";
      }
      if (IsStructOrTable(field->type()->base_type())) {
        const r::Object *field_object = GetObject(field->type());
        code += AppendStructBuilderBody(field_object,
                                        prefix + namer_.Variable(*field) + "_");
      } else {
        code += "  builder:Prepend" + GenerateMethod(field) + "(" + prefix +
                namer_.Variable(*field) + ")\n";
      }
    });

    return code;
  }

  std::string GenerateMethod(const r::Field *field) const {
    const r::BaseType base_type = field->type()->base_type();
    if (IsScalar(base_type)) { return namer_.Type(GenerateType(base_type)); }
    if (IsStructOrTable(base_type)) { return "Struct"; }
    return "UOffsetTRelative";
  }

  std::string GenerateGetter(const r::Type *type,
                             bool element_type = false) const {
    switch (element_type ? type->element() : type->base_type()) {
      case r::String: return "self.view:String(";
      case r::Union: return "self.view:Union(";
      case r::Vector: return GenerateGetter(type, true);
      default:
        return "self.view:Get(flatbuffers.N." +
               namer_.Type(GenerateType(type, element_type)) + ", ";
    }
  }

  std::string GenerateType(const r::Type *type,
                           bool element_type = false) const {
    const r::BaseType base_type =
        element_type ? type->element() : type->base_type();
    if (IsScalar(base_type)) { return GenerateType(base_type); }
    switch (base_type) {
      case r::String: return "string";
      case r::Vector: return GenerateGetter(type, true);
      case r::Obj: return namer_.Type(namer_.Denamespace(GetObject(type)));

      default: return "*flatbuffers.Table";
    }
  }

  std::string GenerateType(const r::BaseType base_type) const {
    // Need to override the default naming to match the Lua runtime libraries.
    // TODO(derekbailey): make overloads in the runtime libraries to avoid this.
    switch (base_type) {
      case r::None: return "uint8";
      case r::UType: return "uint8";
      case r::Byte: return "int8";
      case r::UByte: return "uint8";
      case r::Short: return "int16";
      case r::UShort: return "uint16";
      case r::Int: return "int32";
      case r::UInt: return "uint32";
      case r::Long: return "int64";
      case r::ULong: return "uint64";
      case r::Float: return "Float32";
      case r::Double: return "Float64";
      default: return r::EnumNameBaseType(base_type);
    }
  }

  std::string DefaultValue(const r::Field *field) const {
    const r::BaseType base_type = field->type()->base_type();
    if (IsFloatingPoint(base_type)) {
      return NumToString(field->default_real());
    }
    if (IsBool(base_type)) {
      return field->default_integer() ? "true" : "false";
    }
    if (IsScalar(base_type)) { return NumToString((field->default_integer())); }
    // represents offsets
    return "0";
  }

  void StartCodeBlock(const reflection::Enum *enum_def) {
    current_enum_ = enum_def;
    current_obj_ = nullptr;
    requires_.clear();
  }

  void StartCodeBlock(const reflection::Object *object) {
    current_obj_ = object;
    current_enum_ = nullptr;
    requires_.clear();
  }

  std::string RegisterRequires(const r::Field *field,
                               bool use_element = false) {
    std::string type_name;

    const r::BaseType type =
        use_element ? field->type()->element() : field->type()->base_type();

    if (IsStructOrTable(type)) {
      const r::Object *object = GetObjectByIndex(field->type()->index());
      if (object == current_obj_) { return namer_.Denamespace(object); }
      type_name = object->name()->str();
    } else {
      const r::Enum *enum_def = GetEnumByIndex(field->type()->index());
      if (enum_def == current_enum_) { return namer_.Denamespace(enum_def); }
      type_name = enum_def->name()->str();
    }

    // Prefix with double __ to avoid name clashing, since these are defined
    // at the top of the file and have lexical scoping. Replace '.' with '_'
    // so it can be a legal identifier.
    std::string name = "__" + type_name;
    std::replace(name.begin(), name.end(), '.', '_');

    return RegisterRequires(name, type_name);
  }

  std::string RegisterRequires(const std::string &local_name,
                               const std::string &requires_name) {
    requires_[local_name] = requires_name;
    return local_name;
  }

  void EmitCodeBlock(const std::string &code_block, const std::string &name,
                     const std::string &ns,
                     const std::string &declaring_file) const {
    const std::string root_type = schema_->root_table()->name()->str();
    const std::string root_file =
        schema_->root_table()->declaration_file()->str();
    const std::string full_qualified_name = ns.empty() ? name : ns + "." + name;

    std::string code = "--[[ " + full_qualified_name + "\n\n";
    code +=
        "  Automatically generated by the FlatBuffers compiler, do not "
        "modify.\n";
    code += "  Or modify. I'm a message, not a cop.\n";
    code += "\n";
    code += "  flatc version: " + flatc_version_ + "\n";
    code += "\n";
    code += "  Declared by  : " + declaring_file + "\n";
    code += "  Rooting type : " + root_type + " (" + root_file + ")\n";
    code += "\n--]]\n\n";

    if (!requires_.empty()) {
      for (auto it = requires_.cbegin(); it != requires_.cend(); ++it) {
        code += "local " + it->first + " = require('" + it->second + "')\n";
      }
      code += "\n";
    }

    code += code_block;
    code += "return " + name;

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
  std::map<std::string, std::string> requires_;
  CodeGenOptions options_;

  const r::Object *current_obj_;
  const r::Enum *current_enum_;
  const std::string flatc_version_;
  const BfbsNamer namer_;
};
}  // namespace

std::unique_ptr<CodeGenerator> NewLuaBfbsGenerator(
    const std::string &flatc_version) {
  return std::unique_ptr<LuaBfbsGenerator>(new LuaBfbsGenerator(flatc_version));
}

}  // namespace flatbuffers
