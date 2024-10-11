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

#include "idl_gen_go.h"

#include <algorithm>
#include <cmath>
#include <sstream>
#include <string>

#include "flatbuffers/base.h"
#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

#ifdef _WIN32
#  include <direct.h>
#  define PATH_SEPARATOR "\\"
#  define mkdir(n, m) _mkdir(n)
#else
#  include <sys/stat.h>
#  define PATH_SEPARATOR "/"
#endif

namespace flatbuffers {

namespace go {

namespace {

// see https://golang.org/ref/spec#Keywords
static std::set<std::string> GoKeywords() {
  return {
    "break",    "default",     "func",   "interface", "select",
    "case",     "defer",       "go",     "map",       "struct",
    "chan",     "else",        "goto",   "package",   "switch",
    "const",    "fallthrough", "if",     "range",     "type",
    "continue", "for",         "import", "return",    "var",
  };
}

static Namer::Config GoDefaultConfig() {
  // Note that the functions with user defined types in the name use
  // upper camel case for all but the user defined type itself, which is keep
  // cased. Despite being a function, we interpret it as a Type.
  return { /*types=*/Case::kKeep,
           /*constants=*/Case::kUnknown,
           /*methods=*/Case::kUpperCamel,
           /*functions=*/Case::kUpperCamel,
           /*fields=*/Case::kUpperCamel,
           /*variables=*/Case::kLowerCamel,
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/"",  // I.e. Concatenate.
           /*escape_keywords=*/Namer::Config::Escape::AfterConvertingCase,
           /*namespaces=*/Case::kKeep,
           /*namespace_seperator=*/"__",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kKeep,
           /*directories=*/Case::kKeep,
           /*output_path=*/"",
           /*filename_suffix=*/"",
           /*filename_extension=*/".go" };
}

}  // namespace

class GoGenerator : public BaseGenerator {
 public:
  GoGenerator(const Parser &parser, const std::string &path,
              const std::string &file_name, const std::string &go_namespace)
      : BaseGenerator(parser, path, file_name, "" /* not used*/,
                      "" /* not used */, "go"),
        cur_name_space_(nullptr),
        namer_(WithFlagOptions(GoDefaultConfig(), parser.opts, path),
               GoKeywords()) {
    std::istringstream iss(go_namespace);
    std::string component;
    while (std::getline(iss, component, '.')) {
      go_namespace_.components.push_back(component);
    }
  }

  bool generate() {
    std::string one_file_code;

    if (!generateEnums(&one_file_code)) return false;
    if (!generateStructs(&one_file_code)) return false;

    if (parser_.opts.one_file) {
      std::string code = "";
      const bool is_enum = !parser_.enums_.vec.empty();
      BeginFile(LastNamespacePart(go_namespace_), true, is_enum, &code);
      code += one_file_code;
      const std::string filename =
          GeneratedFileName(path_, file_name_, parser_.opts);
      return SaveFile(filename.c_str(), code, false);
    }

    return true;
  }

 private:
  bool generateEnums(std::string *one_file_code) {
    bool needs_imports = false;
    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      if (!parser_.opts.one_file) {
        needs_imports = false;
        ResetImports();
      }
      auto &enum_def = **it;
      std::string enumcode;
      GenEnum(enum_def, &enumcode);
      if (enum_def.is_union && parser_.opts.generate_object_based_api) {
        GenNativeUnionCreator(enum_def, &enumcode);
        needs_imports = true;
      }
      if (parser_.opts.one_file) {
        *one_file_code += enumcode;
      } else {
        if (!SaveType(enum_def, enumcode, needs_imports, true)) return false;
      }
    }
    return true;
  }

  void GenNativeUnionCreator(const EnumDef &enum_def, std::string *code_ptr) {
    if (enum_def.generated) return;

    GenNativeUnion(enum_def, code_ptr);
    GenNativeUnionPack(enum_def, code_ptr);
    GenNativeUnionUnPack(enum_def, code_ptr);
  }

  bool generateStructs(std::string *one_file_code) {
    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      if (!parser_.opts.one_file) { ResetImports(); }
      std::string declcode;
      auto &struct_def = **it;
      GenStruct(struct_def, &declcode);
      if (parser_.opts.one_file) {
        *one_file_code += declcode;
      } else {
        if (!SaveType(struct_def, declcode, true, false)) return false;
      }
    }
    return true;
  }

  Namespace go_namespace_;
  Namespace *cur_name_space_;
  const IdlNamer namer_;

  struct NamespacePtrLess {
    bool operator()(const Definition *a, const Definition *b) const {
      return *a->defined_namespace < *b->defined_namespace;
    }
  };
  std::set<const Definition *, NamespacePtrLess> tracked_imported_namespaces_;
  bool needs_math_import_ = false;
  bool needs_bytes_import_ = false;

  // Most field accessors need to retrieve and test the field offset first,
  // this is the prefix code for that.
  std::string OffsetPrefix(const FieldDef &field) {
    return "{\n\to := flatbuffers.UOffsetT(rcv._tab.Offset(" +
           NumToString(field.value.offset) + "))\n\tif o != 0 {\n";
  }

  // Begin a class declaration.
  void BeginClass(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    code += "type " + namer_.Type(struct_def) + " struct {\n\t";

    // _ is reserved in flatbuffers field names, so no chance of name
    // conflict:
    code += "_tab ";
    code += struct_def.fixed ? "flatbuffers.Struct" : "flatbuffers.Table";
    code += "\n}\n\n";
  }

  // Construct the name of the type for this enum.
  std::string GetEnumTypeName(const EnumDef &enum_def) {
    return WrapInNameSpaceAndTrack(&enum_def, namer_.Type(enum_def));
  }

  // Create a type for the enum values.
  void GenEnumType(const EnumDef &enum_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "type " + GetEnumTypeName(enum_def) + " ";
    code += GenTypeBasic(enum_def.underlying_type) + "\n\n";
  }

  // Begin enum code with a class declaration.
  void BeginEnum(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "const (\n";
  }

  // A single enum member.
  void EnumMember(const EnumDef &enum_def, const EnumVal &ev,
                  size_t max_name_length, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\t";
    code += namer_.EnumVariant(enum_def, ev);
    code += " ";
    code += std::string(max_name_length - ev.name.length(), ' ');
    code += GetEnumTypeName(enum_def);
    code += " = ";
    code += enum_def.ToString(ev) + "\n";
  }

  // End enum code.
  void EndEnum(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += ")\n\n";
  }

  // Begin enum name map.
  void BeginEnumNames(const EnumDef &enum_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "var EnumNames";
    code += enum_def.name;
    code += " = map[" + GetEnumTypeName(enum_def) + "]string{\n";
  }

  // A single enum name member.
  void EnumNameMember(const EnumDef &enum_def, const EnumVal &ev,
                      size_t max_name_length, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\t";
    code += namer_.EnumVariant(enum_def, ev);
    code += ": ";
    code += std::string(max_name_length - ev.name.length(), ' ');
    code += "\"";
    code += ev.name;
    code += "\",\n";
  }

  // End enum name map.
  void EndEnumNames(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "}\n\n";
  }

  // Generate String() method on enum type.
  void EnumStringer(const EnumDef &enum_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    const std::string enum_type = namer_.Type(enum_def);
    code += "func (v " + enum_type + ") String() string {\n";
    code += "\tif s, ok := EnumNames" + enum_type + "[v]; ok {\n";
    code += "\t\treturn s\n";
    code += "\t}\n";
    code += "\treturn \"" + enum_def.name;
    code += "(\" + strconv.FormatInt(int64(v), 10) + \")\"\n";
    code += "}\n\n";
  }

  // Begin enum value map.
  void BeginEnumValues(const EnumDef &enum_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "var EnumValues";
    code += namer_.Type(enum_def);
    code += " = map[string]" + GetEnumTypeName(enum_def) + "{\n";
  }

  // A single enum value member.
  void EnumValueMember(const EnumDef &enum_def, const EnumVal &ev,
                       size_t max_name_length, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\t\"";
    code += ev.name;
    code += "\": ";
    code += std::string(max_name_length - ev.name.length(), ' ');
    code += namer_.EnumVariant(enum_def, ev);
    code += ",\n";
  }

  // End enum value map.
  void EndEnumValues(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "}\n\n";
  }

  // Initialize a new struct or table from existing data.
  void NewRootTypeFromBuffer(const StructDef &struct_def,
                             std::string *code_ptr) {
    std::string &code = *code_ptr;
    const std::string size_prefix[] = { "", "SizePrefixed" };
    const std::string struct_type = namer_.Type(struct_def);

    bool has_file_identifier = (parser_.root_struct_def_ == &struct_def) &&
                               parser_.file_identifier_.length();

    if (has_file_identifier) {
      code += "const " + struct_type + "Identifier = \"" +
              parser_.file_identifier_ + "\"\n\n";
    }

    for (int i = 0; i < 2; i++) {
      code += "func Get" + size_prefix[i] + "RootAs" + struct_type;
      code += "(buf []byte, offset flatbuffers.UOffsetT) ";
      code += "*" + struct_type + "";
      code += " {\n";
      if (i == 0) {
        code += "\tn := flatbuffers.GetUOffsetT(buf[offset:])\n";
      } else {
        code +=
            "\tn := "
            "flatbuffers.GetUOffsetT(buf[offset+flatbuffers.SizeUint32:])\n";
      }
      code += "\tx := &" + struct_type + "{}\n";
      if (i == 0) {
        code += "\tx.Init(buf, n+offset)\n";
      } else {
        code += "\tx.Init(buf, n+offset+flatbuffers.SizeUint32)\n";
      }
      code += "\treturn x\n";
      code += "}\n\n";

      code += "func Finish" + size_prefix[i] + struct_type +
              "Buffer(builder *flatbuffers.Builder, offset "
              "flatbuffers.UOffsetT) {\n";
      if (has_file_identifier) {
        code += "\tidentifierBytes := []byte(" + struct_type + "Identifier)\n";
        code += "\tbuilder.Finish" + size_prefix[i] +
                "WithFileIdentifier(offset, identifierBytes)\n";
      } else {
        code += "\tbuilder.Finish" + size_prefix[i] + "(offset)\n";
      }
      code += "}\n\n";

      if (has_file_identifier) {
        code += "func " + size_prefix[i] + struct_type +
                "BufferHasIdentifier(buf []byte) bool {\n";
        code += "\treturn flatbuffers." + size_prefix[i] +
                "BufferHasIdentifier(buf, " + struct_type + "Identifier)\n";
        code += "}\n\n";
      }
    }
  }

  // Initialize an existing object with other data, to avoid an allocation.
  void InitializeExisting(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += " Init(buf []byte, i flatbuffers.UOffsetT) ";
    code += "{\n";
    code += "\trcv._tab.Bytes = buf\n";
    code += "\trcv._tab.Pos = i\n";
    code += "}\n\n";
  }

  // Implement the table accessor
  void GenTableAccessor(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += " Table() flatbuffers.Table ";
    code += "{\n";

    if (struct_def.fixed) {
      code += "\treturn rcv._tab.Table\n";
    } else {
      code += "\treturn rcv._tab\n";
    }
    code += "}\n\n";
  }

  // Get the length of a vector.
  void GetVectorLen(const StructDef &struct_def, const FieldDef &field,
                    std::string *code_ptr) {
    std::string &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field) + "Length(";
    code += ") int " + OffsetPrefix(field);
    code += "\t\treturn rcv._tab.VectorLen(o)\n\t}\n";
    code += "\treturn 0\n}\n\n";
  }

  // Get a [ubyte] vector as a byte slice.
  void GetUByteSlice(const StructDef &struct_def, const FieldDef &field,
                     std::string *code_ptr) {
    std::string &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field) + "Bytes(";
    code += ") []byte " + OffsetPrefix(field);
    code += "\t\treturn rcv._tab.ByteVector(o + rcv._tab.Pos)\n\t}\n";
    code += "\treturn nil\n}\n\n";
  }

  // Get the value of a struct's scalar.
  void GetScalarFieldOfStruct(const StructDef &struct_def,
                              const FieldDef &field, std::string *code_ptr) {
    std::string &code = *code_ptr;
    std::string getter = GenGetter(field.value.type);
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "() " + TypeName(field) + " {\n";
    code += "\treturn " +
            CastToEnum(field.value.type,
                       getter + "(rcv._tab.Pos + flatbuffers.UOffsetT(" +
                           NumToString(field.value.offset) + "))");
    code += "\n}\n";
  }

  // Get the value of a table's scalar.
  void GetScalarFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                             std::string *code_ptr) {
    std::string &code = *code_ptr;
    std::string getter = GenGetter(field.value.type);
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "() " + TypeName(field) + " ";
    code += OffsetPrefix(field);
    if (field.IsScalarOptional()) {
      code += "\t\tv := ";
    } else {
      code += "\t\treturn ";
    }
    code += CastToEnum(field.value.type, getter + "(o + rcv._tab.Pos)");
    if (field.IsScalarOptional()) { code += "\n\t\treturn &v"; }
    code += "\n\t}\n";
    code += "\treturn " + GenConstant(field) + "\n";
    code += "}\n\n";
  }

  // Get a struct by initializing an existing struct.
  // Specific to Struct.
  void GetStructFieldOfStruct(const StructDef &struct_def,
                              const FieldDef &field, std::string *code_ptr) {
    std::string &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "(obj *" + TypeName(field);
    code += ") *" + TypeName(field);
    code += " {\n";
    code += "\tif obj == nil {\n";
    code += "\t\tobj = new(" + TypeName(field) + ")\n";
    code += "\t}\n";
    code += "\tobj.Init(rcv._tab.Bytes, rcv._tab.Pos+";
    code += NumToString(field.value.offset) + ")";
    code += "\n\treturn obj\n";
    code += "}\n";
  }

  // Get a struct by initializing an existing struct.
  // Specific to Table.
  void GetStructFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                             std::string *code_ptr) {
    std::string &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "(obj *";
    code += TypeName(field);
    code += ") *" + TypeName(field) + " " + OffsetPrefix(field);
    if (field.value.type.struct_def->fixed) {
      code += "\t\tx := o + rcv._tab.Pos\n";
    } else {
      code += "\t\tx := rcv._tab.Indirect(o + rcv._tab.Pos)\n";
    }
    code += "\t\tif obj == nil {\n";
    code += "\t\t\tobj = new(" + TypeName(field) + ")\n";
    code += "\t\t}\n";
    code += "\t\tobj.Init(rcv._tab.Bytes, x)\n";
    code += "\t\treturn obj\n\t}\n\treturn nil\n";
    code += "}\n\n";
  }

  // Get the value of a string.
  void GetStringField(const StructDef &struct_def, const FieldDef &field,
                      std::string *code_ptr) {
    std::string &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "() " + TypeName(field) + " ";
    code += OffsetPrefix(field) + "\t\treturn " + GenGetter(field.value.type);
    code += "(o + rcv._tab.Pos)\n\t}\n\treturn nil\n";
    code += "}\n\n";
  }

  // Get the value of a union from an object.
  void GetUnionField(const StructDef &struct_def, const FieldDef &field,
                     std::string *code_ptr) {
    std::string &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field) + "(";
    code += "obj " + GenTypePointer(field.value.type) + ") bool ";
    code += OffsetPrefix(field);
    code += "\t\t" + GenGetter(field.value.type);
    code += "(obj, o)\n\t\treturn true\n\t}\n";
    code += "\treturn false\n";
    code += "}\n\n";
  }

  // Get the value of a vector's struct member.
  void GetMemberOfVectorOfStruct(const StructDef &struct_def,
                                 const FieldDef &field, std::string *code_ptr) {
    std::string &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();

    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "(obj *" + TypeName(field);
    code += ", j int) bool " + OffsetPrefix(field);
    code += "\t\tx := rcv._tab.Vector(o)\n";
    code += "\t\tx += flatbuffers.UOffsetT(j) * ";
    code += NumToString(InlineSize(vectortype)) + "\n";
    if (!(vectortype.struct_def->fixed)) {
      code += "\t\tx = rcv._tab.Indirect(x)\n";
    }
    code += "\t\tobj.Init(rcv._tab.Bytes, x)\n";
    code += "\t\treturn true\n\t}\n";
    code += "\treturn false\n";
    code += "}\n\n";
  }

  void GetMemberOfVectorOfStructByKey(const StructDef &struct_def,
                                      const FieldDef &field,
                                      std::string *code_ptr) {
    std::string &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();
    FLATBUFFERS_ASSERT(vectortype.struct_def->has_key);

    auto &vector_struct_fields = vectortype.struct_def->fields.vec;
    auto kit =
        std::find_if(vector_struct_fields.begin(), vector_struct_fields.end(),
                     [&](FieldDef *vector_struct_field) {
                       return vector_struct_field->key;
                     });

    auto &key_field = **kit;
    FLATBUFFERS_ASSERT(key_field.key);

    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Field(field) + "ByKey";
    code += "(obj *" + TypeName(field);
    code += ", key " + NativeType(key_field.value.type) + ") bool " +
            OffsetPrefix(field);
    code += "\t\tx := rcv._tab.Vector(o)\n";
    code += "\t\treturn ";
    code += "obj.LookupByKey(key, x, rcv._tab.Bytes)\n";
    code += "\t}\n";
    code += "\treturn false\n";
    code += "}\n\n";
  }

  // Get the value of a vector's non-struct member.
  void GetMemberOfVectorOfNonStruct(const StructDef &struct_def,
                                    const FieldDef &field,
                                    std::string *code_ptr) {
    std::string &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();

    GenReceiver(struct_def, code_ptr);
    code += " " + namer_.Function(field);
    code += "(j int) " + TypeName(field) + " ";
    code += OffsetPrefix(field);
    code += "\t\ta := rcv._tab.Vector(o)\n";
    code += "\t\treturn " +
            CastToEnum(field.value.type,
                       GenGetter(field.value.type) +
                           "(a + flatbuffers.UOffsetT(j*" +
                           NumToString(InlineSize(vectortype)) + "))");
    code += "\n\t}\n";
    if (IsString(vectortype)) {
      code += "\treturn nil\n";
    } else if (vectortype.base_type == BASE_TYPE_BOOL) {
      code += "\treturn false\n";
    } else {
      code += "\treturn 0\n";
    }
    code += "}\n\n";
  }

  // Begin the creator function signature.
  void BeginBuilderArgs(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    if (code.substr(code.length() - 2) != "\n\n") {
      // a previous mutate has not put an extra new line
      code += "\n";
    }
    code += "func Create" + struct_def.name;
    code += "(builder *flatbuffers.Builder";
  }

  // Recursively generate arguments for a constructor, to deal with nested
  // structs.
  void StructBuilderArgs(const StructDef &struct_def, const char *nameprefix,
                         std::string *code_ptr) {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (IsStruct(field.value.type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        StructBuilderArgs(*field.value.type.struct_def,
                          (nameprefix + (field.name + "_")).c_str(), code_ptr);
      } else {
        std::string &code = *code_ptr;
        code += std::string(", ") + nameprefix;
        code += namer_.Variable(field);
        code += " " + TypeName(field);
      }
    }
  }

  // End the creator function signature.
  void EndBuilderArgs(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += ") flatbuffers.UOffsetT {\n";
  }

  // Recursively generate struct construction statements and instert manual
  // padding.
  void StructBuilderBody(const StructDef &struct_def, const char *nameprefix,
                         std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\tbuilder.Prep(" + NumToString(struct_def.minalign) + ", ";
    code += NumToString(struct_def.bytesize) + ")\n";
    for (auto it = struct_def.fields.vec.rbegin();
         it != struct_def.fields.vec.rend(); ++it) {
      auto &field = **it;
      if (field.padding)
        code += "\tbuilder.Pad(" + NumToString(field.padding) + ")\n";
      if (IsStruct(field.value.type)) {
        StructBuilderBody(*field.value.type.struct_def,
                          (nameprefix + (field.name + "_")).c_str(), code_ptr);
      } else {
        code += "\tbuilder.Prepend" + GenMethod(field) + "(";
        code += CastToBaseType(field.value.type,
                               nameprefix + namer_.Variable(field)) +
                ")\n";
      }
    }
  }

  void EndBuilderBody(std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "\treturn builder.Offset()\n";
    code += "}\n";
  }

  // Get the value of a table's starting offset.
  void GetStartOfTable(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "func " + namer_.Type(struct_def) + "Start";
    code += "(builder *flatbuffers.Builder) {\n";
    code += "\tbuilder.StartObject(";
    code += NumToString(struct_def.fields.vec.size());
    code += ")\n}\n";
  }

  // Set the value of a table's field.
  void BuildFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                         const size_t offset, std::string *code_ptr) {
    std::string &code = *code_ptr;
    const std::string field_var = namer_.Variable(field);
    code += "func " + namer_.Type(struct_def) + "Add" + namer_.Function(field);
    code += "(builder *flatbuffers.Builder, ";
    code += field_var + " ";
    if (!IsScalar(field.value.type.base_type) && (!struct_def.fixed)) {
      code += "flatbuffers.UOffsetT";
    } else {
      code += GenTypeGet(field.value.type);
    }
    code += ") {\n\t";
    code += "builder.Prepend";
    code += GenMethod(field);
    if (field.IsScalarOptional()) {
      code += "(";
    } else {
      code += "Slot(" + NumToString(offset) + ", ";
    }
    if (!IsScalar(field.value.type.base_type) && (!struct_def.fixed)) {
      code += "flatbuffers.UOffsetT";
      code += "(" + field_var + ")";
    } else {
      code += CastToBaseType(field.value.type, field_var);
    }
    if (field.IsScalarOptional()) {
      code += ")\n";
      code += "\tbuilder.Slot(" + NumToString(offset);
    } else {
      code += ", " + GenConstant(field);
    }
    code += ")\n";
    code += "}\n";
  }

  // Set the value of one of the members of a table's vector.
  void BuildVectorOfTable(const StructDef &struct_def, const FieldDef &field,
                          std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "func " + namer_.Type(struct_def) + "Start";
    code += namer_.Function(field);
    code += "Vector(builder *flatbuffers.Builder, numElems int) ";
    code += "flatbuffers.UOffsetT {\n\treturn builder.StartVector(";
    auto vector_type = field.value.type.VectorType();
    auto alignment = InlineAlignment(vector_type);
    auto elem_size = InlineSize(vector_type);
    code += NumToString(elem_size);
    code += ", numElems, " + NumToString(alignment);
    code += ")\n}\n";
  }

  // Get the offset of the end of a table.
  void GetEndOffsetOnTable(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "func " + namer_.Type(struct_def) + "End";
    code += "(builder *flatbuffers.Builder) flatbuffers.UOffsetT ";
    code += "{\n\treturn builder.EndObject()\n}\n";
  }

  // Generate the receiver for function signatures.
  void GenReceiver(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code += "func (rcv *" + namer_.Type(struct_def) + ")";
  }

  // Generate a struct field getter, conditioned on its child type(s).
  void GenStructAccessor(const StructDef &struct_def, const FieldDef &field,
                         std::string *code_ptr) {
    GenComment(field.doc_comment, code_ptr, nullptr, "");
    if (IsScalar(field.value.type.base_type)) {
      if (struct_def.fixed) {
        GetScalarFieldOfStruct(struct_def, field, code_ptr);
      } else {
        GetScalarFieldOfTable(struct_def, field, code_ptr);
      }
    } else {
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT:
          if (struct_def.fixed) {
            GetStructFieldOfStruct(struct_def, field, code_ptr);
          } else {
            GetStructFieldOfTable(struct_def, field, code_ptr);
          }
          break;
        case BASE_TYPE_STRING:
          GetStringField(struct_def, field, code_ptr);
          break;
        case BASE_TYPE_VECTOR: {
          auto vectortype = field.value.type.VectorType();
          if (vectortype.base_type == BASE_TYPE_STRUCT) {
            GetMemberOfVectorOfStruct(struct_def, field, code_ptr);
            // TODO(michaeltle): Support querying fixed struct by key.
            // Currently, we only support keyed tables.
            if (!vectortype.struct_def->fixed &&
                vectortype.struct_def->has_key) {
              GetMemberOfVectorOfStructByKey(struct_def, field, code_ptr);
            }
          } else {
            GetMemberOfVectorOfNonStruct(struct_def, field, code_ptr);
          }
          break;
        }
        case BASE_TYPE_UNION: GetUnionField(struct_def, field, code_ptr); break;
        default: FLATBUFFERS_ASSERT(0);
      }
    }
    if (IsVector(field.value.type)) {
      GetVectorLen(struct_def, field, code_ptr);
      if (field.value.type.element == BASE_TYPE_UCHAR) {
        GetUByteSlice(struct_def, field, code_ptr);
      }
    }
  }

  // Mutate the value of a struct's scalar.
  void MutateScalarFieldOfStruct(const StructDef &struct_def,
                                 const FieldDef &field, std::string *code_ptr) {
    std::string &code = *code_ptr;
    std::string setter =
        "rcv._tab.Mutate" + namer_.Method(GenTypeBasic(field.value.type));
    GenReceiver(struct_def, code_ptr);
    code += " Mutate" + namer_.Function(field);
    code +=
        "(n " + GenTypeGet(field.value.type) + ") bool {\n\treturn " + setter;
    code += "(rcv._tab.Pos+flatbuffers.UOffsetT(";
    code += NumToString(field.value.offset) + "), ";
    code += CastToBaseType(field.value.type, "n") + ")\n}\n\n";
  }

  // Mutate the value of a table's scalar.
  void MutateScalarFieldOfTable(const StructDef &struct_def,
                                const FieldDef &field, std::string *code_ptr) {
    std::string &code = *code_ptr;
    std::string setter = "rcv._tab.Mutate" +
                         namer_.Method(GenTypeBasic(field.value.type)) + "Slot";
    GenReceiver(struct_def, code_ptr);
    code += " Mutate" + namer_.Function(field);
    code += "(n " + GenTypeGet(field.value.type) + ") bool {\n\treturn ";
    code += setter + "(" + NumToString(field.value.offset) + ", ";
    code += CastToBaseType(field.value.type, "n") + ")\n";
    code += "}\n\n";
  }

  // Mutate an element of a vector of scalars.
  void MutateElementOfVectorOfNonStruct(const StructDef &struct_def,
                                        const FieldDef &field,
                                        std::string *code_ptr) {
    std::string &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();
    std::string setter =
        "rcv._tab.Mutate" + namer_.Method(GenTypeBasic(vectortype));
    GenReceiver(struct_def, code_ptr);
    code += " Mutate" + namer_.Function(field);
    code += "(j int, n " + TypeName(field) + ") bool ";
    code += OffsetPrefix(field);
    code += "\t\ta := rcv._tab.Vector(o)\n";
    code += "\t\treturn " + setter + "(";
    code += "a+flatbuffers.UOffsetT(j*";
    code += NumToString(InlineSize(vectortype)) + "), ";
    code += CastToBaseType(vectortype, "n") + ")\n";
    code += "\t}\n";
    code += "\treturn false\n";
    code += "}\n\n";
  }

  // Generate a struct field setter, conditioned on its child type(s).
  void GenStructMutator(const StructDef &struct_def, const FieldDef &field,
                        std::string *code_ptr) {
    GenComment(field.doc_comment, code_ptr, nullptr, "");
    if (IsScalar(field.value.type.base_type)) {
      if (struct_def.fixed) {
        MutateScalarFieldOfStruct(struct_def, field, code_ptr);
      } else {
        MutateScalarFieldOfTable(struct_def, field, code_ptr);
      }
    } else if (IsVector(field.value.type)) {
      if (IsScalar(field.value.type.element)) {
        MutateElementOfVectorOfNonStruct(struct_def, field, code_ptr);
      }
    }
  }

  // Generate table constructors, conditioned on its members' types.
  void GenTableBuilders(const StructDef &struct_def, std::string *code_ptr) {
    GetStartOfTable(struct_def, code_ptr);

    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      auto offset = it - struct_def.fields.vec.begin();
      BuildFieldOfTable(struct_def, field, offset, code_ptr);
      if (IsVector(field.value.type)) {
        BuildVectorOfTable(struct_def, field, code_ptr);
      }
    }

    GetEndOffsetOnTable(struct_def, code_ptr);
  }

  // Generate struct or table methods.
  void GenStruct(const StructDef &struct_def, std::string *code_ptr) {
    if (struct_def.generated) return;

    cur_name_space_ = struct_def.defined_namespace;

    GenComment(struct_def.doc_comment, code_ptr, nullptr);
    if (parser_.opts.generate_object_based_api) {
      GenNativeStruct(struct_def, code_ptr);
    }
    BeginClass(struct_def, code_ptr);
    if (!struct_def.fixed) {
      // Generate a special accessor for the table that has been declared as
      // the root type.
      NewRootTypeFromBuffer(struct_def, code_ptr);
    }
    // Generate the Init method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    InitializeExisting(struct_def, code_ptr);
    // Generate _tab accessor
    GenTableAccessor(struct_def, code_ptr);

    // Generate struct fields accessors
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      GenStructAccessor(struct_def, field, code_ptr);
      GenStructMutator(struct_def, field, code_ptr);
      // TODO(michaeltle): Support querying fixed struct by key. Currently,
      // we only support keyed tables.
      if (!struct_def.fixed && field.key) {
        GenKeyCompare(struct_def, field, code_ptr);
        GenLookupByKey(struct_def, field, code_ptr);
      }
    }

    // Generate builders
    if (struct_def.fixed) {
      // create a struct constructor function
      GenStructBuilder(struct_def, code_ptr);
    } else {
      // Create a set of functions that allow table construction.
      GenTableBuilders(struct_def, code_ptr);
    }
  }

  void GenKeyCompare(const StructDef &struct_def, const FieldDef &field,
                     std::string *code_ptr) {
    FLATBUFFERS_ASSERT(struct_def.has_key);
    FLATBUFFERS_ASSERT(field.key);
    std::string &code = *code_ptr;

    code += "func " + namer_.Type(struct_def) + "KeyCompare(";
    code += "o1, o2 flatbuffers.UOffsetT, buf []byte) bool {\n";
    code += "\tobj1 := &" + namer_.Type(struct_def) + "{}\n";
    code += "\tobj2 := &" + namer_.Type(struct_def) + "{}\n";
    code += "\tobj1.Init(buf, flatbuffers.UOffsetT(len(buf))-o1)\n";
    code += "\tobj2.Init(buf, flatbuffers.UOffsetT(len(buf))-o2)\n";
    if (IsString(field.value.type)) {
      code += "\treturn string(obj1." + namer_.Function(field.name) + "()) < ";
      code += "string(obj2." + namer_.Function(field.name) + "())\n";
    } else {
      code += "\treturn obj1." + namer_.Function(field.name) + "() < ";
      code += "obj2." + namer_.Function(field.name) + "()\n";
    }
    code += "}\n\n";
  }

  void GenLookupByKey(const StructDef &struct_def, const FieldDef &field,
                      std::string *code_ptr) {
    FLATBUFFERS_ASSERT(struct_def.has_key);
    FLATBUFFERS_ASSERT(field.key);
    std::string &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += " LookupByKey(";
    code += "key " + NativeType(field.value.type) + ", ";
    code += "vectorLocation flatbuffers.UOffsetT, ";
    code += "buf []byte) bool {\n";
    code += "\tspan := flatbuffers.GetUOffsetT(buf[vectorLocation-4:])\n";
    code += "\tstart := flatbuffers.UOffsetT(0)\n";
    if (IsString(field.value.type)) { code += "\tbKey := []byte(key)\n"; }
    code += "\tfor span != 0 {\n";
    code += "\t\tmiddle := span / 2\n";
    code += "\t\ttableOffset := flatbuffers.GetIndirectOffset(buf, ";
    code += "vectorLocation+4*(start+middle))\n";

    code += "\t\tobj := &" + namer_.Type(struct_def) + "{}\n";
    code += "\t\tobj.Init(buf, tableOffset)\n";

    if (IsString(field.value.type)) {
      needs_bytes_import_ = true;
      code +=
          "\t\tcomp := bytes.Compare(obj." + namer_.Function(field.name) + "()";
      code += ", bKey)\n";
    } else {
      code += "\t\tval := obj." + namer_.Function(field.name) + "()\n";
      code += "\t\tcomp := 0\n";
      code += "\t\tif val > key {\n";
      code += "\t\t\tcomp = 1\n";
      code += "\t\t} else if val < key {\n";
      code += "\t\t\tcomp = -1\n";
      code += "\t\t}\n";
    }
    code += "\t\tif comp > 0 {\n";
    code += "\t\t\tspan = middle\n";
    code += "\t\t} else if comp < 0 {\n";
    code += "\t\t\tmiddle += 1\n";
    code += "\t\t\tstart += middle\n";
    code += "\t\t\tspan -= middle\n";
    code += "\t\t} else {\n";
    code += "\t\t\trcv.Init(buf, tableOffset)\n";
    code += "\t\t\treturn true\n";
    code += "\t\t}\n";
    code += "\t}\n";
    code += "\treturn false\n";
    code += "}\n\n";
  }

  void GenNativeStruct(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    code += "type " + NativeName(struct_def) + " struct {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.deprecated) continue;
      if (IsScalar(field.value.type.base_type) &&
          field.value.type.enum_def != nullptr &&
          field.value.type.enum_def->is_union)
        continue;
      code += "\t" + namer_.Field(field) + " ";
      if (field.IsScalarOptional()) { code += "*"; }
      code += NativeType(field.value.type) + " `json:\"" + field.name + "\"`" +
              "\n";
    }
    code += "}\n\n";

    if (!struct_def.fixed) {
      GenNativeTablePack(struct_def, code_ptr);
      GenNativeTableUnPack(struct_def, code_ptr);
    } else {
      GenNativeStructPack(struct_def, code_ptr);
      GenNativeStructUnPack(struct_def, code_ptr);
    }
  }

  void GenNativeUnion(const EnumDef &enum_def, std::string *code_ptr) {
    if (enum_def.generated) return;

    std::string &code = *code_ptr;
    code += "type " + NativeName(enum_def) + " struct {\n";
    code += "\tType " + namer_.Type(enum_def) + "\n";
    code += "\tValue interface{}\n";
    code += "}\n\n";
  }

  void GenNativeUnionPack(const EnumDef &enum_def, std::string *code_ptr) {
    if (enum_def.generated) return;

    std::string &code = *code_ptr;
    code += "func (t *" + NativeName(enum_def) +
            ") Pack(builder *flatbuffers.Builder) flatbuffers.UOffsetT {\n";
    code += "\tif t == nil {\n\t\treturn 0\n\t}\n";

    code += "\tswitch t.Type {\n";
    for (auto it2 = enum_def.Vals().begin(); it2 != enum_def.Vals().end();
         ++it2) {
      const EnumVal &ev = **it2;
      if (ev.IsZero()) continue;
      code += "\tcase " + namer_.EnumVariant(enum_def, ev) + ":\n";
      code += "\t\treturn t.Value.(" + NativeType(ev.union_type) +
              ").Pack(builder)\n";
    }
    code += "\t}\n";
    code += "\treturn 0\n";
    code += "}\n\n";
  }

  void GenNativeUnionUnPack(const EnumDef &enum_def, std::string *code_ptr) {
    if (enum_def.generated) return;

    std::string &code = *code_ptr;

    code += "func (rcv " + namer_.Type(enum_def) +
            ") UnPack(table flatbuffers.Table) *" + NativeName(enum_def) +
            " {\n";
    code += "\tswitch rcv {\n";

    for (auto it2 = enum_def.Vals().begin(); it2 != enum_def.Vals().end();
         ++it2) {
      const EnumVal &ev = **it2;
      if (ev.IsZero()) continue;
      code += "\tcase " + namer_.EnumVariant(enum_def, ev) + ":\n";
      code += "\t\tvar x " +
              WrapInNameSpaceAndTrack(ev.union_type.struct_def,
                                      ev.union_type.struct_def->name) +
              "\n";
      code += "\t\tx.Init(table.Bytes, table.Pos)\n";

      code += "\t\treturn &" +
              WrapInNameSpaceAndTrack(&enum_def, NativeName(enum_def)) +
              "{Type: " + namer_.EnumVariant(enum_def, ev) +
              ", Value: x.UnPack()}\n";
    }
    code += "\t}\n";
    code += "\treturn nil\n";
    code += "}\n\n";
  }

  void GenNativeTablePack(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;
    const std::string struct_type = namer_.Type(struct_def);

    code += "func (t *" + NativeName(struct_def) +
            ") Pack(builder *flatbuffers.Builder) flatbuffers.UOffsetT {\n";
    code += "\tif t == nil {\n\t\treturn 0\n\t}\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.deprecated) continue;
      if (IsScalar(field.value.type.base_type)) continue;

      const std::string field_field = namer_.Field(field);
      const std::string field_var = namer_.Variable(field);
      const std::string offset = field_var + "Offset";

      if (IsString(field.value.type)) {
        code += "\t" + offset + " := flatbuffers.UOffsetT(0)\n";
        code += "\tif t." + field_field + " != \"\" {\n";
        code += "\t\t" + offset + " = builder.CreateString(t." + field_field +
                ")\n";
        code += "\t}\n";
      } else if (IsVector(field.value.type) &&
                 field.value.type.element == BASE_TYPE_UCHAR &&
                 field.value.type.enum_def == nullptr) {
        code += "\t" + offset + " := flatbuffers.UOffsetT(0)\n";
        code += "\tif t." + field_field + " != nil {\n";
        code += "\t\t" + offset + " = builder.CreateByteString(t." +
                field_field + ")\n";
        code += "\t}\n";
      } else if (IsVector(field.value.type)) {
        code += "\t" + offset + " := flatbuffers.UOffsetT(0)\n";
        code += "\tif t." + field_field + " != nil {\n";
        std::string length = field_var + "Length";
        std::string offsets = field_var + "Offsets";
        code += "\t\t" + length + " := len(t." + field_field + ")\n";
        if (field.value.type.element == BASE_TYPE_STRING) {
          code += "\t\t" + offsets + " := make([]flatbuffers.UOffsetT, " +
                  length + ")\n";
          code += "\t\tfor j := 0; j < " + length + "; j++ {\n";
          code += "\t\t\t" + offsets + "[j] = builder.CreateString(t." +
                  field_field + "[j])\n";
          code += "\t\t}\n";
        } else if (field.value.type.element == BASE_TYPE_STRUCT &&
                   !field.value.type.struct_def->fixed) {
          code += "\t\t" + offsets + " := make([]flatbuffers.UOffsetT, " +
                  length + ")\n";
          code += "\t\tfor j := 0; j < " + length + "; j++ {\n";
          code += "\t\t\t" + offsets + "[j] = t." + field_field +
                  "[j].Pack(builder)\n";
          code += "\t\t}\n";
        }
        code += "\t\t" + struct_type + "Start" + namer_.Function(field) +
                "Vector(builder, " + length + ")\n";
        code += "\t\tfor j := " + length + " - 1; j >= 0; j-- {\n";
        if (IsScalar(field.value.type.element)) {
          code += "\t\t\tbuilder.Prepend" +
                  namer_.Method(GenTypeBasic(field.value.type.VectorType())) +
                  "(" +
                  CastToBaseType(field.value.type.VectorType(),
                                 "t." + field_field + "[j]") +
                  ")\n";
        } else if (field.value.type.element == BASE_TYPE_STRUCT &&
                   field.value.type.struct_def->fixed) {
          code += "\t\t\tt." + field_field + "[j].Pack(builder)\n";
        } else {
          code += "\t\t\tbuilder.PrependUOffsetT(" + offsets + "[j])\n";
        }
        code += "\t\t}\n";
        code += "\t\t" + offset + " = builder.EndVector(" + length + ")\n";
        code += "\t}\n";
      } else if (field.value.type.base_type == BASE_TYPE_STRUCT) {
        if (field.value.type.struct_def->fixed) continue;
        code += "\t" + offset + " := t." + field_field + ".Pack(builder)\n";
      } else if (field.value.type.base_type == BASE_TYPE_UNION) {
        code += "\t" + offset + " := t." + field_field + ".Pack(builder)\n\n";
      } else {
        FLATBUFFERS_ASSERT(0);
      }
    }
    code += "\t" + struct_type + "Start(builder)\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.deprecated) continue;
      const std::string field_field = namer_.Field(field);
      const std::string field_fn = namer_.Function(field);
      const std::string offset = namer_.Variable(field) + "Offset";

      if (IsScalar(field.value.type.base_type)) {
        std::string prefix;
        if (field.IsScalarOptional()) {
          code += "\tif t." + field_field + " != nil {\n\t";
          prefix = "*";
        }
        if (field.value.type.enum_def == nullptr ||
            !field.value.type.enum_def->is_union) {
          code += "\t" + struct_type + "Add" + field_fn + "(builder, " +
                  prefix + "t." + field_field + ")\n";
        }
        if (field.IsScalarOptional()) { code += "\t}\n"; }
      } else {
        if (field.value.type.base_type == BASE_TYPE_STRUCT &&
            field.value.type.struct_def->fixed) {
          code += "\t" + offset + " := t." + field_field + ".Pack(builder)\n";
        } else if (field.value.type.enum_def != nullptr &&
                   field.value.type.enum_def->is_union) {
          code += "\tif t." + field_field + " != nil {\n";
          code += "\t\t" + struct_type + "Add" +
                  namer_.Method(field.name + UnionTypeFieldSuffix()) +
                  "(builder, t." + field_field + ".Type)\n";
          code += "\t}\n";
        }
        code += "\t" + struct_type + "Add" + field_fn + "(builder, " + offset +
                ")\n";
      }
    }
    code += "\treturn " + struct_type + "End(builder)\n";
    code += "}\n\n";
  }

  void GenNativeTableUnPack(const StructDef &struct_def,
                            std::string *code_ptr) {
    std::string &code = *code_ptr;
    const std::string struct_type = namer_.Type(struct_def);

    code += "func (rcv *" + struct_type + ") UnPackTo(t *" +
            NativeName(struct_def) + ") {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.deprecated) continue;
      const std::string field_field = namer_.Field(field);
      const std::string field_var = namer_.Variable(field);
      const std::string length = field_var + "Length";
      if (IsScalar(field.value.type.base_type)) {
        if (field.value.type.enum_def != nullptr &&
            field.value.type.enum_def->is_union)
          continue;
        code += "\tt." + field_field + " = rcv." + field_field + "()\n";
      } else if (IsString(field.value.type)) {
        code += "\tt." + field_field + " = string(rcv." + field_field + "())\n";
      } else if (IsVector(field.value.type) &&
                 field.value.type.element == BASE_TYPE_UCHAR &&
                 field.value.type.enum_def == nullptr) {
        code += "\tt." + field_field + " = rcv." + field_field + "Bytes()\n";
      } else if (IsVector(field.value.type)) {
        code += "\t" + length + " := rcv." + field_field + "Length()\n";
        code += "\tt." + field_field + " = make(" +
                NativeType(field.value.type) + ", " + length + ")\n";
        code += "\tfor j := 0; j < " + length + "; j++ {\n";
        if (field.value.type.element == BASE_TYPE_STRUCT) {
          code += "\t\tx := " +
                  WrapInNameSpaceAndTrack(field.value.type.struct_def,
                                          field.value.type.struct_def->name) +
                  "{}\n";
          code += "\t\trcv." + field_field + "(&x, j)\n";
        }
        code += "\t\tt." + field_field + "[j] = ";
        if (IsScalar(field.value.type.element)) {
          code += "rcv." + field_field + "(j)";
        } else if (field.value.type.element == BASE_TYPE_STRING) {
          code += "string(rcv." + field_field + "(j))";
        } else if (field.value.type.element == BASE_TYPE_STRUCT) {
          code += "x.UnPack()";
        } else {
          // TODO(iceboy): Support vector of unions.
          FLATBUFFERS_ASSERT(0);
        }
        code += "\n";
        code += "\t}\n";
      } else if (field.value.type.base_type == BASE_TYPE_STRUCT) {
        code +=
            "\tt." + field_field + " = rcv." + field_field + "(nil).UnPack()\n";
      } else if (field.value.type.base_type == BASE_TYPE_UNION) {
        const std::string field_table = field_var + "Table";
        code += "\t" + field_table + " := flatbuffers.Table{}\n";
        code +=
            "\tif rcv." + namer_.Method(field) + "(&" + field_table + ") {\n";
        code += "\t\tt." + field_field + " = rcv." +
                namer_.Method(field.name + UnionTypeFieldSuffix()) +
                "().UnPack(" + field_table + ")\n";
        code += "\t}\n";
      } else {
        FLATBUFFERS_ASSERT(0);
      }
    }
    code += "}\n\n";

    code += "func (rcv *" + struct_type + ") UnPack() *" +
            NativeName(struct_def) + " {\n";
    code += "\tif rcv == nil {\n\t\treturn nil\n\t}\n";
    code += "\tt := &" + NativeName(struct_def) + "{}\n";
    code += "\trcv.UnPackTo(t)\n";
    code += "\treturn t\n";
    code += "}\n\n";
  }

  void GenNativeStructPack(const StructDef &struct_def, std::string *code_ptr) {
    std::string &code = *code_ptr;

    code += "func (t *" + NativeName(struct_def) +
            ") Pack(builder *flatbuffers.Builder) flatbuffers.UOffsetT {\n";
    code += "\tif t == nil {\n\t\treturn 0\n\t}\n";
    code += "\treturn Create" + namer_.Type(struct_def) + "(builder";
    StructPackArgs(struct_def, "", code_ptr);
    code += ")\n";
    code += "}\n";
  }

  void StructPackArgs(const StructDef &struct_def, const char *nameprefix,
                      std::string *code_ptr) {
    std::string &code = *code_ptr;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.value.type.base_type == BASE_TYPE_STRUCT) {
        StructPackArgs(*field.value.type.struct_def,
                       (nameprefix + namer_.Field(field) + ".").c_str(),
                       code_ptr);
      } else {
        code += std::string(", t.") + nameprefix + namer_.Field(field);
      }
    }
  }

  void GenNativeStructUnPack(const StructDef &struct_def,
                             std::string *code_ptr) {
    std::string &code = *code_ptr;

    code += "func (rcv *" + namer_.Type(struct_def) + ") UnPackTo(t *" +
            NativeName(struct_def) + ") {\n";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      const FieldDef &field = **it;
      if (field.value.type.base_type == BASE_TYPE_STRUCT) {
        code += "\tt." + namer_.Field(field) + " = rcv." +
                namer_.Method(field) + "(nil).UnPack()\n";
      } else {
        code += "\tt." + namer_.Field(field) + " = rcv." +
                namer_.Method(field) + "()\n";
      }
    }
    code += "}\n\n";

    code += "func (rcv *" + namer_.Type(struct_def) + ") UnPack() *" +
            NativeName(struct_def) + " {\n";
    code += "\tif rcv == nil {\n\t\treturn nil\n\t}\n";
    code += "\tt := &" + NativeName(struct_def) + "{}\n";
    code += "\trcv.UnPackTo(t)\n";
    code += "\treturn t\n";
    code += "}\n\n";
  }

  // Generate enum declarations.
  void GenEnum(const EnumDef &enum_def, std::string *code_ptr) {
    if (enum_def.generated) return;

    auto max_name_length = MaxNameLength(enum_def);
    cur_name_space_ = enum_def.defined_namespace;

    GenComment(enum_def.doc_comment, code_ptr, nullptr);
    GenEnumType(enum_def, code_ptr);
    BeginEnum(code_ptr);
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      const EnumVal &ev = **it;
      GenComment(ev.doc_comment, code_ptr, nullptr, "\t");
      EnumMember(enum_def, ev, max_name_length, code_ptr);
    }
    EndEnum(code_ptr);

    BeginEnumNames(enum_def, code_ptr);
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      const EnumVal &ev = **it;
      EnumNameMember(enum_def, ev, max_name_length, code_ptr);
    }
    EndEnumNames(code_ptr);

    BeginEnumValues(enum_def, code_ptr);
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      EnumValueMember(enum_def, ev, max_name_length, code_ptr);
    }
    EndEnumValues(code_ptr);

    EnumStringer(enum_def, code_ptr);
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetter(const Type &type) {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "rcv._tab.ByteVector";
      case BASE_TYPE_UNION: return "rcv._tab.Union";
      case BASE_TYPE_VECTOR: return GenGetter(type.VectorType());
      default: return "rcv._tab.Get" + namer_.Function(GenTypeBasic(type));
    }
  }

  // Returns the method name for use with add/put calls.
  std::string GenMethod(const FieldDef &field) {
    return IsScalar(field.value.type.base_type)
               ? namer_.Method(GenTypeBasic(field.value.type))
               : (IsStruct(field.value.type) ? "Struct" : "UOffsetT");
  }

  std::string GenTypeBasic(const Type &type) {
    // clang-format off
    static const char *ctypename[] = {
      #define FLATBUFFERS_TD(ENUM, IDLTYPE, CTYPE, JTYPE, GTYPE, ...) \
        #GTYPE,
        FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
      #undef FLATBUFFERS_TD
    };
    // clang-format on
    return ctypename[type.base_type];
  }

  std::string GenTypePointer(const Type &type) {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "[]byte";
      case BASE_TYPE_VECTOR: return GenTypeGet(type.VectorType());
      case BASE_TYPE_STRUCT:
        return WrapInNameSpaceAndTrack(type.struct_def, type.struct_def->name);
      case BASE_TYPE_UNION:
        // fall through
      default: return "*flatbuffers.Table";
    }
  }

  std::string GenTypeGet(const Type &type) {
    if (type.enum_def != nullptr) { return GetEnumTypeName(*type.enum_def); }
    return IsScalar(type.base_type) ? GenTypeBasic(type) : GenTypePointer(type);
  }

  std::string TypeName(const FieldDef &field) {
    std::string prefix;
    if (field.IsScalarOptional()) { prefix = "*"; }
    return prefix + GenTypeGet(field.value.type);
  }

  // If type is an enum, returns value with a cast to the enum type, otherwise
  // returns value as-is.
  std::string CastToEnum(const Type &type, std::string value) {
    if (type.enum_def == nullptr) {
      return value;
    } else {
      return GenTypeGet(type) + "(" + value + ")";
    }
  }

  // If type is an enum, returns value with a cast to the enum base type,
  // otherwise returns value as-is.
  std::string CastToBaseType(const Type &type, std::string value) {
    if (type.enum_def == nullptr) {
      return value;
    } else {
      return GenTypeBasic(type) + "(" + value + ")";
    }
  }

  std::string GenConstant(const FieldDef &field) {
    if (field.IsScalarOptional()) { return "nil"; }
    switch (field.value.type.base_type) {
      case BASE_TYPE_BOOL:
        return field.value.constant == "0" ? "false" : "true";
      case BASE_TYPE_FLOAT:
      case BASE_TYPE_DOUBLE: {
        const std::string float_type =
            field.value.type.base_type == BASE_TYPE_FLOAT ? "float32"
                                                          : "float64";
        if (StringIsFlatbufferNan(field.value.constant)) {
          needs_math_import_ = true;
          return float_type + "(math.NaN())";
        } else if (StringIsFlatbufferPositiveInfinity(field.value.constant)) {
          needs_math_import_ = true;
          return float_type + "(math.Inf(1))";
        } else if (StringIsFlatbufferNegativeInfinity(field.value.constant)) {
          needs_math_import_ = true;
          return float_type + "(math.Inf(-1))";
        }
        return field.value.constant;
      }
      default: return field.value.constant;
    }
  }

  std::string NativeName(const StructDef &struct_def) const {
    return namer_.ObjectType(struct_def);
  }

  std::string NativeName(const EnumDef &enum_def) const {
    return namer_.ObjectType(enum_def);
  }

  std::string NativeType(const Type &type) {
    if (IsScalar(type.base_type)) {
      if (type.enum_def == nullptr) {
        return GenTypeBasic(type);
      } else {
        return GetEnumTypeName(*type.enum_def);
      }
    } else if (IsString(type)) {
      return "string";
    } else if (IsVector(type)) {
      return "[]" + NativeType(type.VectorType());
    } else if (type.base_type == BASE_TYPE_STRUCT) {
      return "*" + WrapInNameSpaceAndTrack(type.struct_def,
                                           NativeName(*type.struct_def));
    } else if (type.base_type == BASE_TYPE_UNION) {
      return "*" +
             WrapInNameSpaceAndTrack(type.enum_def, NativeName(*type.enum_def));
    }
    FLATBUFFERS_ASSERT(0);
    return std::string();
  }

  // Create a struct with a builder and the struct's arguments.
  void GenStructBuilder(const StructDef &struct_def, std::string *code_ptr) {
    BeginBuilderArgs(struct_def, code_ptr);
    StructBuilderArgs(struct_def, "", code_ptr);
    EndBuilderArgs(code_ptr);

    StructBuilderBody(struct_def, "", code_ptr);
    EndBuilderBody(code_ptr);
  }

  // Begin by declaring namespace and imports.
  void BeginFile(const std::string &name_space_name, const bool needs_imports,
                 const bool is_enum, std::string *code_ptr) {
    std::string &code = *code_ptr;
    code = code +
           "// Code generated by the FlatBuffers compiler. DO NOT EDIT.\n\n";
    code += "package " + name_space_name + "\n\n";
    if (needs_imports) {
      code += "import (\n";
      // standard imports, in alphabetical order for go fmt
      if (needs_bytes_import_) code += "\t\"bytes\"\n";
      if (!parser_.opts.go_import.empty()) {
        code += "\tflatbuffers \"" + parser_.opts.go_import + "\"\n";
      } else {
        code += "\tflatbuffers \"github.com/google/flatbuffers/go\"\n";
      }
      // math is needed to support non-finite scalar default values.
      if (needs_math_import_) { code += "\t\"math\"\n"; }
      if (is_enum) { code += "\t\"strconv\"\n"; }

      if (tracked_imported_namespaces_.size() > 0) {
        code += "\n";
        for (auto it = tracked_imported_namespaces_.begin();
             it != tracked_imported_namespaces_.end(); ++it) {
          if ((*it)->defined_namespace->components.empty()) {
            code += "\t" + (*it)->name + " \"" + (*it)->name + "\"\n";
          } else {
            code += "\t" + NamespaceImportName((*it)->defined_namespace) +
                    " \"" + NamespaceImportPath((*it)->defined_namespace) +
                    "\"\n";
          }
        }
      }
      code += ")\n\n";
    } else {
      if (is_enum) { code += "import \"strconv\"\n\n"; }
      if (needs_math_import_) {
        // math is needed to support non-finite scalar default values.
        code += "import \"math\"\n\n";
      }
    }
  }

  // Resets the needed imports before generating a new file.
  void ResetImports() {
    tracked_imported_namespaces_.clear();
    needs_bytes_import_ = false;
    needs_math_import_ = false;
  }

  // Save out the generated code for a Go Table type.
  bool SaveType(const Definition &def, const std::string &classcode,
                const bool needs_imports, const bool is_enum) {
    if (!classcode.length()) return true;

    Namespace &ns = go_namespace_.components.empty() ? *def.defined_namespace
                                                     : go_namespace_;
    std::string code = "";
    BeginFile(ns.components.empty() ? def.name : LastNamespacePart(ns),
              needs_imports, is_enum, &code);
    code += classcode;
    // Strip extra newlines at end of file to make it gofmt-clean.
    while (code.length() > 2 && code.substr(code.length() - 2) == "\n\n") {
      code.pop_back();
    }
    std::string directory = namer_.Directories(ns);
    std::string file = namer_.File(def, SkipFile::Suffix);
    EnsureDirExists(directory);
    std::string filename = directory + file;
    return SaveFile(filename.c_str(), code, false);
  }

  // Create the full name of the imported namespace (format: A__B__C).
  std::string NamespaceImportName(const Namespace *ns) const {
    return namer_.Namespace(*ns);
  }

  // Create the full path for the imported namespace (format: A/B/C).
  std::string NamespaceImportPath(const Namespace *ns) const {
    std::string path =
        namer_.Directories(*ns, SkipDir::OutputPathAndTrailingPathSeparator);
    if (!parser_.opts.go_module_name.empty()) {
      path = parser_.opts.go_module_name + "/" + path;
    }
    return path;
  }

  // Ensure that a type is prefixed with its go package import name if it is
  // used outside of its namespace.
  std::string WrapInNameSpaceAndTrack(const Definition *def,
                                      const std::string &name) {
    if (CurrentNameSpace() == def->defined_namespace) return name;
    tracked_imported_namespaces_.insert(def);
    if (def->defined_namespace->components.empty())
      return def->name + "." + name;
    else
      return NamespaceImportName(def->defined_namespace) + "." + name;
  }

  const Namespace *CurrentNameSpace() const { return cur_name_space_; }

  static size_t MaxNameLength(const EnumDef &enum_def) {
    size_t max = 0;
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      max = std::max((*it)->name.length(), max);
    }
    return max;
  }
};
}  // namespace go

static bool GenerateGo(const Parser &parser, const std::string &path,
                       const std::string &file_name) {
  go::GoGenerator generator(parser, path, file_name, parser.opts.go_namespace);
  return generator.generate();
}

namespace {

class GoCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GenerateGo(parser, path, filename)) { return Status::ERROR; }
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
    if (!GenerateGoGRPC(parser, path, filename)) { return Status::ERROR; }
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

  IDLOptions::Language Language() const override { return IDLOptions::kGo; }

  std::string LanguageName() const override { return "Go"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewGoCodeGenerator() {
  return std::unique_ptr<GoCodeGenerator>(new GoCodeGenerator());
}

}  // namespace flatbuffers
