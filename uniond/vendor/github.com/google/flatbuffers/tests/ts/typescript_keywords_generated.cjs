"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// typescript_keywords.ts
var typescript_keywords_exports = {};
__export(typescript_keywords_exports, {
  foobar: () => foobar_exports,
  reflection: () => reflection_exports,
  typescript: () => typescript_exports
});
module.exports = __toCommonJS(typescript_keywords_exports);

// foobar.js
var foobar_exports = {};
__export(foobar_exports, {
  Abc: () => Abc
});

// foobar/abc.js
var Abc;
(function(Abc2) {
  Abc2[Abc2["a"] = 0] = "a";
})(Abc || (Abc = {}));

// reflection.js
var reflection_exports = {};
__export(reflection_exports, {
  AdvancedFeatures: () => AdvancedFeatures,
  BaseType: () => BaseType,
  Enum: () => Enum,
  EnumT: () => EnumT,
  EnumVal: () => EnumVal,
  EnumValT: () => EnumValT,
  Field: () => Field,
  FieldT: () => FieldT,
  KeyValue: () => KeyValue,
  KeyValueT: () => KeyValueT,
  Object_: () => Object_,
  RPCCall: () => RPCCall,
  RPCCallT: () => RPCCallT,
  Schema: () => Schema,
  SchemaFile: () => SchemaFile,
  SchemaFileT: () => SchemaFileT,
  SchemaT: () => SchemaT,
  Service: () => Service,
  ServiceT: () => ServiceT,
  Type: () => Type,
  TypeT: () => TypeT
});

// reflection/advanced-features.js
var AdvancedFeatures;
(function(AdvancedFeatures2) {
  AdvancedFeatures2["AdvancedArrayFeatures"] = "1";
  AdvancedFeatures2["AdvancedUnionFeatures"] = "2";
  AdvancedFeatures2["OptionalScalars"] = "4";
  AdvancedFeatures2["DefaultVectorsAndStrings"] = "8";
})(AdvancedFeatures || (AdvancedFeatures = {}));

// reflection/base-type.js
var BaseType;
(function(BaseType2) {
  BaseType2[BaseType2["None"] = 0] = "None";
  BaseType2[BaseType2["UType"] = 1] = "UType";
  BaseType2[BaseType2["Bool"] = 2] = "Bool";
  BaseType2[BaseType2["Byte"] = 3] = "Byte";
  BaseType2[BaseType2["UByte"] = 4] = "UByte";
  BaseType2[BaseType2["Short"] = 5] = "Short";
  BaseType2[BaseType2["UShort"] = 6] = "UShort";
  BaseType2[BaseType2["Int"] = 7] = "Int";
  BaseType2[BaseType2["UInt"] = 8] = "UInt";
  BaseType2[BaseType2["Long"] = 9] = "Long";
  BaseType2[BaseType2["ULong"] = 10] = "ULong";
  BaseType2[BaseType2["Float"] = 11] = "Float";
  BaseType2[BaseType2["Double"] = 12] = "Double";
  BaseType2[BaseType2["String"] = 13] = "String";
  BaseType2[BaseType2["Vector"] = 14] = "Vector";
  BaseType2[BaseType2["Obj"] = 15] = "Obj";
  BaseType2[BaseType2["Union"] = 16] = "Union";
  BaseType2[BaseType2["Array"] = 17] = "Array";
  BaseType2[BaseType2["Vector64"] = 18] = "Vector64";
  BaseType2[BaseType2["MaxBaseType"] = 19] = "MaxBaseType";
})(BaseType || (BaseType = {}));

// reflection/enum.js
var flatbuffers4 = __toESM(require("flatbuffers"), 1);

// reflection/enum-val.js
var flatbuffers3 = __toESM(require("flatbuffers"), 1);

// reflection/key-value.js
var flatbuffers = __toESM(require("flatbuffers"), 1);
var KeyValue = class _KeyValue {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsKeyValue(bb, obj) {
    return (obj || new _KeyValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsKeyValue(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
    return (obj || new _KeyValue()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  key(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  value(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static getFullyQualifiedName() {
    return "reflection.KeyValue";
  }
  static startKeyValue(builder) {
    builder.startObject(2);
  }
  static addKey(builder, keyOffset) {
    builder.addFieldOffset(0, keyOffset, 0);
  }
  static addValue(builder, valueOffset) {
    builder.addFieldOffset(1, valueOffset, 0);
  }
  static endKeyValue(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    return offset;
  }
  static createKeyValue(builder, keyOffset, valueOffset) {
    _KeyValue.startKeyValue(builder);
    _KeyValue.addKey(builder, keyOffset);
    _KeyValue.addValue(builder, valueOffset);
    return _KeyValue.endKeyValue(builder);
  }
  unpack() {
    return new KeyValueT(this.key(), this.value());
  }
  unpackTo(_o) {
    _o.key = this.key();
    _o.value = this.value();
  }
};
var KeyValueT = class {
  constructor(key = null, value = null) {
    this.key = key;
    this.value = value;
  }
  pack(builder) {
    const key = this.key !== null ? builder.createString(this.key) : 0;
    const value = this.value !== null ? builder.createString(this.value) : 0;
    return KeyValue.createKeyValue(builder, key, value);
  }
};

// reflection/type.js
var flatbuffers2 = __toESM(require("flatbuffers"), 1);
var Type = class _Type {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsType(bb, obj) {
    return (obj || new _Type()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsType(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers2.SIZE_PREFIX_LENGTH);
    return (obj || new _Type()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  baseType() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readInt8(this.bb_pos + offset) : BaseType.None;
  }
  mutate_base_type(value) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, value);
    return true;
  }
  element() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt8(this.bb_pos + offset) : BaseType.None;
  }
  mutate_element(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, value);
    return true;
  }
  index() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : -1;
  }
  mutate_index(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  fixedLength() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_fixed_length(value) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  /**
   * The size (octets) of the `base_type` field.
   */
  baseSize() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 4;
  }
  mutate_base_size(value) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint32(this.bb_pos + offset, value);
    return true;
  }
  /**
   * The size (octets) of the `element` field, if present.
   */
  elementSize() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  mutate_element_size(value) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint32(this.bb_pos + offset, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "reflection.Type";
  }
  static startType(builder) {
    builder.startObject(6);
  }
  static addBaseType(builder, baseType) {
    builder.addFieldInt8(0, baseType, BaseType.None);
  }
  static addElement(builder, element) {
    builder.addFieldInt8(1, element, BaseType.None);
  }
  static addIndex(builder, index) {
    builder.addFieldInt32(2, index, -1);
  }
  static addFixedLength(builder, fixedLength) {
    builder.addFieldInt16(3, fixedLength, 0);
  }
  static addBaseSize(builder, baseSize) {
    builder.addFieldInt32(4, baseSize, 4);
  }
  static addElementSize(builder, elementSize) {
    builder.addFieldInt32(5, elementSize, 0);
  }
  static endType(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createType(builder, baseType, element, index, fixedLength, baseSize, elementSize) {
    _Type.startType(builder);
    _Type.addBaseType(builder, baseType);
    _Type.addElement(builder, element);
    _Type.addIndex(builder, index);
    _Type.addFixedLength(builder, fixedLength);
    _Type.addBaseSize(builder, baseSize);
    _Type.addElementSize(builder, elementSize);
    return _Type.endType(builder);
  }
  unpack() {
    return new TypeT(this.baseType(), this.element(), this.index(), this.fixedLength(), this.baseSize(), this.elementSize());
  }
  unpackTo(_o) {
    _o.baseType = this.baseType();
    _o.element = this.element();
    _o.index = this.index();
    _o.fixedLength = this.fixedLength();
    _o.baseSize = this.baseSize();
    _o.elementSize = this.elementSize();
  }
};
var TypeT = class {
  constructor(baseType = BaseType.None, element = BaseType.None, index = -1, fixedLength = 0, baseSize = 4, elementSize = 0) {
    this.baseType = baseType;
    this.element = element;
    this.index = index;
    this.fixedLength = fixedLength;
    this.baseSize = baseSize;
    this.elementSize = elementSize;
  }
  pack(builder) {
    return Type.createType(builder, this.baseType, this.element, this.index, this.fixedLength, this.baseSize, this.elementSize);
  }
};

// reflection/enum-val.js
var EnumVal = class _EnumVal {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsEnumVal(bb, obj) {
    return (obj || new _EnumVal()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsEnumVal(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers3.SIZE_PREFIX_LENGTH);
    return (obj || new _EnumVal()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  value() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_value(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  unionType(obj) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? (obj || new Type()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static getFullyQualifiedName() {
    return "reflection.EnumVal";
  }
  static startEnumVal(builder) {
    builder.startObject(6);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addValue(builder, value) {
    builder.addFieldInt64(1, value, BigInt("0"));
  }
  static addUnionType(builder, unionTypeOffset) {
    builder.addFieldOffset(3, unionTypeOffset, 0);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(4, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(5, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endEnumVal(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    return offset;
  }
  unpack() {
    return new EnumValT(this.name(), this.value(), this.unionType() !== null ? this.unionType().unpack() : null, this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()), this.bb.createObjList(this.attributes.bind(this), this.attributesLength()));
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.value = this.value();
    _o.unionType = this.unionType() !== null ? this.unionType().unpack() : null;
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
  }
};
var EnumValT = class {
  constructor(name = null, value = BigInt("0"), unionType = null, documentation = [], attributes = []) {
    this.name = name;
    this.value = value;
    this.unionType = unionType;
    this.documentation = documentation;
    this.attributes = attributes;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const unionType = this.unionType !== null ? this.unionType.pack(builder) : 0;
    const documentation = EnumVal.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    const attributes = EnumVal.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    EnumVal.startEnumVal(builder);
    EnumVal.addName(builder, name);
    EnumVal.addValue(builder, this.value);
    EnumVal.addUnionType(builder, unionType);
    EnumVal.addDocumentation(builder, documentation);
    EnumVal.addAttributes(builder, attributes);
    return EnumVal.endEnumVal(builder);
  }
};

// reflection/enum.js
var Enum = class _Enum {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsEnum(bb, obj) {
    return (obj || new _Enum()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsEnum(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers4.SIZE_PREFIX_LENGTH);
    return (obj || new _Enum()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  values(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new EnumVal()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  valuesLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  isUnion() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_is_union(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  underlyingType(obj) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? (obj || new Type()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  declarationFile(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static getFullyQualifiedName() {
    return "reflection.Enum";
  }
  static startEnum(builder) {
    builder.startObject(7);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addValues(builder, valuesOffset) {
    builder.addFieldOffset(1, valuesOffset, 0);
  }
  static createValuesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startValuesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addIsUnion(builder, isUnion) {
    builder.addFieldInt8(2, +isUnion, 0);
  }
  static addUnderlyingType(builder, underlyingTypeOffset) {
    builder.addFieldOffset(3, underlyingTypeOffset, 0);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(4, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(5, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDeclarationFile(builder, declarationFileOffset) {
    builder.addFieldOffset(6, declarationFileOffset, 0);
  }
  static endEnum(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    builder.requiredField(offset, 6);
    builder.requiredField(offset, 10);
    return offset;
  }
  unpack() {
    return new EnumT(this.name(), this.bb.createObjList(this.values.bind(this), this.valuesLength()), this.isUnion(), this.underlyingType() !== null ? this.underlyingType().unpack() : null, this.bb.createObjList(this.attributes.bind(this), this.attributesLength()), this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()), this.declarationFile());
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.values = this.bb.createObjList(this.values.bind(this), this.valuesLength());
    _o.isUnion = this.isUnion();
    _o.underlyingType = this.underlyingType() !== null ? this.underlyingType().unpack() : null;
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
    _o.declarationFile = this.declarationFile();
  }
};
var EnumT = class {
  constructor(name = null, values = [], isUnion = false, underlyingType = null, attributes = [], documentation = [], declarationFile = null) {
    this.name = name;
    this.values = values;
    this.isUnion = isUnion;
    this.underlyingType = underlyingType;
    this.attributes = attributes;
    this.documentation = documentation;
    this.declarationFile = declarationFile;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const values = Enum.createValuesVector(builder, builder.createObjectOffsetList(this.values));
    const underlyingType = this.underlyingType !== null ? this.underlyingType.pack(builder) : 0;
    const attributes = Enum.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    const documentation = Enum.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    const declarationFile = this.declarationFile !== null ? builder.createString(this.declarationFile) : 0;
    Enum.startEnum(builder);
    Enum.addName(builder, name);
    Enum.addValues(builder, values);
    Enum.addIsUnion(builder, this.isUnion);
    Enum.addUnderlyingType(builder, underlyingType);
    Enum.addAttributes(builder, attributes);
    Enum.addDocumentation(builder, documentation);
    Enum.addDeclarationFile(builder, declarationFile);
    return Enum.endEnum(builder);
  }
};

// reflection/field.js
var flatbuffers5 = __toESM(require("flatbuffers"), 1);
var Field = class _Field {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsField(bb, obj) {
    return (obj || new _Field()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsField(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers5.SIZE_PREFIX_LENGTH);
    return (obj || new _Field()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  type(obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new Type()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  id() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_id(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  offset() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_offset(value) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  defaultInteger() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_default_integer(value) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  defaultReal() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readFloat64(this.bb_pos + offset) : 0;
  }
  mutate_default_real(value) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat64(this.bb_pos + offset, value);
    return true;
  }
  deprecated() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_deprecated(value) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  required() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_required(value) {
    const offset = this.bb.__offset(this.bb_pos, 18);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  key() {
    const offset = this.bb.__offset(this.bb_pos, 20);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_key(value) {
    const offset = this.bb.__offset(this.bb_pos, 20);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  optional() {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_optional(value) {
    const offset = this.bb.__offset(this.bb_pos, 26);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  /**
   * Number of padding octets to always add after this field. Structs only.
   */
  padding() {
    const offset = this.bb.__offset(this.bb_pos, 28);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_padding(value) {
    const offset = this.bb.__offset(this.bb_pos, 28);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  /**
   * If the field uses 64-bit offsets.
   */
  offset64() {
    const offset = this.bb.__offset(this.bb_pos, 30);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_offset64(value) {
    const offset = this.bb.__offset(this.bb_pos, 30);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  static getFullyQualifiedName() {
    return "reflection.Field";
  }
  static startField(builder) {
    builder.startObject(14);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addType(builder, typeOffset) {
    builder.addFieldOffset(1, typeOffset, 0);
  }
  static addId(builder, id) {
    builder.addFieldInt16(2, id, 0);
  }
  static addOffset(builder, offset) {
    builder.addFieldInt16(3, offset, 0);
  }
  static addDefaultInteger(builder, defaultInteger) {
    builder.addFieldInt64(4, defaultInteger, BigInt("0"));
  }
  static addDefaultReal(builder, defaultReal) {
    builder.addFieldFloat64(5, defaultReal, 0);
  }
  static addDeprecated(builder, deprecated) {
    builder.addFieldInt8(6, +deprecated, 0);
  }
  static addRequired(builder, required) {
    builder.addFieldInt8(7, +required, 0);
  }
  static addKey(builder, key) {
    builder.addFieldInt8(8, +key, 0);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(9, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(10, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addOptional(builder, optional) {
    builder.addFieldInt8(11, +optional, 0);
  }
  static addPadding(builder, padding) {
    builder.addFieldInt16(12, padding, 0);
  }
  static addOffset64(builder, offset64) {
    builder.addFieldInt8(13, +offset64, 0);
  }
  static endField(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    builder.requiredField(offset, 6);
    return offset;
  }
  unpack() {
    return new FieldT(this.name(), this.type() !== null ? this.type().unpack() : null, this.id(), this.offset(), this.defaultInteger(), this.defaultReal(), this.deprecated(), this.required(), this.key(), this.bb.createObjList(this.attributes.bind(this), this.attributesLength()), this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()), this.optional(), this.padding(), this.offset64());
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.type = this.type() !== null ? this.type().unpack() : null;
    _o.id = this.id();
    _o.offset = this.offset();
    _o.defaultInteger = this.defaultInteger();
    _o.defaultReal = this.defaultReal();
    _o.deprecated = this.deprecated();
    _o.required = this.required();
    _o.key = this.key();
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
    _o.optional = this.optional();
    _o.padding = this.padding();
    _o.offset64 = this.offset64();
  }
};
var FieldT = class {
  constructor(name = null, type = null, id = 0, offset = 0, defaultInteger = BigInt("0"), defaultReal = 0, deprecated = false, required = false, key = false, attributes = [], documentation = [], optional = false, padding = 0, offset64 = false) {
    this.name = name;
    this.type = type;
    this.id = id;
    this.offset = offset;
    this.defaultInteger = defaultInteger;
    this.defaultReal = defaultReal;
    this.deprecated = deprecated;
    this.required = required;
    this.key = key;
    this.attributes = attributes;
    this.documentation = documentation;
    this.optional = optional;
    this.padding = padding;
    this.offset64 = offset64;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const type = this.type !== null ? this.type.pack(builder) : 0;
    const attributes = Field.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    const documentation = Field.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    Field.startField(builder);
    Field.addName(builder, name);
    Field.addType(builder, type);
    Field.addId(builder, this.id);
    Field.addOffset(builder, this.offset);
    Field.addDefaultInteger(builder, this.defaultInteger);
    Field.addDefaultReal(builder, this.defaultReal);
    Field.addDeprecated(builder, this.deprecated);
    Field.addRequired(builder, this.required);
    Field.addKey(builder, this.key);
    Field.addAttributes(builder, attributes);
    Field.addDocumentation(builder, documentation);
    Field.addOptional(builder, this.optional);
    Field.addPadding(builder, this.padding);
    Field.addOffset64(builder, this.offset64);
    return Field.endField(builder);
  }
};

// reflection/object.js
var flatbuffers6 = __toESM(require("flatbuffers"), 1);
var Object_ = class _Object_ {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsObject(bb, obj) {
    return (obj || new _Object_()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsObject(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers6.SIZE_PREFIX_LENGTH);
    return (obj || new _Object_()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  fields(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new Field()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  fieldsLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  isStruct() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_is_struct(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  minalign() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_minalign(value) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  bytesize() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_bytesize(value) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  declarationFile(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static getFullyQualifiedName() {
    return "reflection.Object";
  }
  static startObject(builder) {
    builder.startObject(8);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addFields(builder, fieldsOffset) {
    builder.addFieldOffset(1, fieldsOffset, 0);
  }
  static createFieldsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startFieldsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addIsStruct(builder, isStruct) {
    builder.addFieldInt8(2, +isStruct, 0);
  }
  static addMinalign(builder, minalign) {
    builder.addFieldInt32(3, minalign, 0);
  }
  static addBytesize(builder, bytesize) {
    builder.addFieldInt32(4, bytesize, 0);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(5, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(6, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDeclarationFile(builder, declarationFileOffset) {
    builder.addFieldOffset(7, declarationFileOffset, 0);
  }
  static endObject(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    builder.requiredField(offset, 6);
    return offset;
  }
  static createObject(builder, nameOffset, fieldsOffset, isStruct, minalign, bytesize, attributesOffset, documentationOffset, declarationFileOffset) {
    _Object_.startObject(builder);
    _Object_.addName(builder, nameOffset);
    _Object_.addFields(builder, fieldsOffset);
    _Object_.addIsStruct(builder, isStruct);
    _Object_.addMinalign(builder, minalign);
    _Object_.addBytesize(builder, bytesize);
    _Object_.addAttributes(builder, attributesOffset);
    _Object_.addDocumentation(builder, documentationOffset);
    _Object_.addDeclarationFile(builder, declarationFileOffset);
    return _Object_.endObject(builder);
  }
  unpack() {
    return new Object_T(this.name(), this.bb.createObjList(this.fields.bind(this), this.fieldsLength()), this.isStruct(), this.minalign(), this.bytesize(), this.bb.createObjList(this.attributes.bind(this), this.attributesLength()), this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()), this.declarationFile());
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.fields = this.bb.createObjList(this.fields.bind(this), this.fieldsLength());
    _o.isStruct = this.isStruct();
    _o.minalign = this.minalign();
    _o.bytesize = this.bytesize();
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
    _o.declarationFile = this.declarationFile();
  }
};
var Object_T = class {
  constructor(name = null, fields = [], isStruct = false, minalign = 0, bytesize = 0, attributes = [], documentation = [], declarationFile = null) {
    this.name = name;
    this.fields = fields;
    this.isStruct = isStruct;
    this.minalign = minalign;
    this.bytesize = bytesize;
    this.attributes = attributes;
    this.documentation = documentation;
    this.declarationFile = declarationFile;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const fields = Object_.createFieldsVector(builder, builder.createObjectOffsetList(this.fields));
    const attributes = Object_.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    const documentation = Object_.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    const declarationFile = this.declarationFile !== null ? builder.createString(this.declarationFile) : 0;
    return Object_.createObject(builder, name, fields, this.isStruct, this.minalign, this.bytesize, attributes, documentation, declarationFile);
  }
};

// reflection/rpccall.js
var flatbuffers7 = __toESM(require("flatbuffers"), 1);
var RPCCall = class _RPCCall {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsRPCCall(bb, obj) {
    return (obj || new _RPCCall()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsRPCCall(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers7.SIZE_PREFIX_LENGTH);
    return (obj || new _RPCCall()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  request(obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new Object_()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  response(obj) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? (obj || new Object_()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static getFullyQualifiedName() {
    return "reflection.RPCCall";
  }
  static startRPCCall(builder) {
    builder.startObject(5);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addRequest(builder, requestOffset) {
    builder.addFieldOffset(1, requestOffset, 0);
  }
  static addResponse(builder, responseOffset) {
    builder.addFieldOffset(2, responseOffset, 0);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(3, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(4, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endRPCCall(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    builder.requiredField(offset, 6);
    builder.requiredField(offset, 8);
    return offset;
  }
  unpack() {
    return new RPCCallT(this.name(), this.request() !== null ? this.request().unpack() : null, this.response() !== null ? this.response().unpack() : null, this.bb.createObjList(this.attributes.bind(this), this.attributesLength()), this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()));
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.request = this.request() !== null ? this.request().unpack() : null;
    _o.response = this.response() !== null ? this.response().unpack() : null;
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
  }
};
var RPCCallT = class {
  constructor(name = null, request = null, response = null, attributes = [], documentation = []) {
    this.name = name;
    this.request = request;
    this.response = response;
    this.attributes = attributes;
    this.documentation = documentation;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const request = this.request !== null ? this.request.pack(builder) : 0;
    const response = this.response !== null ? this.response.pack(builder) : 0;
    const attributes = RPCCall.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    const documentation = RPCCall.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    RPCCall.startRPCCall(builder);
    RPCCall.addName(builder, name);
    RPCCall.addRequest(builder, request);
    RPCCall.addResponse(builder, response);
    RPCCall.addAttributes(builder, attributes);
    RPCCall.addDocumentation(builder, documentation);
    return RPCCall.endRPCCall(builder);
  }
};

// reflection/schema.js
var flatbuffers10 = __toESM(require("flatbuffers"), 1);

// reflection/schema-file.js
var flatbuffers8 = __toESM(require("flatbuffers"), 1);
var SchemaFile = class _SchemaFile {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsSchemaFile(bb, obj) {
    return (obj || new _SchemaFile()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsSchemaFile(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers8.SIZE_PREFIX_LENGTH);
    return (obj || new _SchemaFile()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  filename(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  includedFilenames(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  includedFilenamesLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static getFullyQualifiedName() {
    return "reflection.SchemaFile";
  }
  static startSchemaFile(builder) {
    builder.startObject(2);
  }
  static addFilename(builder, filenameOffset) {
    builder.addFieldOffset(0, filenameOffset, 0);
  }
  static addIncludedFilenames(builder, includedFilenamesOffset) {
    builder.addFieldOffset(1, includedFilenamesOffset, 0);
  }
  static createIncludedFilenamesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startIncludedFilenamesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endSchemaFile(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    return offset;
  }
  static createSchemaFile(builder, filenameOffset, includedFilenamesOffset) {
    _SchemaFile.startSchemaFile(builder);
    _SchemaFile.addFilename(builder, filenameOffset);
    _SchemaFile.addIncludedFilenames(builder, includedFilenamesOffset);
    return _SchemaFile.endSchemaFile(builder);
  }
  unpack() {
    return new SchemaFileT(this.filename(), this.bb.createScalarList(this.includedFilenames.bind(this), this.includedFilenamesLength()));
  }
  unpackTo(_o) {
    _o.filename = this.filename();
    _o.includedFilenames = this.bb.createScalarList(this.includedFilenames.bind(this), this.includedFilenamesLength());
  }
};
var SchemaFileT = class {
  constructor(filename = null, includedFilenames = []) {
    this.filename = filename;
    this.includedFilenames = includedFilenames;
  }
  pack(builder) {
    const filename = this.filename !== null ? builder.createString(this.filename) : 0;
    const includedFilenames = SchemaFile.createIncludedFilenamesVector(builder, builder.createObjectOffsetList(this.includedFilenames));
    return SchemaFile.createSchemaFile(builder, filename, includedFilenames);
  }
};

// reflection/service.js
var flatbuffers9 = __toESM(require("flatbuffers"), 1);
var Service = class _Service {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsService(bb, obj) {
    return (obj || new _Service()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsService(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers9.SIZE_PREFIX_LENGTH);
    return (obj || new _Service()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  calls(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new RPCCall()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  callsLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  attributes(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? (obj || new KeyValue()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  attributesLength() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  documentation(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  documentationLength() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  declarationFile(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  static getFullyQualifiedName() {
    return "reflection.Service";
  }
  static startService(builder) {
    builder.startObject(5);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(0, nameOffset, 0);
  }
  static addCalls(builder, callsOffset) {
    builder.addFieldOffset(1, callsOffset, 0);
  }
  static createCallsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startCallsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addAttributes(builder, attributesOffset) {
    builder.addFieldOffset(2, attributesOffset, 0);
  }
  static createAttributesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startAttributesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDocumentation(builder, documentationOffset) {
    builder.addFieldOffset(3, documentationOffset, 0);
  }
  static createDocumentationVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startDocumentationVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addDeclarationFile(builder, declarationFileOffset) {
    builder.addFieldOffset(4, declarationFileOffset, 0);
  }
  static endService(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    return offset;
  }
  static createService(builder, nameOffset, callsOffset, attributesOffset, documentationOffset, declarationFileOffset) {
    _Service.startService(builder);
    _Service.addName(builder, nameOffset);
    _Service.addCalls(builder, callsOffset);
    _Service.addAttributes(builder, attributesOffset);
    _Service.addDocumentation(builder, documentationOffset);
    _Service.addDeclarationFile(builder, declarationFileOffset);
    return _Service.endService(builder);
  }
  unpack() {
    return new ServiceT(this.name(), this.bb.createObjList(this.calls.bind(this), this.callsLength()), this.bb.createObjList(this.attributes.bind(this), this.attributesLength()), this.bb.createScalarList(this.documentation.bind(this), this.documentationLength()), this.declarationFile());
  }
  unpackTo(_o) {
    _o.name = this.name();
    _o.calls = this.bb.createObjList(this.calls.bind(this), this.callsLength());
    _o.attributes = this.bb.createObjList(this.attributes.bind(this), this.attributesLength());
    _o.documentation = this.bb.createScalarList(this.documentation.bind(this), this.documentationLength());
    _o.declarationFile = this.declarationFile();
  }
};
var ServiceT = class {
  constructor(name = null, calls = [], attributes = [], documentation = [], declarationFile = null) {
    this.name = name;
    this.calls = calls;
    this.attributes = attributes;
    this.documentation = documentation;
    this.declarationFile = declarationFile;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const calls = Service.createCallsVector(builder, builder.createObjectOffsetList(this.calls));
    const attributes = Service.createAttributesVector(builder, builder.createObjectOffsetList(this.attributes));
    const documentation = Service.createDocumentationVector(builder, builder.createObjectOffsetList(this.documentation));
    const declarationFile = this.declarationFile !== null ? builder.createString(this.declarationFile) : 0;
    return Service.createService(builder, name, calls, attributes, documentation, declarationFile);
  }
};

// reflection/schema.js
var Schema = class _Schema {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsSchema(bb, obj) {
    return (obj || new _Schema()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsSchema(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers10.SIZE_PREFIX_LENGTH);
    return (obj || new _Schema()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static bufferHasIdentifier(bb) {
    return bb.__has_identifier("BFBS");
  }
  objects(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new Object_()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  objectsLength() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  enums(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new Enum()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  enumsLength() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  fileIdent(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  fileExt(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  rootTable(obj) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? (obj || new Object_()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  services(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? (obj || new Service()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  servicesLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  advancedFeatures() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_advanced_features(value) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  /**
   * All the files used in this compilation. Files are relative to where
   * flatc was invoked.
   */
  fbsFiles(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? (obj || new SchemaFile()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  fbsFilesLength() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  static getFullyQualifiedName() {
    return "reflection.Schema";
  }
  static startSchema(builder) {
    builder.startObject(8);
  }
  static addObjects(builder, objectsOffset) {
    builder.addFieldOffset(0, objectsOffset, 0);
  }
  static createObjectsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startObjectsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addEnums(builder, enumsOffset) {
    builder.addFieldOffset(1, enumsOffset, 0);
  }
  static createEnumsVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startEnumsVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addFileIdent(builder, fileIdentOffset) {
    builder.addFieldOffset(2, fileIdentOffset, 0);
  }
  static addFileExt(builder, fileExtOffset) {
    builder.addFieldOffset(3, fileExtOffset, 0);
  }
  static addRootTable(builder, rootTableOffset) {
    builder.addFieldOffset(4, rootTableOffset, 0);
  }
  static addServices(builder, servicesOffset) {
    builder.addFieldOffset(5, servicesOffset, 0);
  }
  static createServicesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startServicesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addAdvancedFeatures(builder, advancedFeatures) {
    builder.addFieldInt64(6, advancedFeatures, BigInt("0"));
  }
  static addFbsFiles(builder, fbsFilesOffset) {
    builder.addFieldOffset(7, fbsFilesOffset, 0);
  }
  static createFbsFilesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startFbsFilesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static endSchema(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 4);
    builder.requiredField(offset, 6);
    return offset;
  }
  static finishSchemaBuffer(builder, offset) {
    builder.finish(offset, "BFBS");
  }
  static finishSizePrefixedSchemaBuffer(builder, offset) {
    builder.finish(offset, "BFBS", true);
  }
  unpack() {
    return new SchemaT(this.bb.createObjList(this.objects.bind(this), this.objectsLength()), this.bb.createObjList(this.enums.bind(this), this.enumsLength()), this.fileIdent(), this.fileExt(), this.rootTable() !== null ? this.rootTable().unpack() : null, this.bb.createObjList(this.services.bind(this), this.servicesLength()), this.advancedFeatures(), this.bb.createObjList(this.fbsFiles.bind(this), this.fbsFilesLength()));
  }
  unpackTo(_o) {
    _o.objects = this.bb.createObjList(this.objects.bind(this), this.objectsLength());
    _o.enums = this.bb.createObjList(this.enums.bind(this), this.enumsLength());
    _o.fileIdent = this.fileIdent();
    _o.fileExt = this.fileExt();
    _o.rootTable = this.rootTable() !== null ? this.rootTable().unpack() : null;
    _o.services = this.bb.createObjList(this.services.bind(this), this.servicesLength());
    _o.advancedFeatures = this.advancedFeatures();
    _o.fbsFiles = this.bb.createObjList(this.fbsFiles.bind(this), this.fbsFilesLength());
  }
};
var SchemaT = class {
  constructor(objects = [], enums = [], fileIdent = null, fileExt = null, rootTable = null, services = [], advancedFeatures = BigInt("0"), fbsFiles = []) {
    this.objects = objects;
    this.enums = enums;
    this.fileIdent = fileIdent;
    this.fileExt = fileExt;
    this.rootTable = rootTable;
    this.services = services;
    this.advancedFeatures = advancedFeatures;
    this.fbsFiles = fbsFiles;
  }
  pack(builder) {
    const objects = Schema.createObjectsVector(builder, builder.createObjectOffsetList(this.objects));
    const enums = Schema.createEnumsVector(builder, builder.createObjectOffsetList(this.enums));
    const fileIdent = this.fileIdent !== null ? builder.createString(this.fileIdent) : 0;
    const fileExt = this.fileExt !== null ? builder.createString(this.fileExt) : 0;
    const rootTable = this.rootTable !== null ? this.rootTable.pack(builder) : 0;
    const services = Schema.createServicesVector(builder, builder.createObjectOffsetList(this.services));
    const fbsFiles = Schema.createFbsFilesVector(builder, builder.createObjectOffsetList(this.fbsFiles));
    Schema.startSchema(builder);
    Schema.addObjects(builder, objects);
    Schema.addEnums(builder, enums);
    Schema.addFileIdent(builder, fileIdent);
    Schema.addFileExt(builder, fileExt);
    Schema.addRootTable(builder, rootTable);
    Schema.addServices(builder, services);
    Schema.addAdvancedFeatures(builder, this.advancedFeatures);
    Schema.addFbsFiles(builder, fbsFiles);
    return Schema.endSchema(builder);
  }
};

// typescript.js
var typescript_exports = {};
__export(typescript_exports, {
  Object_: () => Object_2,
  class_: () => class_2
});

// typescript/object.js
var flatbuffers11 = __toESM(require("flatbuffers"), 1);

// foobar/class.js
var class_;
(function(class_3) {
  class_3[class_3["arguments_"] = 0] = "arguments_";
})(class_ || (class_ = {}));

// typescript/class.js
var class_2;
(function(class_3) {
  class_3[class_3["new_"] = 0] = "new_";
  class_3[class_3["instanceof_"] = 1] = "instanceof_";
})(class_2 || (class_2 = {}));

// typescript/object.js
var Object_2 = class _Object_ {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsObject(bb, obj) {
    return (obj || new _Object_()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsObject(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers11.SIZE_PREFIX_LENGTH);
    return (obj || new _Object_()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  return_() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_return(value) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  if_() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_if(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  switch_() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_switch(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  enum_() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : class_2.new_;
  }
  mutate_enum(value) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  enum2() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : class_.arguments_;
  }
  mutate_enum2(value) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  enum3() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : Abc.a;
  }
  mutate_enum3(value) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  reflect(obj) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? (obj || new Schema()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static getFullyQualifiedName() {
    return "typescript.Object";
  }
  static startObject(builder) {
    builder.startObject(7);
  }
  static addReturn(builder, return_) {
    builder.addFieldInt32(0, return_, 0);
  }
  static addIf(builder, if_) {
    builder.addFieldInt32(1, if_, 0);
  }
  static addSwitch(builder, switch_) {
    builder.addFieldInt32(2, switch_, 0);
  }
  static addEnum(builder, enum_) {
    builder.addFieldInt32(3, enum_, class_2.new_);
  }
  static addEnum2(builder, enum2) {
    builder.addFieldInt32(4, enum2, class_.arguments_);
  }
  static addEnum3(builder, enum3) {
    builder.addFieldInt32(5, enum3, Abc.a);
  }
  static addReflect(builder, reflectOffset) {
    builder.addFieldOffset(6, reflectOffset, 0);
  }
  static endObject(builder) {
    const offset = builder.endObject();
    return offset;
  }
  unpack() {
    return new Object_T2(this.return_(), this.if_(), this.switch_(), this.enum_(), this.enum2(), this.enum3(), this.reflect() !== null ? this.reflect().unpack() : null);
  }
  unpackTo(_o) {
    _o.return_ = this.return_();
    _o.if_ = this.if_();
    _o.switch_ = this.switch_();
    _o.enum_ = this.enum_();
    _o.enum2 = this.enum2();
    _o.enum3 = this.enum3();
    _o.reflect = this.reflect() !== null ? this.reflect().unpack() : null;
  }
};
var Object_T2 = class {
  constructor(return_ = 0, if_ = 0, switch_ = 0, enum_ = class_2.new_, enum2 = class_.arguments_, enum3 = Abc.a, reflect = null) {
    this.return_ = return_;
    this.if_ = if_;
    this.switch_ = switch_;
    this.enum_ = enum_;
    this.enum2 = enum2;
    this.enum3 = enum3;
    this.reflect = reflect;
  }
  pack(builder) {
    const reflect = this.reflect !== null ? this.reflect.pack(builder) : 0;
    Object_2.startObject(builder);
    Object_2.addReturn(builder, this.return_);
    Object_2.addIf(builder, this.if_);
    Object_2.addSwitch(builder, this.switch_);
    Object_2.addEnum(builder, this.enum_);
    Object_2.addEnum2(builder, this.enum2);
    Object_2.addEnum3(builder, this.enum3);
    Object_2.addReflect(builder, reflect);
    return Object_2.endObject(builder);
  }
};
