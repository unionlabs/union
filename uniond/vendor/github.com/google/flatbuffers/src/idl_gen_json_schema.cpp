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

#include "idl_gen_json_schema.h"

#include <algorithm>
#include <iostream>
#include <limits>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"

namespace flatbuffers {

namespace jsons {

namespace {

template<class T> static std::string GenFullName(const T *enum_def) {
  std::string full_name;
  const auto &name_spaces = enum_def->defined_namespace->components;
  for (auto ns = name_spaces.cbegin(); ns != name_spaces.cend(); ++ns) {
    full_name.append(*ns + "_");
  }
  full_name.append(enum_def->name);
  return full_name;
}

template<class T> static std::string GenTypeRef(const T *enum_def) {
  return "\"$ref\" : \"#/definitions/" + GenFullName(enum_def) + "\"";
}

static std::string GenType(const std::string &name) {
  return "\"type\" : \"" + name + "\"";
}

static std::string GenType(BaseType type) {
  switch (type) {
    case BASE_TYPE_BOOL: return "\"type\" : \"boolean\"";
    case BASE_TYPE_CHAR:
      return "\"type\" : \"integer\", \"minimum\" : " +
             NumToString(std::numeric_limits<int8_t>::min()) +
             ", \"maximum\" : " +
             NumToString(std::numeric_limits<int8_t>::max());
    case BASE_TYPE_UCHAR:
      return "\"type\" : \"integer\", \"minimum\" : 0, \"maximum\" :" +
             NumToString(std::numeric_limits<uint8_t>::max());
    case BASE_TYPE_SHORT:
      return "\"type\" : \"integer\", \"minimum\" : " +
             NumToString(std::numeric_limits<int16_t>::min()) +
             ", \"maximum\" : " +
             NumToString(std::numeric_limits<int16_t>::max());
    case BASE_TYPE_USHORT:
      return "\"type\" : \"integer\", \"minimum\" : 0, \"maximum\" : " +
             NumToString(std::numeric_limits<uint16_t>::max());
    case BASE_TYPE_INT:
      return "\"type\" : \"integer\", \"minimum\" : " +
             NumToString(std::numeric_limits<int32_t>::min()) +
             ", \"maximum\" : " +
             NumToString(std::numeric_limits<int32_t>::max());
    case BASE_TYPE_UINT:
      return "\"type\" : \"integer\", \"minimum\" : 0, \"maximum\" : " +
             NumToString(std::numeric_limits<uint32_t>::max());
    case BASE_TYPE_LONG:
      return "\"type\" : \"integer\", \"minimum\" : " +
             NumToString(std::numeric_limits<int64_t>::min()) +
             ", \"maximum\" : " +
             NumToString(std::numeric_limits<int64_t>::max());
    case BASE_TYPE_ULONG:
      return "\"type\" : \"integer\", \"minimum\" : 0, \"maximum\" : " +
             NumToString(std::numeric_limits<uint64_t>::max());
    case BASE_TYPE_FLOAT:
    case BASE_TYPE_DOUBLE: return "\"type\" : \"number\"";
    case BASE_TYPE_STRING: return "\"type\" : \"string\"";
    default: return "";
  }
}

static std::string GenBaseType(const Type &type) {
  if (type.struct_def != nullptr) { return GenTypeRef(type.struct_def); }
  if (type.enum_def != nullptr) { return GenTypeRef(type.enum_def); }
  return GenType(type.base_type);
}

static std::string GenArrayType(const Type &type) {
  std::string element_type;
  if (type.struct_def != nullptr) {
    element_type = GenTypeRef(type.struct_def);
  } else if (type.enum_def != nullptr) {
    element_type = GenTypeRef(type.enum_def);
  } else {
    element_type = GenType(type.element);
  }

  return "\"type\" : \"array\", \"items\" : {" + element_type + "}";
}

static std::string GenType(const Type &type) {
  switch (type.base_type) {
    case BASE_TYPE_ARRAY: FLATBUFFERS_FALLTHROUGH();  // fall thru
    case BASE_TYPE_VECTOR: {
      return GenArrayType(type);
    }
    case BASE_TYPE_STRUCT: {
      return GenTypeRef(type.struct_def);
    }
    case BASE_TYPE_UNION: {
      std::string union_type_string("\"anyOf\": [");
      const auto &union_types = type.enum_def->Vals();
      for (auto ut = union_types.cbegin(); ut < union_types.cend(); ++ut) {
        const auto &union_type = *ut;
        if (union_type->union_type.base_type == BASE_TYPE_NONE) { continue; }
        if (union_type->union_type.base_type == BASE_TYPE_STRUCT) {
          union_type_string.append(
              "{ " + GenTypeRef(union_type->union_type.struct_def) + " }");
        }
        if (union_type != *type.enum_def->Vals().rbegin()) {
          union_type_string.append(",");
        }
      }
      union_type_string.append("]");
      return union_type_string;
    }
    case BASE_TYPE_UTYPE: return GenTypeRef(type.enum_def);
    default: {
      return GenBaseType(type);
    }
  }
}

}  // namespace

class JsonSchemaGenerator : public BaseGenerator {
 private:
  std::string code_;

 public:
  JsonSchemaGenerator(const Parser &parser, const std::string &path,
                      const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "", "", "json") {}

  explicit JsonSchemaGenerator(const BaseGenerator &base_generator)
      : BaseGenerator(base_generator) {}

  std::string GeneratedFileName(const std::string &path,
                                const std::string &file_name,
                                const IDLOptions &options /* unused */) const {
    (void)options;
    return path + file_name + ".schema.json";
  }

  // If indentation is less than 0, that indicates we don't want any newlines
  // either.
  std::string NewLine() const {
    return parser_.opts.indent_step >= 0 ? "\n" : "";
  }

  std::string Indent(int indent) const {
    const auto num_spaces = indent * std::max(parser_.opts.indent_step, 0);
    return std::string(num_spaces, ' ');
  }

  std::string PrepareDescription(
      const std::vector<std::string> &comment_lines) {
    std::string comment;
    for (auto line_iterator = comment_lines.cbegin();
         line_iterator != comment_lines.cend(); ++line_iterator) {
      const auto &comment_line = *line_iterator;

      // remove leading and trailing spaces from comment line
      const auto start = std::find_if(comment_line.begin(), comment_line.end(),
                                      [](char c) { return !isspace(c); });
      const auto end =
          std::find_if(comment_line.rbegin(), comment_line.rend(), [](char c) {
            return !isspace(c);
          }).base();
      if (start < end) {
        comment.append(start, end);
      } else {
        comment.append(comment_line);
      }

      if (line_iterator + 1 != comment_lines.cend()) comment.append("\n");
    }
    if (!comment.empty()) {
      std::string description;
      if (EscapeString(comment.c_str(), comment.length(), &description, true,
                       true)) {
        return description;
      }
      return "";
    }
    return "";
  }

  bool generate() {
    code_ = "";
    if (parser_.root_struct_def_ == nullptr) {
      std::cerr << "Error: Binary schema not generated, no root struct found\n";
      return false;
    }
    code_ += "{" + NewLine();
    code_ += Indent(1) +
             "\"$schema\": \"https://json-schema.org/draft/2019-09/schema\"," +
             NewLine();
    code_ += Indent(1) + "\"definitions\": {" + NewLine();
    for (auto e = parser_.enums_.vec.cbegin(); e != parser_.enums_.vec.cend();
         ++e) {
      code_ += Indent(2) + "\"" + GenFullName(*e) + "\" : {" + NewLine();
      code_ += Indent(3) + GenType("string") + "," + NewLine();
      auto enumdef(Indent(3) + "\"enum\": [");
      for (auto enum_value = (*e)->Vals().begin();
           enum_value != (*e)->Vals().end(); ++enum_value) {
        enumdef.append("\"" + (*enum_value)->name + "\"");
        if (*enum_value != (*e)->Vals().back()) { enumdef.append(", "); }
      }
      enumdef.append("]");
      code_ += enumdef + NewLine();
      code_ += Indent(2) + "}," + NewLine();  // close type
    }
    for (auto s = parser_.structs_.vec.cbegin();
         s != parser_.structs_.vec.cend(); ++s) {
      const auto &structure = *s;
      code_ += Indent(2) + "\"" + GenFullName(structure) + "\" : {" + NewLine();
      code_ += Indent(3) + GenType("object") + "," + NewLine();
      const auto &comment_lines = structure->doc_comment;
      auto comment = PrepareDescription(comment_lines);
      if (comment != "") {
        code_ += Indent(3) + "\"description\" : " + comment + "," + NewLine();
      }

      code_ += Indent(3) + "\"properties\" : {" + NewLine();

      const auto &properties = structure->fields.vec;
      for (auto prop = properties.cbegin(); prop != properties.cend(); ++prop) {
        const auto &property = *prop;
        std::string arrayInfo = "";
        if (IsArray(property->value.type)) {
          arrayInfo = "," + NewLine() + Indent(8) + "\"minItems\": " +
                      NumToString(property->value.type.fixed_length) + "," +
                      NewLine() + Indent(8) + "\"maxItems\": " +
                      NumToString(property->value.type.fixed_length);
        }
        std::string deprecated_info = "";
        if (property->deprecated) {
          deprecated_info =
              "," + NewLine() + Indent(8) + "\"deprecated\" : true";
        }
        std::string typeLine = Indent(4) + "\"" + property->name + "\"";
        typeLine += " : {" + NewLine() + Indent(8);
        typeLine += GenType(property->value.type);
        typeLine += arrayInfo;
        typeLine += deprecated_info;
        auto description = PrepareDescription(property->doc_comment);
        if (description != "") {
          typeLine +=
              "," + NewLine() + Indent(8) + "\"description\" : " + description;
        }

        typeLine += NewLine() + Indent(7) + "}";
        if (property != properties.back()) { typeLine.append(","); }
        code_ += typeLine + NewLine();
      }
      code_ += Indent(3) + "}," + NewLine();  // close properties

      std::vector<FieldDef *> requiredProperties;
      std::copy_if(properties.begin(), properties.end(),
                   back_inserter(requiredProperties),
                   [](FieldDef const *prop) { return prop->IsRequired(); });
      if (!requiredProperties.empty()) {
        auto required_string(Indent(3) + "\"required\" : [");
        for (auto req_prop = requiredProperties.cbegin();
             req_prop != requiredProperties.cend(); ++req_prop) {
          required_string.append("\"" + (*req_prop)->name + "\"");
          if (*req_prop != requiredProperties.back()) {
            required_string.append(", ");
          }
        }
        required_string.append("],");
        code_ += required_string + NewLine();
      }
      code_ += Indent(3) + "\"additionalProperties\" : false" + NewLine();
      auto closeType(Indent(2) + "}");
      if (*s != parser_.structs_.vec.back()) { closeType.append(","); }
      code_ += closeType + NewLine();  // close type
    }
    code_ += Indent(1) + "}," + NewLine();  // close definitions

    // mark root type
    code_ += Indent(1) + "\"$ref\" : \"#/definitions/" +
             GenFullName(parser_.root_struct_def_) + "\"" + NewLine();

    code_ += "}" + NewLine();  // close schema root
    return true;
  }

  bool save() const {
    const auto file_path = GeneratedFileName(path_, file_name_, parser_.opts);
    return SaveFile(file_path.c_str(), code_, false);
  }

  const std::string getJson() { return code_; }
};
}  // namespace jsons

static bool GenerateJsonSchema(const Parser &parser, const std::string &path,
                               const std::string &file_name) {
  jsons::JsonSchemaGenerator generator(parser, path, file_name);
  if (!generator.generate()) { return false; }
  return generator.save();
}

namespace {

class JsonSchemaCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateJsonSchema(parser, path, filename)) { return Status::ERROR; }
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
    return IDLOptions::kJsonSchema;
  }

  std::string LanguageName() const override { return "JsonSchema"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewJsonSchemaCodeGenerator() {
  return std::unique_ptr<JsonSchemaCodeGenerator>(
      new JsonSchemaCodeGenerator());
}
}  // namespace flatbuffers
