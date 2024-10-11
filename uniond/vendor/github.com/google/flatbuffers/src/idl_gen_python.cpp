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

#include "idl_gen_python.h"

#include <cctype>
#include <set>
#include <string>
#include <unordered_set>
#include <vector>

#include "flatbuffers/code_generators.h"
#include "flatbuffers/flatbuffers.h"
#include "flatbuffers/idl.h"
#include "flatbuffers/util.h"
#include "idl_namer.h"

namespace flatbuffers {
namespace python {

namespace {

typedef std::pair<std::string, std::string> ImportMapEntry;
typedef std::set<ImportMapEntry> ImportMap;

static std::set<std::string> PythonKeywords() {
  return { "False", "None",   "True",     "and",   "as",     "assert",
           "break", "class",  "continue", "def",   "del",    "elif",
           "else",  "except", "finally",  "for",   "from",   "global",
           "if",    "import", "in",       "is",    "lambda", "nonlocal",
           "not",   "or",     "pass",     "raise", "return", "try",
           "while", "with",   "yield" };
}

static Namer::Config PythonDefaultConfig() {
  return { /*types=*/Case::kKeep,
           /*constants=*/Case::kScreamingSnake,
           /*methods=*/Case::kUpperCamel,
           /*functions=*/Case::kUpperCamel,
           /*fields=*/Case::kLowerCamel,
           /*variable=*/Case::kLowerCamel,
           /*variants=*/Case::kKeep,
           /*enum_variant_seperator=*/".",
           /*escape_keywords=*/Namer::Config::Escape::BeforeConvertingCase,
           /*namespaces=*/Case::kKeep,  // Packages in python.
           /*namespace_seperator=*/".",
           /*object_prefix=*/"",
           /*object_suffix=*/"T",
           /*keyword_prefix=*/"",
           /*keyword_suffix=*/"_",
           /*filenames=*/Case::kKeep,
           /*directories=*/Case::kKeep,
           /*output_path=*/"",
           /*filename_suffix=*/"",
           /*filename_extension=*/".py" };
}

// Hardcode spaces per indentation.
static const CommentConfig def_comment = { nullptr, "#", nullptr };
static const std::string Indent = "    ";

}  // namespace

class PythonGenerator : public BaseGenerator {
 public:
  PythonGenerator(const Parser &parser, const std::string &path,
                  const std::string &file_name)
      : BaseGenerator(parser, path, file_name, "" /* not used */,
                      "" /* not used */, "py"),
        float_const_gen_("float('nan')", "float('inf')", "float('-inf')"),
        namer_(WithFlagOptions(PythonDefaultConfig(), parser.opts, path),
               PythonKeywords()) {}

  // Most field accessors need to retrieve and test the field offset first,
  // this is the prefix code for that.
  std::string OffsetPrefix(const FieldDef &field, bool new_line = true) const {
    return "\n" + Indent + Indent +
           "o = flatbuffers.number_types.UOffsetTFlags.py_type" +
           "(self._tab.Offset(" + NumToString(field.value.offset) + "))\n" +
           Indent + Indent + "if o != 0:" + (new_line ? "\n" : "");
  }

  // Begin a class declaration.
  void BeginClass(const StructDef &struct_def, std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += "class " + namer_.Type(struct_def) + "(object):\n";
    code += Indent + "__slots__ = ['_tab']";
    code += "\n\n";
  }

  // Begin enum code with a class declaration.
  void BeginEnum(const EnumDef &enum_def, std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += "class " + namer_.Type(enum_def) + "(object):\n";
  }

  // Starts a new line and then indents.
  std::string GenIndents(int num) const {
    return "\n" + std::string(num * Indent.length(), ' ');
  }

  // A single enum member.
  void EnumMember(const EnumDef &enum_def, const EnumVal &ev,
                  std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += Indent;
    code += namer_.Variant(ev);
    code += " = ";
    code += enum_def.ToString(ev) + "\n";
  }

  // Initialize a new struct or table from existing data.
  void NewRootTypeFromBuffer(const StructDef &struct_def,
                             std::string *code_ptr) const {
    auto &code = *code_ptr;
    const std::string struct_type = namer_.Type(struct_def);

    code += Indent + "@classmethod\n";
    code += Indent + "def GetRootAs";
    if (parser_.opts.python_typing) {
      code += "(cls, buf, offset: int = 0):";
    } else {
      code += "(cls, buf, offset=0):";
    }
    code += "\n";
    code += Indent + Indent;
    code += "n = flatbuffers.encode.Get";
    code += "(flatbuffers.packer.uoffset, buf, offset)\n";
    code += Indent + Indent + "x = " + struct_type + "()\n";
    code += Indent + Indent + "x.Init(buf, n + offset)\n";
    code += Indent + Indent + "return x\n";
    code += "\n";

    if (!parser_.opts.python_no_type_prefix_suffix) {
      // Add an alias with the old name
      code += Indent + "@classmethod\n";
      code +=
          Indent + "def GetRootAs" + struct_type + "(cls, buf, offset=0):\n";
      code += Indent + Indent +
              "\"\"\"This method is deprecated. Please switch to "
              "GetRootAs.\"\"\"\n";
      code += Indent + Indent + "return cls.GetRootAs(buf, offset)\n";
    }
  }

  // Initialize an existing object with other data, to avoid an allocation.
  void InitializeExisting(const StructDef &struct_def,
                          std::string *code_ptr) const {
    auto &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    if (parser_.opts.python_typing) {
      code += "Init(self, buf: bytes, pos: int):\n";
    } else {
      code += "Init(self, buf, pos):\n";
    }
    code += Indent + Indent + "self._tab = flatbuffers.table.Table(buf, pos)\n";
    code += "\n";
  }

  // Get the length of a vector.
  void GetVectorLen(const StructDef &struct_def, const FieldDef &field,
                    std::string *code_ptr) const {
    auto &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field) + "Length(self)";
    if (parser_.opts.python_typing) { code += " -> int"; }
    code += ":";
    if (!IsArray(field.value.type)) {
      code += OffsetPrefix(field, false);
      code += GenIndents(3) + "return self._tab.VectorLen(o)";
      code += GenIndents(2) + "return 0\n\n";
    } else {
      code += GenIndents(2) + "return " +
              NumToString(field.value.type.fixed_length) + "\n\n";
    }
  }

  // Determines whether a vector is none or not.
  void GetVectorIsNone(const StructDef &struct_def, const FieldDef &field,
                       std::string *code_ptr) const {
    auto &code = *code_ptr;

    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field) + "IsNone(self)";
    if (parser_.opts.python_typing) { code += " -> bool"; }
    code += ":";
    if (!IsArray(field.value.type)) {
      code += GenIndents(2) +
              "o = flatbuffers.number_types.UOffsetTFlags.py_type" +
              "(self._tab.Offset(" + NumToString(field.value.offset) + "))";
      code += GenIndents(2) + "return o == 0";
    } else {
      // assume that we always have an array as memory is preassigned
      code += GenIndents(2) + "return False";
    }
    code += "\n\n";
  }

  // Get the value of a struct's scalar.
  void GetScalarFieldOfStruct(const StructDef &struct_def,
                              const FieldDef &field,
                              std::string *code_ptr) const {
    auto &code = *code_ptr;
    std::string getter = GenGetter(field.value.type);
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    code += "(self): return " + getter;
    code += "self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(";
    code += NumToString(field.value.offset) + "))\n";
  }

  // Get the value of a table's scalar.
  void GetScalarFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                             std::string *code_ptr) const {
    auto &code = *code_ptr;
    std::string getter = GenGetter(field.value.type);
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    code += "(self):";
    code += OffsetPrefix(field);
    getter += "o + self._tab.Pos)";
    auto is_bool = IsBool(field.value.type.base_type);
    if (is_bool) { getter = "bool(" + getter + ")"; }
    code += Indent + Indent + Indent + "return " + getter + "\n";
    std::string default_value;
    if (field.IsScalarOptional()) {
      default_value = "None";
    } else if (is_bool) {
      default_value = field.value.constant == "0" ? "False" : "True";
    } else {
      default_value = IsFloat(field.value.type.base_type)
                          ? float_const_gen_.GenFloatConstant(field)
                          : field.value.constant;
    }
    code += Indent + Indent + "return " + default_value + "\n\n";
  }

  // Get a struct by initializing an existing struct.
  // Specific to Struct.
  void GetStructFieldOfStruct(const StructDef &struct_def,
                              const FieldDef &field,
                              std::string *code_ptr) const {
    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    code += "(self, obj):\n";
    code += Indent + Indent + "obj.Init(self._tab.Bytes, self._tab.Pos + ";
    code += NumToString(field.value.offset) + ")";
    code += "\n" + Indent + Indent + "return obj\n\n";
  }

  // Get the value of a fixed size array.
  void GetArrayOfStruct(const StructDef &struct_def, const FieldDef &field,
                        std::string *code_ptr, ImportMap &imports) const {
    auto &code = *code_ptr;
    const auto vec_type = field.value.type.VectorType();
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);

    const ImportMapEntry import_entry = {
      GenPackageReference(field.value.type), TypeName(field)
    };

    if (parser_.opts.python_typing) {
      const std::string return_type = ReturnType(struct_def, field);
      code += "(self, i: int)";
      code += " -> " + return_type + ":";

      imports.insert(import_entry);
    } else {
      code += "(self, i):";
    }

    if (parser_.opts.include_dependence_headers &&
        !parser_.opts.python_typing) {
      code += GenIndents(2);
      code += "from " + import_entry.first + " import " + import_entry.second +
              "\n";
    }

    code += GenIndents(2) + "obj = " + TypeName(field) + "()";
    code += GenIndents(2) + "obj.Init(self._tab.Bytes, self._tab.Pos + ";
    code += NumToString(field.value.offset) + " + i * ";
    code += NumToString(InlineSize(vec_type));
    code += ")" + GenIndents(2) + "return obj\n\n";
  }

  // Get the value of a vector's non-struct member. Uses a named return
  // argument to conveniently set the zero value for the result.
  void GetArrayOfNonStruct(const StructDef &struct_def, const FieldDef &field,
                           std::string *code_ptr) const {
    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    code += "(self, j = None):";
    code += GenIndents(2) + "if j is None:";
    code += GenIndents(3) + "return [" + GenGetter(field.value.type);
    code += "self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(";
    code += NumToString(field.value.offset) + " + i * ";
    code += NumToString(InlineSize(field.value.type.VectorType()));
    code += ")) for i in range(";
    code += "self." + namer_.Method(field) + "Length()" + ")]";
    code += GenIndents(2) + "elif j >= 0 and j < self." + namer_.Method(field) +
            "Length():";
    code += GenIndents(3) + "return " + GenGetter(field.value.type);
    code += "self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(";
    code += NumToString(field.value.offset) + " + j * ";
    code += NumToString(InlineSize(field.value.type.VectorType()));
    code += "))";
    code += GenIndents(2) + "else:";
    code += GenIndents(3) + "return None\n\n";
  }

  // Get a struct by initializing an existing struct.
  // Specific to Table.
  void GetStructFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                             std::string *code_ptr, ImportMap &imports) const {
    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field) + "(self)";

    const ImportMapEntry import_entry = {
      GenPackageReference(field.value.type), TypeName(field)
    };

    if (parser_.opts.python_typing) {
      const std::string return_type = ReturnType(struct_def, field);
      code += " -> Optional[" + return_type + "]";
      imports.insert(ImportMapEntry{ "typing", "Optional" });
      imports.insert(import_entry);
    }
    code += ":";
    code += OffsetPrefix(field);
    if (field.value.type.struct_def->fixed) {
      code += Indent + Indent + Indent + "x = o + self._tab.Pos\n";
    } else {
      code += Indent + Indent + Indent;
      code += "x = self._tab.Indirect(o + self._tab.Pos)\n";
    }

    if (parser_.opts.include_dependence_headers &&
        !parser_.opts.python_typing) {
      code += Indent + Indent + Indent;
      code += "from " + import_entry.first + " import " + import_entry.second +
              "\n";
    }
    code += Indent + Indent + Indent + "obj = " + TypeName(field) + "()\n";
    code += Indent + Indent + Indent + "obj.Init(self._tab.Bytes, x)\n";
    code += Indent + Indent + Indent + "return obj\n";
    code += Indent + Indent + "return None\n\n";
  }

  // Get the value of a string.
  void GetStringField(const StructDef &struct_def, const FieldDef &field,
                      std::string *code_ptr, ImportMap &imports) const {
    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);

    if (parser_.opts.python_typing) {
      code += "(self) -> Optional[str]:";
      imports.insert(ImportMapEntry{ "typing", "Optional" });
    } else {
      code += "(self):";
    }

    code += OffsetPrefix(field);
    code += Indent + Indent + Indent + "return " + GenGetter(field.value.type);
    code += "o + self._tab.Pos)\n";
    code += Indent + Indent + "return None\n\n";
  }

  // Get the value of a union from an object.
  void GetUnionField(const StructDef &struct_def, const FieldDef &field,
                     std::string *code_ptr, ImportMap &imports) const {
    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    std::string return_ty = "flatbuffers.table.Table";

    bool is_native_table = TypeName(field) == "*flatbuffers.Table";
    ImportMapEntry import_entry;
    if (is_native_table) {
      import_entry = ImportMapEntry{ "flatbuffers.table", "Table" };
    } else {
      return_ty = TypeName(field);
      import_entry = ImportMapEntry{ GenPackageReference(field.value.type),
                                     TypeName(field) };
    }

    code += namer_.Method(field) + "(self)";
    if (parser_.opts.python_typing) {
      code += " -> Optional[" + return_ty + "]";
      imports.insert(ImportMapEntry{ "typing", "Optional" });
      imports.insert(import_entry);
    }
    code += ":";
    code += OffsetPrefix(field);

    if (!parser_.opts.python_typing) {
      code += Indent + Indent + Indent;
      code += "from " + import_entry.first + " import " + import_entry.second +
              "\n";
    }
    code += Indent + Indent + Indent + "obj = Table(bytearray(), 0)\n";
    code += Indent + Indent + Indent + GenGetter(field.value.type);
    code += "obj, o)\n" + Indent + Indent + Indent + "return obj\n";
    code += Indent + Indent + "return None\n\n";
  }

  // Generate the package reference when importing a struct or enum from its
  // module.
  std::string GenPackageReference(const Type &type) const {
    if (type.struct_def) {
      return namer_.NamespacedType(*type.struct_def);
    } else if (type.enum_def) {
      return namer_.NamespacedType(*type.enum_def);
    } else {
      return "." + GenTypeGet(type);
    }
  }

  // Get the value of a vector's struct member.
  void GetMemberOfVectorOfStruct(const StructDef &struct_def,
                                 const FieldDef &field, std::string *code_ptr,
                                 ImportMap &imports) const {
    auto &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();

    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    const ImportMapEntry import_entry = {
      GenPackageReference(field.value.type), TypeName(field)
    };

    if (parser_.opts.python_typing) {
      const std::string return_type = ReturnType(struct_def, field);
      code += "(self, j: int) -> Optional[" + return_type + "]";
      imports.insert(ImportMapEntry{ "typing", "Optional" });
      imports.insert(import_entry);
    } else {
      code += "(self, j)";
    }
    code += ":" + OffsetPrefix(field);
    code += Indent + Indent + Indent + "x = self._tab.Vector(o)\n";
    code += Indent + Indent + Indent;
    code += "x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * ";
    code += NumToString(InlineSize(vectortype)) + "\n";
    if (!(vectortype.struct_def->fixed)) {
      code += Indent + Indent + Indent + "x = self._tab.Indirect(x)\n";
    }
    if (parser_.opts.include_dependence_headers &&
        !parser_.opts.python_typing) {
      code += Indent + Indent + Indent;
      code += "from " + import_entry.first + " import " + import_entry.second +
              "\n";
    }
    code += Indent + Indent + Indent + "obj = " + TypeName(field) + "()\n";
    code += Indent + Indent + Indent + "obj.Init(self._tab.Bytes, x)\n";
    code += Indent + Indent + Indent + "return obj\n";
    code += Indent + Indent + "return None\n\n";
  }

  // Get the value of a vector's non-struct member. Uses a named return
  // argument to conveniently set the zero value for the result.
  void GetMemberOfVectorOfNonStruct(const StructDef &struct_def,
                                    const FieldDef &field,
                                    std::string *code_ptr) const {
    auto &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();

    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field);
    if (parser_.opts.python_typing) {
      code += "(self, j: int)";
    } else {
      code += "(self, j)";
    }
    code += ":";
    code += OffsetPrefix(field);
    code += Indent + Indent + Indent + "a = self._tab.Vector(o)\n";
    code += Indent + Indent + Indent;
    code += "return " + GenGetter(field.value.type);
    code += "a + flatbuffers.number_types.UOffsetTFlags.py_type(j * ";
    code += NumToString(InlineSize(vectortype)) + "))\n";
    if (IsString(vectortype)) {
      code += Indent + Indent + "return \"\"\n";
    } else {
      code += Indent + Indent + "return 0\n";
    }
    code += "\n";
  }

  // Returns a non-struct vector as a numpy array. Much faster
  // than iterating over the vector element by element.
  void GetVectorOfNonStructAsNumpy(const StructDef &struct_def,
                                   const FieldDef &field,
                                   std::string *code_ptr) const {
    auto &code = *code_ptr;
    auto vectortype = field.value.type.VectorType();

    // Currently, we only support accessing as numpy array if
    // the vector type is a scalar.
    if (!(IsScalar(vectortype.base_type))) { return; }

    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field) + "AsNumpy(self):";
    if (!IsArray(field.value.type)) {
      code += OffsetPrefix(field, false);

      code += GenIndents(3);
      code += "return ";
      code += "self._tab.GetVectorAsNumpy(flatbuffers.number_types.";
      code += namer_.Method(GenTypeGet(field.value.type));
      code += "Flags, o)";

      if (IsString(vectortype)) {
        code += GenIndents(2) + "return \"\"\n";
      } else {
        code += GenIndents(2) + "return 0\n";
      }
    } else {
      code += GenIndents(2) + "return ";
      code += "self._tab.GetArrayAsNumpy(flatbuffers.number_types.";
      code += namer_.Method(GenTypeGet(field.value.type.VectorType()));
      code += "Flags, self._tab.Pos + " + NumToString(field.value.offset) +
              ", " + NumToString("self." + namer_.Method(field) + "Length()") +
              ")\n";
    }
    code += "\n";
  }

  std::string NestedFlatbufferType(std::string unqualified_name) const {
    StructDef *nested_root = parser_.LookupStruct(unqualified_name);
    std::string qualified_name;
    if (nested_root == nullptr) {
      qualified_name = namer_.NamespacedType(
          parser_.current_namespace_->components, unqualified_name);
      // Double check qualified name just to be sure it exists.
      nested_root = parser_.LookupStruct(qualified_name);
    }
    FLATBUFFERS_ASSERT(nested_root);  // Guaranteed to exist by parser.
    return qualified_name;
  }

  // Returns a nested flatbuffer as itself.
  void GetVectorAsNestedFlatbuffer(const StructDef &struct_def,
                                   const FieldDef &field, std::string *code_ptr,
                                   ImportMap &imports) const {
    auto nested = field.attributes.Lookup("nested_flatbuffer");
    if (!nested) { return; }  // There is no nested flatbuffer.

    const std::string unqualified_name = nested->constant;
    std::string qualified_name = NestedFlatbufferType(unqualified_name);
    if (qualified_name.empty()) { qualified_name = nested->constant; }

    const ImportMapEntry import_entry = { qualified_name,
                                          unqualified_name };

    auto &code = *code_ptr;
    GenReceiver(struct_def, code_ptr);
    code += namer_.Method(field) + "NestedRoot(self)";
    if (parser_.opts.python_typing) {
      code += " -> Union[" + unqualified_name + ", int]";
      imports.insert(ImportMapEntry{ "typing", "Union" });
      imports.insert(import_entry);
    }
    code += ":";

    code += OffsetPrefix(field);

    if (!parser_.opts.python_typing) {
      code += Indent + Indent + Indent;
      code += "from " + import_entry.first + " import " + import_entry.second +
              "\n";
    }
    code += Indent + Indent + Indent + "return " + unqualified_name;
    code += ".GetRootAs";
    code += "(self._tab.Bytes, self._tab.Vector(o))\n";
    code += Indent + Indent + "return 0\n";
    code += "\n";
  }

  // Begin the creator function signature.
  void BeginBuilderArgs(const StructDef &struct_def,
                        std::string *code_ptr) const {
    auto &code = *code_ptr;

    code += "\n";
    code += "def Create" + namer_.Type(struct_def);
    code += "(builder";
  }

  // Recursively generate arguments for a constructor, to deal with nested
  // structs.
  void StructBuilderArgs(const StructDef &struct_def,
                         const std::string nameprefix,
                         const std::string namesuffix, bool has_field_name,
                         const std::string fieldname_suffix,
                         std::string *code_ptr) const {
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      const auto &type =
          IsArray(field_type) ? field_type.VectorType() : field_type;
      if (IsStruct(type)) {
        // Generate arguments for a struct inside a struct. To ensure names
        // don't clash, and to make it obvious these arguments are constructing
        // a nested struct, prefix the name with the field name.
        auto subprefix = nameprefix;
        if (has_field_name) {
          subprefix += namer_.Field(field) + fieldname_suffix;
        }
        StructBuilderArgs(*field.value.type.struct_def, subprefix, namesuffix,
                          has_field_name, fieldname_suffix, code_ptr);
      } else {
        auto &code = *code_ptr;
        code += std::string(", ") + nameprefix;
        if (has_field_name) { code += namer_.Field(field); }
        code += namesuffix;
      }
    }
  }

  // End the creator function signature.
  void EndBuilderArgs(std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += "):\n";
  }

  // Recursively generate struct construction statements and instert manual
  // padding.
  void StructBuilderBody(const StructDef &struct_def, const char *nameprefix,
                         std::string *code_ptr, size_t index = 0,
                         bool in_array = false) const {
    auto &code = *code_ptr;
    std::string indent(index * 4, ' ');
    code +=
        indent + "    builder.Prep(" + NumToString(struct_def.minalign) + ", ";
    code += NumToString(struct_def.bytesize) + ")\n";
    for (auto it = struct_def.fields.vec.rbegin();
         it != struct_def.fields.vec.rend(); ++it) {
      auto &field = **it;
      const auto &field_type = field.value.type;
      const auto &type =
          IsArray(field_type) ? field_type.VectorType() : field_type;
      if (field.padding)
        code +=
            indent + "    builder.Pad(" + NumToString(field.padding) + ")\n";
      if (IsStruct(field_type)) {
        StructBuilderBody(*field_type.struct_def,
                          (nameprefix + (namer_.Field(field) + "_")).c_str(),
                          code_ptr, index, in_array);
      } else {
        const auto index_var = "_idx" + NumToString(index);
        if (IsArray(field_type)) {
          code += indent + "    for " + index_var + " in range(";
          code += NumToString(field_type.fixed_length);
          code += " , 0, -1):\n";
          in_array = true;
        }
        if (IsStruct(type)) {
          StructBuilderBody(*field_type.struct_def,
                            (nameprefix + (namer_.Field(field) + "_")).c_str(),
                            code_ptr, index + 1, in_array);
        } else {
          code += IsArray(field_type) ? "    " : "";
          code += indent + "    builder.Prepend" + GenMethod(field) + "(";
          code += nameprefix + namer_.Variable(field);
          size_t array_cnt = index + (IsArray(field_type) ? 1 : 0);
          for (size_t i = 0; in_array && i < array_cnt; i++) {
            code += "[_idx" + NumToString(i) + "-1]";
          }
          code += ")\n";
        }
      }
    }
  }

  void EndBuilderBody(std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += "    return builder.Offset()\n";
  }

  // Get the value of a table's starting offset.
  void GetStartOfTable(const StructDef &struct_def,
                       std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_type = namer_.Type(struct_def);
    // Generate method with struct name.

    const auto name = parser_.opts.python_no_type_prefix_suffix
                          ? "Start"
                          : struct_type + "Start";

    code += "def " + name;
    if (parser_.opts.python_typing) {
      code += "(builder: flatbuffers.Builder):\n";
    } else {
      code += "(builder):\n";
    }

    code += Indent + "builder.StartObject(";
    code += NumToString(struct_def.fields.vec.size());
    code += ")\n\n";

    if (!parser_.opts.one_file && !parser_.opts.python_no_type_prefix_suffix) {
      // Generate method without struct name.
      if (parser_.opts.python_typing) {
        code += "def Start(builder: flatbuffers.Builder):\n";
      } else {
        code += "def Start(builder):\n";
      }
      code += Indent + struct_type + "Start(builder)\n\n";
    }
  }

  // Set the value of a table's field.
  void BuildFieldOfTable(const StructDef &struct_def, const FieldDef &field,
                         const size_t offset, std::string *code_ptr) const {
    auto &code = *code_ptr;
    const std::string field_var = namer_.Variable(field);
    const std::string field_method = namer_.Method(field);
    const std::string field_ty = GenFieldTy(field);

    const auto name = parser_.opts.python_no_type_prefix_suffix
                          ? "Add" + field_method
                          : namer_.Type(struct_def) + "Add" + field_method;

    // Generate method with struct name.
    code += "def " + name;
    if (parser_.opts.python_typing) {
      code += "(builder: flatbuffers.Builder, " + field_var + ": " + field_ty;
    } else {
      code += "(builder, " + field_var;
    }
    code += "):\n";
    code += Indent + "builder.Prepend";
    code += GenMethod(field) + "Slot(";
    code += NumToString(offset) + ", ";
    if (!IsScalar(field.value.type.base_type) && (!struct_def.fixed)) {
      code += "flatbuffers.number_types.UOffsetTFlags.py_type";
      code += "(" + field_var + ")";
    } else {
      code += field_var;
    }
    code += ", ";
    if (field.IsScalarOptional()) {
      code += "None";
    } else if (IsFloat(field.value.type.base_type)) {
      code += float_const_gen_.GenFloatConstant(field);
    } else {
      code += field.value.constant;
    }
    code += ")\n\n";

    if (!parser_.opts.one_file && !parser_.opts.python_no_type_prefix_suffix) {
      // Generate method without struct name.
      code += "def Add" + field_method;
      if (parser_.opts.python_typing) {
        code += "(builder: flatbuffers.Builder, " + field_var + ": " + field_ty;
      } else {
        code += "(builder, " + field_var;
      }
      code += "):\n";
      code += Indent + namer_.Type(struct_def) + "Add" + field_method;
      code += "(builder, ";
      code += field_var;
      code += ")\n\n";
    }
  }

  // Set the value of one of the members of a table's vector.
  void BuildVectorOfTable(const StructDef &struct_def, const FieldDef &field,
                          std::string *code_ptr) const {
    auto &code = *code_ptr;
    const std::string struct_type = namer_.Type(struct_def);
    const std::string field_method = namer_.Method(field);

    // Generate method with struct name.
    const auto name = parser_.opts.python_no_type_prefix_suffix
                          ? "Start" + field_method
                          : struct_type + "Start" + field_method;
    code += "def " + name;
    if (parser_.opts.python_typing) {
      code += "Vector(builder, numElems: int) -> int:\n";
    } else {
      code += "Vector(builder, numElems):\n";
    }

    code += Indent + "return builder.StartVector(";
    auto vector_type = field.value.type.VectorType();
    auto alignment = InlineAlignment(vector_type);
    auto elem_size = InlineSize(vector_type);
    code += NumToString(elem_size);
    code += ", numElems, " + NumToString(alignment);
    code += ")\n\n";

    if (!parser_.opts.one_file && !parser_.opts.python_no_type_prefix_suffix) {
      // Generate method without struct name.
      if (parser_.opts.python_typing) {
        code += "def Start" + field_method +
                "Vector(builder, numElems: int) -> int:\n";
      } else {
        code += "def Start" + field_method + "Vector(builder, numElems):\n";
      }
      code += Indent + "return " + struct_type + "Start";
      code += field_method + "Vector(builder, numElems)\n\n";
    }
  }

  // Set the value of one of the members of a table's vector and fills in the
  // elements from a bytearray. This is for simplifying the use of nested
  // flatbuffers.
  void BuildVectorOfTableFromBytes(const StructDef &struct_def,
                                   const FieldDef &field,
                                   std::string *code_ptr) const {
    auto nested = field.attributes.Lookup("nested_flatbuffer");
    if (!nested) { return; }  // There is no nested flatbuffer.

    auto &code = *code_ptr;
    const std::string field_method = namer_.Method(field);
    const std::string struct_type = namer_.Type(struct_def);

    // Generate method with struct and field name.
    code += "def " + struct_type + "Make" + field_method;
    code += "VectorFromBytes(builder, bytes):\n";
    code += Indent + "builder.StartVector(";
    auto vector_type = field.value.type.VectorType();
    auto alignment = InlineAlignment(vector_type);
    auto elem_size = InlineSize(vector_type);
    code += NumToString(elem_size);
    code += ", len(bytes), " + NumToString(alignment);
    code += ")\n";
    code += Indent + "builder.head = builder.head - len(bytes)\n";
    code += Indent + "builder.Bytes[builder.head : builder.head + len(bytes)]";
    code += " = bytes\n";
    code += Indent + "return builder.EndVector()\n";

    if (!parser_.opts.one_file) {
      // Generate method without struct and field name.
      code += "def Make" + field_method + "VectorFromBytes(builder, bytes):\n";
      code += Indent + "return " + struct_type + "Make" + field_method +
              "VectorFromBytes(builder, bytes)\n";
    }
  }

  // Get the offset of the end of a table.
  void GetEndOffsetOnTable(const StructDef &struct_def,
                           std::string *code_ptr) const {
    auto &code = *code_ptr;

    const auto name = parser_.opts.python_no_type_prefix_suffix
                          ? "End"
                          : namer_.Type(struct_def) + "End";
    // Generate method with struct name.
    if (parser_.opts.python_typing) {
      code += "def " + name + "(builder: flatbuffers.Builder) -> int:\n";
    } else {
      code += "def " + name + "(builder):\n";
    }
    code += Indent + "return builder.EndObject()\n\n";

    if (!parser_.opts.one_file && !parser_.opts.python_no_type_prefix_suffix) {
      // Generate method without struct name.
      if (parser_.opts.python_typing) {
        code += "def End(builder: flatbuffers.Builder) -> int:\n";
      } else {
        code += "def End(builder):\n";
      }
      code += Indent + "return " + namer_.Type(struct_def) + "End(builder)";
      code += "\n";
    }
  }

  // Generate the receiver for function signatures.
  void GenReceiver(const StructDef &struct_def, std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += Indent + "# " + namer_.Type(struct_def) + "\n";
    code += Indent + "def ";
  }

  // Generate a struct field, conditioned on its child type(s).
  void GenStructAccessor(const StructDef &struct_def, const FieldDef &field,
                         std::string *code_ptr, ImportMap &imports) const {
    GenComment(field.doc_comment, code_ptr, &def_comment, Indent.c_str());
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
            GetStructFieldOfTable(struct_def, field, code_ptr, imports);
          }
          break;
        case BASE_TYPE_STRING:
          GetStringField(struct_def, field, code_ptr, imports);
          break;
        case BASE_TYPE_VECTOR: {
          auto vectortype = field.value.type.VectorType();
          if (vectortype.base_type == BASE_TYPE_STRUCT) {
            GetMemberOfVectorOfStruct(struct_def, field, code_ptr, imports);
          } else {
            GetMemberOfVectorOfNonStruct(struct_def, field, code_ptr);
            GetVectorOfNonStructAsNumpy(struct_def, field, code_ptr);
            GetVectorAsNestedFlatbuffer(struct_def, field, code_ptr, imports);
          }
          break;
        }
        case BASE_TYPE_ARRAY: {
          auto vectortype = field.value.type.VectorType();
          if (vectortype.base_type == BASE_TYPE_STRUCT) {
            GetArrayOfStruct(struct_def, field, code_ptr, imports);
          } else {
            GetArrayOfNonStruct(struct_def, field, code_ptr);
            GetVectorOfNonStructAsNumpy(struct_def, field, code_ptr);
            GetVectorAsNestedFlatbuffer(struct_def, field, code_ptr, imports);
          }
          break;
        }
        case BASE_TYPE_UNION:
          GetUnionField(struct_def, field, code_ptr, imports);
          break;
        default: FLATBUFFERS_ASSERT(0);
      }
    }
    if (IsVector(field.value.type) || IsArray(field.value.type)) {
      GetVectorLen(struct_def, field, code_ptr);
      GetVectorIsNone(struct_def, field, code_ptr);
    }
  }

  // Generate struct sizeof.
  void GenStructSizeOf(const StructDef &struct_def,
                       std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += Indent + "@classmethod\n";
    if (parser_.opts.python_typing) {
      code += Indent + "def SizeOf(cls) -> int:\n";
    } else {
      code += Indent + "def SizeOf(cls):\n";
    }
    code +=
        Indent + Indent + "return " + NumToString(struct_def.bytesize) + "\n";
    code += "\n";
  }

  // Generate table constructors, conditioned on its members' types.
  void GenTableBuilders(const StructDef &struct_def,
                        std::string *code_ptr) const {
    GetStartOfTable(struct_def, code_ptr);

    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      auto offset = it - struct_def.fields.vec.begin();
      BuildFieldOfTable(struct_def, field, offset, code_ptr);
      if (IsVector(field.value.type)) {
        BuildVectorOfTable(struct_def, field, code_ptr);
        BuildVectorOfTableFromBytes(struct_def, field, code_ptr);
      }
    }

    GetEndOffsetOnTable(struct_def, code_ptr);
  }

  // Generate function to check for proper file identifier
  void GenHasFileIdentifier(const StructDef &struct_def,
                            std::string *code_ptr) const {
    auto &code = *code_ptr;
    std::string escapedID;
    // In the event any of file_identifier characters are special(NULL, \, etc),
    // problems occur. To prevent this, convert all chars to their hex-escaped
    // equivalent.
    for (auto it = parser_.file_identifier_.begin();
         it != parser_.file_identifier_.end(); ++it) {
      escapedID += "\\x" + IntToStringHex(*it, 2);
    }

    code += Indent + "@classmethod\n";
    code += Indent + "def " + namer_.Type(struct_def);
    code += "BufferHasIdentifier(cls, buf, offset, size_prefixed=False):";
    code += "\n";
    code += Indent + Indent;
    code += "return flatbuffers.util.BufferHasIdentifier(buf, offset, b\"";
    code += escapedID;
    code += "\", size_prefixed=size_prefixed)\n";
    code += "\n";
  }

  // Generates struct or table methods.
  void GenStruct(const StructDef &struct_def, std::string *code_ptr,
                 ImportMap &imports) const {
    if (struct_def.generated) return;

    GenComment(struct_def.doc_comment, code_ptr, &def_comment);
    BeginClass(struct_def, code_ptr);
    if (!struct_def.fixed) {
      // Generate a special accessor for the table that has been declared as
      // the root type.
      NewRootTypeFromBuffer(struct_def, code_ptr);
      if (parser_.file_identifier_.length()) {
        // Generate a special function to test file_identifier
        GenHasFileIdentifier(struct_def, code_ptr);
      }
    } else {
      // Generates the SizeOf method for all structs.
      GenStructSizeOf(struct_def, code_ptr);
    }
    // Generates the Init method that sets the field in a pre-existing
    // accessor object. This is to allow object reuse.
    InitializeExisting(struct_def, code_ptr);
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      GenStructAccessor(struct_def, field, code_ptr, imports);
    }

    if (struct_def.fixed) {
      // creates a struct constructor function
      GenStructBuilder(struct_def, code_ptr);
    } else {
      // Creates a set of functions that allow table construction.
      GenTableBuilders(struct_def, code_ptr);
    }
  }

  void GenReceiverForObjectAPI(const StructDef &struct_def,
                               std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += GenIndents(1) + "# " + namer_.ObjectType(struct_def);
    code += GenIndents(1) + "def ";
  }

  void BeginClassForObjectAPI(const StructDef &struct_def,
                              std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += "\n";
    code += "class " + namer_.ObjectType(struct_def) + "(object):";
    code += "\n";
  }

  // Gets the accoresponding python builtin type of a BaseType for scalars and
  // string.
  std::string GetBasePythonTypeForScalarAndString(
      const BaseType &base_type) const {
    if (IsBool(base_type)) {
      return "bool";
    } else if (IsFloat(base_type)) {
      return "float";
    } else if (IsInteger(base_type)) {
      return "int";
    } else if (base_type == BASE_TYPE_STRING) {
      return "str";
    } else {
      FLATBUFFERS_ASSERT(false && "base_type is not a scalar or string type.");
      return "";
    }
  }

  std::string GetDefaultValue(const FieldDef &field) const {
    BaseType base_type = field.value.type.base_type;
    if (field.IsScalarOptional()) {
      return "None";
    } else if (IsBool(base_type)) {
      return field.value.constant == "0" ? "False" : "True";
    } else if (IsFloat(base_type)) {
      return float_const_gen_.GenFloatConstant(field);
    } else if (IsInteger(base_type)) {
      return field.value.constant;
    } else {
      // For string, struct, and table.
      return "None";
    }
  }

  void GenUnionInit(const FieldDef &field, std::string *field_types_ptr,
                    std::set<std::string> *import_list,
                    std::set<std::string> *import_typing_list) const {
    // Gets all possible types in the union.
    import_typing_list->insert("Union");
    auto &field_types = *field_types_ptr;
    field_types = "Union[";

    std::string separator_string = ", ";
    auto enum_def = field.value.type.enum_def;
    for (auto it = enum_def->Vals().begin(); it != enum_def->Vals().end();
         ++it) {
      auto &ev = **it;
      // Union only supports string and table.
      std::string field_type;
      switch (ev.union_type.base_type) {
        case BASE_TYPE_STRUCT:
          field_type = namer_.ObjectType(*ev.union_type.struct_def);
          if (parser_.opts.include_dependence_headers) {
            auto package_reference = GenPackageReference(ev.union_type);
            field_type = package_reference + "." + field_type;
            import_list->insert("import " + package_reference);
          }
          break;
        case BASE_TYPE_STRING: field_type += "str"; break;
        case BASE_TYPE_NONE: field_type += "None"; break;
        default: break;
      }
      field_types += field_type + separator_string;
    }

    // Removes the last separator_string.
    field_types.erase(field_types.length() - separator_string.size());
    field_types += "]";

    // Gets the import lists for the union.
    if (parser_.opts.include_dependence_headers) {
      const auto package_reference = GenPackageReference(field.value.type);
      import_list->insert("import " + package_reference);
    }
  }

  void GenStructInit(const FieldDef &field, std::string *out_ptr,
                     std::set<std::string> *import_list,
                     std::set<std::string> *import_typing_list) const {
    import_typing_list->insert("Optional");
    auto &output = *out_ptr;
    const Type &type = field.value.type;
    const std::string object_type = namer_.ObjectType(*type.struct_def);
    if (parser_.opts.include_dependence_headers) {
      auto package_reference = GenPackageReference(type);
      output = package_reference + "." + object_type + "]";
      import_list->insert("import " + package_reference);
    } else {
      output = object_type + "]";
    }
    output = "Optional[" + output;
  }

  void GenVectorInit(const FieldDef &field, std::string *field_type_ptr,
                     std::set<std::string> *import_list,
                     std::set<std::string> *import_typing_list) const {
    import_typing_list->insert("List");
    auto &field_type = *field_type_ptr;
    const Type &vector_type = field.value.type.VectorType();
    const BaseType base_type = vector_type.base_type;
    if (base_type == BASE_TYPE_STRUCT) {
      const std::string object_type =
          namer_.ObjectType(*vector_type.struct_def);
      field_type = object_type + "]";
      if (parser_.opts.include_dependence_headers) {
        auto package_reference = GenPackageReference(vector_type);
        field_type = package_reference + "." + object_type + "]";
        import_list->insert("import " + package_reference);
      }
      field_type = "List[" + field_type;
    } else {
      field_type =
          "List[" + GetBasePythonTypeForScalarAndString(base_type) + "]";
    }
  }

  void GenInitialize(const StructDef &struct_def, std::string *code_ptr,
                     std::set<std::string> *import_list) const {
    std::string code;
    std::set<std::string> import_typing_list;
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      // Determines field type, default value, and typing imports.
      auto base_type = field.value.type.base_type;
      std::string field_type;
      switch (base_type) {
        case BASE_TYPE_UNION: {
          GenUnionInit(field, &field_type, import_list, &import_typing_list);
          break;
        }
        case BASE_TYPE_STRUCT: {
          GenStructInit(field, &field_type, import_list, &import_typing_list);
          break;
        }
        case BASE_TYPE_VECTOR:
        case BASE_TYPE_ARRAY: {
          GenVectorInit(field, &field_type, import_list, &import_typing_list);
          break;
        }
        default:
          // Scalar or sting fields.
          field_type = GetBasePythonTypeForScalarAndString(base_type);
          if (field.IsScalarOptional()) {
            field_type = "Optional[" + field_type + "]";
          }
          break;
      }

      const auto default_value = GetDefaultValue(field);
      // Wrties the init statement.
      const auto field_field = namer_.Field(field);
      code += GenIndents(2) + "self." + field_field + " = " + default_value +
              "  # type: " + field_type;
    }

    // Writes __init__ method.
    auto &code_base = *code_ptr;
    GenReceiverForObjectAPI(struct_def, code_ptr);
    code_base += "__init__(self):";
    if (code.empty()) {
      code_base += GenIndents(2) + "pass";
    } else {
      code_base += code;
    }
    code_base += "\n";

    // Merges the typing imports into import_list.
    if (!import_typing_list.empty()) {
      // Adds the try statement.
      std::string typing_imports = "try:";
      typing_imports += GenIndents(1) + "from typing import ";
      std::string separator_string = ", ";
      for (auto it = import_typing_list.begin(); it != import_typing_list.end();
           ++it) {
        const std::string &im = *it;
        typing_imports += im + separator_string;
      }
      // Removes the last separator_string.
      typing_imports.erase(typing_imports.length() - separator_string.size());

      // Adds the except statement.
      typing_imports += "\n";
      typing_imports += "except:";
      typing_imports += GenIndents(1) + "pass";
      import_list->insert(typing_imports);
    }

    // Removes the import of the struct itself, if applied.
    auto struct_import = "import " + namer_.NamespacedType(struct_def);
    import_list->erase(struct_import);
  }

  void InitializeFromBuf(const StructDef &struct_def,
                         std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_var = namer_.Variable(struct_def);
    const auto struct_type = namer_.Type(struct_def);

    code += GenIndents(1) + "@classmethod";
    code += GenIndents(1) + "def InitFromBuf(cls, buf, pos):";
    code += GenIndents(2) + struct_var + " = " + struct_type + "()";
    code += GenIndents(2) + struct_var + ".Init(buf, pos)";
    code += GenIndents(2) + "return cls.InitFromObj(" + struct_var + ")";
    code += "\n";
  }

  void InitializeFromPackedBuf(const StructDef &struct_def,
                               std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_var = namer_.Variable(struct_def);
    const auto struct_type = namer_.Type(struct_def);

    code += GenIndents(1) + "@classmethod";
    code += GenIndents(1) + "def InitFromPackedBuf(cls, buf, pos=0):";
    code += GenIndents(2) +
            "n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, pos)";
    code += GenIndents(2) + "return cls.InitFromBuf(buf, pos+n)";
    code += "\n";
  }

  void InitializeFromObjForObject(const StructDef &struct_def,
                                  std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_var = namer_.Variable(struct_def);
    const auto struct_object = namer_.ObjectType(struct_def);

    code += GenIndents(1) + "@classmethod";
    code += GenIndents(1) + "def InitFromObj(cls, " + struct_var + "):";
    code += GenIndents(2) + "x = " + struct_object + "()";
    code += GenIndents(2) + "x._UnPack(" + struct_var + ")";
    code += GenIndents(2) + "return x";
    code += "\n";
  }

  void GenCompareOperator(const StructDef &struct_def,
                          std::string *code_ptr) const {
    auto &code = *code_ptr;
    code += GenIndents(1) + "def __eq__(self, other):";
    code += GenIndents(2) + "return type(self) == type(other)";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      // Wrties the comparison statement for this field.
      const auto field_field = namer_.Field(field);
      code += " and \\" + GenIndents(3) + "self." + field_field +
              " == " + "other." + field_field;
    }
    code += "\n";
  }

  void GenUnPackForStruct(const StructDef &struct_def, const FieldDef &field,
                          std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_var = namer_.Variable(struct_def);
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    auto field_type = TypeName(field);

    if (parser_.opts.include_dependence_headers) {
      auto package_reference = GenPackageReference(field.value.type);
      field_type = package_reference + "." + TypeName(field);
    }

    code += GenIndents(2) + "if " + struct_var + "." + field_method + "(";
    // if field is a struct, we need to create an instance for it first.
    if (struct_def.fixed && field.value.type.base_type == BASE_TYPE_STRUCT) {
      code += field_type + "()";
    }
    code += ") is not None:";
    code += GenIndents(3) + "self." + field_field + " = " +
            namer_.ObjectType(field_type) + +".InitFromObj(" + struct_var +
            "." + field_method + "(";
    // A struct's accessor requires a struct buf instance.
    if (struct_def.fixed && field.value.type.base_type == BASE_TYPE_STRUCT) {
      code += field_type + "()";
    }
    code += "))";
  }

  void GenUnPackForUnion(const StructDef &struct_def, const FieldDef &field,
                         std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);
    const EnumDef &enum_def = *field.value.type.enum_def;
    auto union_type = namer_.Type(enum_def);

    if (parser_.opts.include_dependence_headers) {
      union_type = namer_.NamespacedType(enum_def) + "." + union_type;
    }
    code += GenIndents(2) + "self." + field_field + " = " + union_type +
            "Creator(" + "self." + field_field + "Type, " + struct_var + "." +
            field_method + "())";
  }

  void GenUnPackForStructVector(const StructDef &struct_def,
                                const FieldDef &field,
                                std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);

    code += GenIndents(2) + "if not " + struct_var + "." + field_method +
            "IsNone():";
    code += GenIndents(3) + "self." + field_field + " = []";
    code += GenIndents(3) + "for i in range(" + struct_var + "." +
            field_method + "Length()):";

    auto field_type = TypeName(field);
    auto one_instance = field_type + "_";
    one_instance[0] = CharToLower(one_instance[0]);
    if (parser_.opts.include_dependence_headers) {
      auto package_reference = GenPackageReference(field.value.type);
      field_type = package_reference + "." + TypeName(field);
    }
    code += GenIndents(4) + "if " + struct_var + "." + field_method +
            "(i) is None:";
    code += GenIndents(5) + "self." + field_field + ".append(None)";
    code += GenIndents(4) + "else:";
    code += GenIndents(5) + one_instance + " = " +
            namer_.ObjectType(field_type) + ".InitFromObj(" + struct_var + "." +
            field_method + "(i))";
    code +=
        GenIndents(5) + "self." + field_field + ".append(" + one_instance + ")";
  }

  void GenUnpackForTableVector(const StructDef &struct_def,
                               const FieldDef &field,
                               std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);

    code += GenIndents(2) + "if not " + struct_var + "." + field_method +
            "IsNone():";
    code += GenIndents(3) + "self." + field_field + " = []";
    code += GenIndents(3) + "for i in range(" + struct_var + "." +
            field_method + "Length()):";

    auto field_type = TypeName(field);
    auto one_instance = field_type + "_";
    one_instance[0] = CharToLower(one_instance[0]);
    if (parser_.opts.include_dependence_headers) {
      auto package_reference = GenPackageReference(field.value.type);
      field_type = package_reference + "." + TypeName(field);
    }
    code += GenIndents(4) + "if " + struct_var + "." + field_method +
            "(i) is None:";
    code += GenIndents(5) + "self." + field_field + ".append(None)";
    code += GenIndents(4) + "else:";
    code += GenIndents(5) + one_instance + " = " +
            namer_.ObjectType(field_type) + ".InitFromObj(" + struct_var + "." +
            field_method + "(i))";
    code +=
        GenIndents(5) + "self." + field_field + ".append(" + one_instance + ")";
  }

  void GenUnpackforScalarVectorHelper(const StructDef &struct_def,
                                      const FieldDef &field,
                                      std::string *code_ptr,
                                      int indents) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);

    code += GenIndents(indents) + "self." + field_field + " = []";
    code += GenIndents(indents) + "for i in range(" + struct_var + "." +
            field_method + "Length()):";
    code += GenIndents(indents + 1) + "self." + field_field + ".append(" +
            struct_var + "." + field_method + "(i))";
  }

  void GenUnPackForScalarVector(const StructDef &struct_def,
                                const FieldDef &field,
                                std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);

    code += GenIndents(2) + "if not " + struct_var + "." + field_method +
            "IsNone():";

    // String does not have the AsNumpy method.
    if (!(IsScalar(field.value.type.VectorType().base_type))) {
      GenUnpackforScalarVectorHelper(struct_def, field, code_ptr, 3);
      return;
    }

    code += GenIndents(3) + "if np is None:";
    GenUnpackforScalarVectorHelper(struct_def, field, code_ptr, 4);

    // If numpy exists, use the AsNumpy method to optimize the unpack speed.
    code += GenIndents(3) + "else:";
    code += GenIndents(4) + "self." + field_field + " = " + struct_var + "." +
            field_method + "AsNumpy()";
  }

  void GenUnPackForScalar(const StructDef &struct_def, const FieldDef &field,
                          std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_var = namer_.Variable(struct_def);

    code += GenIndents(2) + "self." + field_field + " = " + struct_var + "." +
            field_method + "()";
  }

  // Generates the UnPack method for the object class.
  void GenUnPack(const StructDef &struct_def, std::string *code_ptr) const {
    std::string code;
    // Items that needs to be imported. No duplicate modules will be imported.
    std::set<std::string> import_list;

    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      auto field_type = TypeName(field);
      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          GenUnPackForStruct(struct_def, field, &code);
          break;
        }
        case BASE_TYPE_UNION: {
          GenUnPackForUnion(struct_def, field, &code);
          break;
        }
        case BASE_TYPE_ARRAY:
        case BASE_TYPE_VECTOR: {
          auto vectortype = field.value.type.VectorType();
          if (vectortype.base_type == BASE_TYPE_STRUCT) {
            GenUnPackForStructVector(struct_def, field, &code);
          } else {
            GenUnPackForScalarVector(struct_def, field, &code);
          }
          break;
        }
        default: GenUnPackForScalar(struct_def, field, &code);
      }
    }

    // Writes import statements and code into the generated file.
    auto &code_base = *code_ptr;
    const auto struct_var = namer_.Variable(struct_def);

    GenReceiverForObjectAPI(struct_def, code_ptr);
    code_base += "_UnPack(self, " + struct_var + "):";
    code_base += GenIndents(2) + "if " + struct_var + " is None:";
    code_base += GenIndents(3) + "return";

    // Write the import statements.
    for (std::set<std::string>::iterator it = import_list.begin();
         it != import_list.end(); ++it) {
      code_base += GenIndents(2) + *it;
    }

    // Write the code.
    code_base += code;
    code_base += "\n";
  }

  void GenPackForStruct(const StructDef &struct_def,
                        std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto struct_fn = namer_.Function(struct_def);

    GenReceiverForObjectAPI(struct_def, code_ptr);
    code += "Pack(self, builder):";
    code += GenIndents(2) + "return Create" + struct_fn + "(builder";

    StructBuilderArgs(struct_def,
                      /* nameprefix = */ "self.",
                      /* namesuffix = */ "",
                      /* has_field_name = */ true,
                      /* fieldname_suffix = */ ".", code_ptr);
    code += ")\n";
  }

  void GenPackForStructVectorField(const StructDef &struct_def,
                                   const FieldDef &field,
                                   std::string *code_prefix_ptr,
                                   std::string *code_ptr) const {
    auto &code_prefix = *code_prefix_ptr;
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto struct_type = namer_.Type(struct_def);
    const auto field_method = namer_.Method(field);

    // Creates the field.
    code_prefix += GenIndents(2) + "if self." + field_field + " is not None:";
    if (field.value.type.struct_def->fixed) {
      code_prefix += GenIndents(3) + struct_type + "Start" + field_method +
                     "Vector(builder, len(self." + field_field + "))";
      code_prefix += GenIndents(3) + "for i in reversed(range(len(self." +
                     field_field + "))):";
      code_prefix +=
          GenIndents(4) + "self." + field_field + "[i].Pack(builder)";
      code_prefix += GenIndents(3) + field_field + " = builder.EndVector()";
    } else {
      // If the vector is a struct vector, we need to first build accessor for
      // each struct element.
      code_prefix += GenIndents(3) + field_field + "list = []";
      code_prefix += GenIndents(3);
      code_prefix += "for i in range(len(self." + field_field + ")):";
      code_prefix += GenIndents(4) + field_field + "list.append(self." +
                     field_field + "[i].Pack(builder))";

      code_prefix += GenIndents(3) + struct_type + "Start" + field_method +
                     "Vector(builder, len(self." + field_field + "))";
      code_prefix += GenIndents(3) + "for i in reversed(range(len(self." +
                     field_field + "))):";
      code_prefix += GenIndents(4) + "builder.PrependUOffsetTRelative" + "(" +
                     field_field + "list[i])";
      code_prefix += GenIndents(3) + field_field + " = builder.EndVector()";
    }

    // Adds the field into the struct.
    code += GenIndents(2) + "if self." + field_field + " is not None:";
    code += GenIndents(3) + struct_type + "Add" + field_method + "(builder, " +
            field_field + ")";
  }

  void GenPackForScalarVectorFieldHelper(const StructDef &struct_def,
                                         const FieldDef &field,
                                         std::string *code_ptr,
                                         int indents) const {
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_type = namer_.Type(struct_def);
    const auto vectortype = field.value.type.VectorType();

    code += GenIndents(indents) + struct_type + "Start" + field_method +
            "Vector(builder, len(self." + field_field + "))";
    code += GenIndents(indents) + "for i in reversed(range(len(self." +
            field_field + "))):";
    code += GenIndents(indents + 1) + "builder.Prepend";

    std::string type_name;
    switch (vectortype.base_type) {
      case BASE_TYPE_BOOL: type_name = "Bool"; break;
      case BASE_TYPE_CHAR: type_name = "Byte"; break;
      case BASE_TYPE_UCHAR: type_name = "Uint8"; break;
      case BASE_TYPE_SHORT: type_name = "Int16"; break;
      case BASE_TYPE_USHORT: type_name = "Uint16"; break;
      case BASE_TYPE_INT: type_name = "Int32"; break;
      case BASE_TYPE_UINT: type_name = "Uint32"; break;
      case BASE_TYPE_LONG: type_name = "Int64"; break;
      case BASE_TYPE_ULONG: type_name = "Uint64"; break;
      case BASE_TYPE_FLOAT: type_name = "Float32"; break;
      case BASE_TYPE_DOUBLE: type_name = "Float64"; break;
      case BASE_TYPE_STRING: type_name = "UOffsetTRelative"; break;
      default: type_name = "VOffsetT"; break;
    }
    code += type_name;
  }

  void GenPackForScalarVectorField(const StructDef &struct_def,
                                   const FieldDef &field,
                                   std::string *code_prefix_ptr,
                                   std::string *code_ptr) const {
    auto &code = *code_ptr;
    auto &code_prefix = *code_prefix_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_type = namer_.Type(struct_def);

    // Adds the field into the struct.
    code += GenIndents(2) + "if self." + field_field + " is not None:";
    code += GenIndents(3) + struct_type + "Add" + field_method + "(builder, " +
            field_field + ")";

    // Creates the field.
    code_prefix += GenIndents(2) + "if self." + field_field + " is not None:";
    // If the vector is a string vector, we need to first build accessor for
    // each string element. And this generated code, needs to be
    // placed ahead of code_prefix.
    auto vectortype = field.value.type.VectorType();
    if (IsString(vectortype)) {
      code_prefix += GenIndents(3) + field_field + "list = []";
      code_prefix +=
          GenIndents(3) + "for i in range(len(self." + field_field + ")):";
      code_prefix += GenIndents(4) + field_field +
                     "list.append(builder.CreateString(self." + field_field +
                     "[i]))";
      GenPackForScalarVectorFieldHelper(struct_def, field, code_prefix_ptr, 3);
      code_prefix += "(" + field_field + "list[i])";
      code_prefix += GenIndents(3) + field_field + " = builder.EndVector()";
      return;
    }

    code_prefix += GenIndents(3) + "if np is not None and type(self." +
                   field_field + ") is np.ndarray:";
    code_prefix += GenIndents(4) + field_field +
                   " = builder.CreateNumpyVector(self." + field_field + ")";
    code_prefix += GenIndents(3) + "else:";
    GenPackForScalarVectorFieldHelper(struct_def, field, code_prefix_ptr, 4);
    code_prefix += "(self." + field_field + "[i])";
    code_prefix += GenIndents(4) + field_field + " = builder.EndVector()";
  }

  void GenPackForStructField(const StructDef &struct_def, const FieldDef &field,
                             std::string *code_prefix_ptr,
                             std::string *code_ptr) const {
    auto &code_prefix = *code_prefix_ptr;
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_type = namer_.Type(struct_def);

    if (field.value.type.struct_def->fixed) {
      // Pure struct fields need to be created along with their parent
      // structs.
      code += GenIndents(2) + "if self." + field_field + " is not None:";
      code += GenIndents(3) + field_field + " = self." + field_field +
              ".Pack(builder)";
    } else {
      // Tables need to be created before their parent structs are created.
      code_prefix += GenIndents(2) + "if self." + field_field + " is not None:";
      code_prefix += GenIndents(3) + field_field + " = self." + field_field +
                     ".Pack(builder)";
      code += GenIndents(2) + "if self." + field_field + " is not None:";
    }

    code += GenIndents(3) + struct_type + "Add" + field_method + "(builder, " +
            field_field + ")";
  }

  void GenPackForUnionField(const StructDef &struct_def, const FieldDef &field,
                            std::string *code_prefix_ptr,
                            std::string *code_ptr) const {
    auto &code_prefix = *code_prefix_ptr;
    auto &code = *code_ptr;
    const auto field_field = namer_.Field(field);
    const auto field_method = namer_.Method(field);
    const auto struct_type = namer_.Type(struct_def);

    // TODO(luwa): TypeT should be moved under the None check as well.
    code_prefix += GenIndents(2) + "if self." + field_field + " is not None:";
    code_prefix += GenIndents(3) + field_field + " = self." + field_field +
                   ".Pack(builder)";
    code += GenIndents(2) + "if self." + field_field + " is not None:";
    code += GenIndents(3) + struct_type + "Add" + field_method + "(builder, " +
            field_field + ")";
  }

  void GenPackForTable(const StructDef &struct_def,
                       std::string *code_ptr) const {
    auto &code_base = *code_ptr;
    std::string code, code_prefix;
    const auto struct_var = namer_.Variable(struct_def);
    const auto struct_type = namer_.Type(struct_def);

    GenReceiverForObjectAPI(struct_def, code_ptr);
    code_base += "Pack(self, builder):";
    code += GenIndents(2) + struct_type + "Start(builder)";
    for (auto it = struct_def.fields.vec.begin();
         it != struct_def.fields.vec.end(); ++it) {
      auto &field = **it;
      if (field.deprecated) continue;

      const auto field_method = namer_.Method(field);
      const auto field_field = namer_.Field(field);

      switch (field.value.type.base_type) {
        case BASE_TYPE_STRUCT: {
          GenPackForStructField(struct_def, field, &code_prefix, &code);
          break;
        }
        case BASE_TYPE_UNION: {
          GenPackForUnionField(struct_def, field, &code_prefix, &code);
          break;
        }
        case BASE_TYPE_ARRAY:
        case BASE_TYPE_VECTOR: {
          auto vectortype = field.value.type.VectorType();
          if (vectortype.base_type == BASE_TYPE_STRUCT) {
            GenPackForStructVectorField(struct_def, field, &code_prefix, &code);
          } else {
            GenPackForScalarVectorField(struct_def, field, &code_prefix, &code);
          }
          break;
        }
        case BASE_TYPE_STRING: {
          code_prefix +=
              GenIndents(2) + "if self." + field_field + " is not None:";
          code_prefix += GenIndents(3) + field_field +
                         " = builder.CreateString(self." + field_field + ")";
          code += GenIndents(2) + "if self." + field_field + " is not None:";
          code += GenIndents(3) + struct_type + "Add" + field_method +
                  "(builder, " + field_field + ")";
          break;
        }
        default:
          // Generates code for scalar values. If the value equals to the
          // default value, builder will automatically ignore it. So we don't
          // need to check the value ahead.
          code += GenIndents(2) + struct_type + "Add" + field_method +
                  "(builder, self." + field_field + ")";
          break;
      }
    }

    code += GenIndents(2) + struct_var + " = " + struct_type + "End(builder)";
    code += GenIndents(2) + "return " + struct_var;

    code_base += code_prefix + code;
    code_base += "\n";
  }

  void GenStructForObjectAPI(const StructDef &struct_def,
                             std::string *code_ptr) const {
    if (struct_def.generated) return;

    std::set<std::string> import_list;
    std::string code;

    // Creates an object class for a struct or a table
    BeginClassForObjectAPI(struct_def, &code);

    GenInitialize(struct_def, &code, &import_list);

    InitializeFromBuf(struct_def, &code);

    InitializeFromPackedBuf(struct_def, &code);

    InitializeFromObjForObject(struct_def, &code);

    if (parser_.opts.gen_compare) { GenCompareOperator(struct_def, &code); }

    GenUnPack(struct_def, &code);

    if (struct_def.fixed) {
      GenPackForStruct(struct_def, &code);
    } else {
      GenPackForTable(struct_def, &code);
    }

    // Adds the imports at top.
    auto &code_base = *code_ptr;
    code_base += "\n";
    for (auto it = import_list.begin(); it != import_list.end(); it++) {
      auto im = *it;
      code_base += im + "\n";
    }
    code_base += code;
  }

  void GenUnionCreatorForStruct(const EnumDef &enum_def, const EnumVal &ev,
                                std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto union_type = namer_.Type(enum_def);
    const auto variant = namer_.Variant(ev);
    auto field_type = namer_.ObjectType(*ev.union_type.struct_def);

    code +=
        GenIndents(1) + "if unionType == " + union_type + "()." + variant + ":";
    if (parser_.opts.include_dependence_headers) {
      auto package_reference = GenPackageReference(ev.union_type);
      code += GenIndents(2) + "import " + package_reference;
      field_type = package_reference + "." + field_type;
    }
    code += GenIndents(2) + "return " + field_type +
            ".InitFromBuf(table.Bytes, table.Pos)";
  }

  void GenUnionCreatorForString(const EnumDef &enum_def, const EnumVal &ev,
                                std::string *code_ptr) const {
    auto &code = *code_ptr;
    const auto union_type = namer_.Type(enum_def);
    const auto variant = namer_.Variant(ev);

    code +=
        GenIndents(1) + "if unionType == " + union_type + "()." + variant + ":";
    code += GenIndents(2) + "tab = Table(table.Bytes, table.Pos)";
    code += GenIndents(2) + "union = tab.String(table.Pos)";
    code += GenIndents(2) + "return union";
  }

  // Creates an union object based on union type.
  void GenUnionCreator(const EnumDef &enum_def, std::string *code_ptr) const {
    if (enum_def.generated) return;

    auto &code = *code_ptr;
    const auto enum_fn = namer_.Function(enum_def);

    code += "\n";
    code += "def " + enum_fn + "Creator(unionType, table):";
    code += GenIndents(1) + "from flatbuffers.table import Table";
    code += GenIndents(1) + "if not isinstance(table, Table):";
    code += GenIndents(2) + "return None";

    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      // Union only supports string and table.
      switch (ev.union_type.base_type) {
        case BASE_TYPE_STRUCT:
          GenUnionCreatorForStruct(enum_def, ev, &code);
          break;
        case BASE_TYPE_STRING:
          GenUnionCreatorForString(enum_def, ev, &code);
          break;
        default: break;
      }
    }
    code += GenIndents(1) + "return None";
    code += "\n";
  }

  // Generate enum declarations.
  void GenEnum(const EnumDef &enum_def, std::string *code_ptr) const {
    if (enum_def.generated) return;

    GenComment(enum_def.doc_comment, code_ptr, &def_comment);
    BeginEnum(enum_def, code_ptr);
    for (auto it = enum_def.Vals().begin(); it != enum_def.Vals().end(); ++it) {
      auto &ev = **it;
      GenComment(ev.doc_comment, code_ptr, &def_comment, Indent.c_str());
      EnumMember(enum_def, ev, code_ptr);
    }
  }

  // Returns the function name that is able to read a value of the given type.
  std::string GenGetter(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "self._tab.String(";
      case BASE_TYPE_UNION: return "self._tab.Union(";
      case BASE_TYPE_VECTOR: return GenGetter(type.VectorType());
      default:
        return "self._tab.Get(flatbuffers.number_types." +
               namer_.Method(GenTypeGet(type)) + "Flags, ";
    }
  }

  std::string GenFieldTy(const FieldDef &field) const {
    if (IsScalar(field.value.type.base_type) || IsArray(field.value.type)) {
      const std::string ty = GenTypeBasic(field.value.type);
      if (ty.find("int") != std::string::npos) { return "int"; }

      if (ty.find("float") != std::string::npos) { return "float"; }

      if (ty == "bool") { return "bool"; }

      return "Any";
    } else {
      if (IsStruct(field.value.type)) {
        return "Any";
      } else {
        return "int";
      }
    }
  }

  // Returns the method name for use with add/put calls.
  std::string GenMethod(const FieldDef &field) const {
    return (IsScalar(field.value.type.base_type) || IsArray(field.value.type))
               ? namer_.Method(GenTypeBasic(field.value.type))
               : (IsStruct(field.value.type) ? "Struct" : "UOffsetTRelative");
  }

  std::string GenTypeBasic(const Type &type) const {
    // clang-format off
    static const char *ctypename[] = {
      #define FLATBUFFERS_TD(ENUM, IDLTYPE, \
              CTYPE, JTYPE, GTYPE, NTYPE, PTYPE, ...) \
        #PTYPE,
        FLATBUFFERS_GEN_TYPES(FLATBUFFERS_TD)
      #undef FLATBUFFERS_TD
    };
    // clang-format on
    return ctypename[IsArray(type) ? type.VectorType().base_type
                                   : type.base_type];
  }

  std::string GenTypePointer(const Type &type) const {
    switch (type.base_type) {
      case BASE_TYPE_STRING: return "string";
      case BASE_TYPE_VECTOR:
        // fall through
      case BASE_TYPE_ARRAY: return GenTypeGet(type.VectorType());
      case BASE_TYPE_STRUCT: return type.struct_def->name;
      case BASE_TYPE_UNION:
        // fall through
      default: return "*flatbuffers.Table";
    }
  }

  std::string GenTypeGet(const Type &type) const {
    return IsScalar(type.base_type) ? GenTypeBasic(type) : GenTypePointer(type);
  }

  std::string TypeName(const FieldDef &field) const {
    return GenTypeGet(field.value.type);
  }

  std::string ReturnType(const StructDef &struct_def,
                         const FieldDef &field) const {
    // If we have a class member that returns an instance of the same class,
    // for example:
    // class Field(object):
    //   def Children(self, j: int) -> Optional[Field]:
    //     pass
    //
    // we need to quote the return type:
    // class Field(object):
    //   def Children(self, j: int) -> Optional['Field']:
    //     pass
    //
    // because Python is unable to resolve the name during parse and will return
    // an error.
    // (see PEP 484 under forward references:
    // https://peps.python.org/pep-0484/#forward-references)
    const std::string self_type = struct_def.name;
    std::string field_type = TypeName(field);

    if (self_type == field_type) { field_type = "'" + field_type + "'"; }

    return field_type;
  }

  // Create a struct with a builder and the struct's arguments.
  void GenStructBuilder(const StructDef &struct_def,
                        std::string *code_ptr) const {
    BeginBuilderArgs(struct_def, code_ptr);
    StructBuilderArgs(struct_def,
                      /* nameprefix = */ "",
                      /* namesuffix = */ "",
                      /* has_field_name = */ true,
                      /* fieldname_suffix = */ "_", code_ptr);
    EndBuilderArgs(code_ptr);

    StructBuilderBody(struct_def, "", code_ptr);
    EndBuilderBody(code_ptr);
  }

  bool generate() {
    std::string one_file_code;
    ImportMap one_file_imports;
    if (!generateEnums(&one_file_code)) return false;
    if (!generateStructs(&one_file_code, one_file_imports)) return false;

    if (parser_.opts.one_file) {
      const std::string mod = file_name_ + "_generated";

      // Legacy file format uses keep casing.
      return SaveType(mod + ".py", *parser_.current_namespace_, one_file_code,
                      one_file_imports, mod, true);
    }

    return true;
  }

 private:
  bool generateEnums(std::string *one_file_code) const {
    for (auto it = parser_.enums_.vec.begin(); it != parser_.enums_.vec.end();
         ++it) {
      auto &enum_def = **it;
      std::string enumcode;
      GenEnum(enum_def, &enumcode);
      if (parser_.opts.generate_object_based_api & enum_def.is_union) {
        GenUnionCreator(enum_def, &enumcode);
      }

      if (parser_.opts.one_file && !enumcode.empty()) {
        *one_file_code += enumcode + "\n\n";
      } else {
        ImportMap imports;
        const std::string mod =
            namer_.File(enum_def, SkipFile::SuffixAndExtension);

        if (!SaveType(namer_.File(enum_def, SkipFile::Suffix),
                      *enum_def.defined_namespace, enumcode, imports, mod,
                      false))
          return false;
      }
    }
    return true;
  }

  bool generateStructs(std::string *one_file_code,
                       ImportMap &one_file_imports) const {
    for (auto it = parser_.structs_.vec.begin();
         it != parser_.structs_.vec.end(); ++it) {
      auto &struct_def = **it;
      std::string declcode;
      ImportMap imports;
      GenStruct(struct_def, &declcode, imports);
      if (parser_.opts.generate_object_based_api) {
        GenStructForObjectAPI(struct_def, &declcode);
      }

      if (parser_.opts.one_file) {
        if (!declcode.empty()) { *one_file_code += declcode + "\n\n"; }

        for (auto import_str : imports) { one_file_imports.insert(import_str); }
      } else {
        const std::string mod =
            namer_.File(struct_def, SkipFile::SuffixAndExtension);
        if (!SaveType(namer_.File(struct_def, SkipFile::Suffix),
                      *struct_def.defined_namespace, declcode, imports, mod,
                      true))
          return false;
      }
    }
    return true;
  }

  // Begin by declaring namespace and imports.
  void BeginFile(const std::string &name_space_name, const bool needs_imports,
                 std::string *code_ptr, const std::string &mod,
                 const ImportMap &imports) const {
    auto &code = *code_ptr;
    code = code + "# " + FlatBuffersGeneratedWarning() + "\n\n";
    code += "# namespace: " + name_space_name + "\n\n";

    if (needs_imports) {
      const std::string local_import = "." + mod;

      code += "import flatbuffers\n";
      code += "from flatbuffers.compat import import_numpy\n";
      if (parser_.opts.python_typing) {
        code += "from typing import Any\n";

        for (auto import_entry : imports) {
          // If we have a file called, say, "MyType.py" and in it we have a
          // class "MyType", we can generate imports -- usually when we
          // have a type that contains arrays of itself -- of the type
          // "from .MyType import MyType", which Python can't resolve. So
          // if we are trying to import ourself, we skip.
          if (import_entry.first != local_import) {
            code += "from " + import_entry.first + " import " +
                    import_entry.second + "\n";
          }
        }
      }
      code += "np = import_numpy()\n\n";
    }
  }

  // Save out the generated code for a Python Table type.
  bool SaveType(const std::string &defname, const Namespace &ns,
                const std::string &classcode, const ImportMap &imports,
                const std::string &mod, bool needs_imports) const {
    if (!classcode.length()) return true;

    std::string code = "";
    BeginFile(LastNamespacePart(ns), needs_imports, &code, mod, imports);
    code += classcode;

    const std::string directories =
        parser_.opts.one_file ? path_ : namer_.Directories(ns.components);
    EnsureDirExists(directories);

    for (size_t i = path_.size() + 1; i != std::string::npos;
         i = directories.find(kPathSeparator, i + 1)) {
      const std::string init_py =
          directories.substr(0, i) + kPathSeparator + "__init__.py";
      SaveFile(init_py.c_str(), "", false);
    }

    const std::string filename = directories + defname;
    return SaveFile(filename.c_str(), code, false);
  }

 private:
  const SimpleFloatConstantGenerator float_const_gen_;
  const IdlNamer namer_;
};

}  // namespace python

static bool GeneratePython(const Parser &parser, const std::string &path,
                           const std::string &file_name) {
  python::PythonGenerator generator(parser, path, file_name);
  return generator.generate();
}

namespace {

class PythonCodeGenerator : public CodeGenerator {
 public:
  Status GenerateCode(const Parser &parser, const std::string &path,
                      const std::string &filename) override {
    if (!GeneratePython(parser, path, filename)) { return Status::ERROR; }
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
    if (!GeneratePythonGRPC(parser, path, filename)) { return Status::ERROR; }
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

  IDLOptions::Language Language() const override { return IDLOptions::kPython; }

  std::string LanguageName() const override { return "Python"; }
};
}  // namespace

std::unique_ptr<CodeGenerator> NewPythonCodeGenerator() {
  return std::unique_ptr<PythonCodeGenerator>(new PythonCodeGenerator());
}

}  // namespace flatbuffers
