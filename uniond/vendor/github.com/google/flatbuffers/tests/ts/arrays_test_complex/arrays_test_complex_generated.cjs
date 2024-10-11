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

// arrays_test_complex/my-game/example.ts
var example_exports = {};
__export(example_exports, {
  ArrayStruct: () => ArrayStruct,
  ArrayStructT: () => ArrayStructT,
  ArrayTable: () => ArrayTable,
  ArrayTableT: () => ArrayTableT,
  InnerStruct: () => InnerStruct,
  InnerStructT: () => InnerStructT,
  NestedStruct: () => NestedStruct,
  NestedStructT: () => NestedStructT,
  OuterStruct: () => OuterStruct,
  OuterStructT: () => OuterStructT,
  TestEnum: () => TestEnum
});
module.exports = __toCommonJS(example_exports);

// arrays_test_complex/my-game/example/inner-struct.js
var InnerStruct = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  a() {
    return this.bb.readFloat64(this.bb_pos);
  }
  b(index) {
    return this.bb.readUint8(this.bb_pos + 8 + index);
  }
  c() {
    return this.bb.readInt8(this.bb_pos + 21);
  }
  dUnderscore() {
    return this.bb.readInt64(this.bb_pos + 24);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.InnerStruct";
  }
  static sizeOf() {
    return 32;
  }
  static createInnerStruct(builder, a, b, c, d_underscore) {
    builder.prep(8, 32);
    builder.writeInt64(BigInt(d_underscore ?? 0));
    builder.pad(2);
    builder.writeInt8(c);
    for (let i = 12; i >= 0; --i) {
      builder.writeInt8(b?.[i] ?? 0);
    }
    builder.writeFloat64(a);
    return builder.offset();
  }
  unpack() {
    return new InnerStructT(this.a(), this.bb.createScalarList(this.b.bind(this), 13), this.c(), this.dUnderscore());
  }
  unpackTo(_o) {
    _o.a = this.a();
    _o.b = this.bb.createScalarList(this.b.bind(this), 13);
    _o.c = this.c();
    _o.dUnderscore = this.dUnderscore();
  }
};
var InnerStructT = class {
  constructor(a = 0, b = [], c = 0, dUnderscore = BigInt("0")) {
    this.a = a;
    this.b = b;
    this.c = c;
    this.dUnderscore = dUnderscore;
  }
  pack(builder) {
    return InnerStruct.createInnerStruct(builder, this.a, this.b, this.c, this.dUnderscore);
  }
};

// arrays_test_complex/my-game/example/outer-struct.js
var OuterStruct = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  a() {
    return !!this.bb.readInt8(this.bb_pos);
  }
  b() {
    return this.bb.readFloat64(this.bb_pos + 8);
  }
  cUnderscore(obj) {
    return (obj || new InnerStruct()).__init(this.bb_pos + 16, this.bb);
  }
  d(index, obj) {
    return (obj || new InnerStruct()).__init(this.bb_pos + 48 + index * 32, this.bb);
  }
  e(obj) {
    return (obj || new InnerStruct()).__init(this.bb_pos + 144, this.bb);
  }
  f(index) {
    return this.bb.readFloat64(this.bb_pos + 176 + index * 8);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.OuterStruct";
  }
  static sizeOf() {
    return 208;
  }
  static createOuterStruct(builder, a, b, c_underscore_a, c_underscore_b, c_underscore_c, c_underscore_d_underscore, d, e_a, e_b, e_c, e_d_underscore, f) {
    builder.prep(8, 208);
    for (let i = 3; i >= 0; --i) {
      builder.writeFloat64(f?.[i] ?? 0);
    }
    builder.prep(8, 32);
    builder.writeInt64(BigInt(e_d_underscore ?? 0));
    builder.pad(2);
    builder.writeInt8(e_c);
    for (let i = 12; i >= 0; --i) {
      builder.writeInt8(e_b?.[i] ?? 0);
    }
    builder.writeFloat64(e_a);
    for (let i = 2; i >= 0; --i) {
      const item = d?.[i];
      if (item instanceof InnerStructT) {
        item.pack(builder);
        continue;
      }
      InnerStruct.createInnerStruct(builder, item?.a, item?.b, item?.c, item?.dUnderscore);
    }
    builder.prep(8, 32);
    builder.writeInt64(BigInt(c_underscore_d_underscore ?? 0));
    builder.pad(2);
    builder.writeInt8(c_underscore_c);
    for (let i = 12; i >= 0; --i) {
      builder.writeInt8(c_underscore_b?.[i] ?? 0);
    }
    builder.writeFloat64(c_underscore_a);
    builder.writeFloat64(b);
    builder.pad(7);
    builder.writeInt8(Number(Boolean(a)));
    return builder.offset();
  }
  unpack() {
    return new OuterStructT(this.a(), this.b(), this.cUnderscore() !== null ? this.cUnderscore().unpack() : null, this.bb.createObjList(this.d.bind(this), 3), this.e() !== null ? this.e().unpack() : null, this.bb.createScalarList(this.f.bind(this), 4));
  }
  unpackTo(_o) {
    _o.a = this.a();
    _o.b = this.b();
    _o.cUnderscore = this.cUnderscore() !== null ? this.cUnderscore().unpack() : null;
    _o.d = this.bb.createObjList(this.d.bind(this), 3);
    _o.e = this.e() !== null ? this.e().unpack() : null;
    _o.f = this.bb.createScalarList(this.f.bind(this), 4);
  }
};
var OuterStructT = class {
  constructor(a = false, b = 0, cUnderscore = null, d = [], e = null, f = []) {
    this.a = a;
    this.b = b;
    this.cUnderscore = cUnderscore;
    this.d = d;
    this.e = e;
    this.f = f;
  }
  pack(builder) {
    return OuterStruct.createOuterStruct(builder, this.a, this.b, this.cUnderscore?.a ?? 0, this.cUnderscore?.b ?? [], this.cUnderscore?.c ?? 0, this.cUnderscore?.dUnderscore ?? BigInt(0), this.d, this.e?.a ?? 0, this.e?.b ?? [], this.e?.c ?? 0, this.e?.dUnderscore ?? BigInt(0), this.f);
  }
};

// arrays_test_complex/my-game/example/test-enum.js
var TestEnum;
(function(TestEnum2) {
  TestEnum2[TestEnum2["A"] = 0] = "A";
  TestEnum2[TestEnum2["B"] = 1] = "B";
  TestEnum2[TestEnum2["C"] = 2] = "C";
})(TestEnum || (TestEnum = {}));

// arrays_test_complex/my-game/example/nested-struct.js
var NestedStruct = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  a(index) {
    return this.bb.readInt32(this.bb_pos + 0 + index * 4);
  }
  b() {
    return this.bb.readInt8(this.bb_pos + 8);
  }
  cUnderscore(index) {
    return this.bb.readInt8(this.bb_pos + 9 + index);
  }
  dOuter(index, obj) {
    return (obj || new OuterStruct()).__init(this.bb_pos + 16 + index * 208, this.bb);
  }
  e(index) {
    return this.bb.readInt64(this.bb_pos + 1056 + index * 8);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.NestedStruct";
  }
  static sizeOf() {
    return 1072;
  }
  static createNestedStruct(builder, a, b, c_underscore, d_outer, e) {
    builder.prep(8, 1072);
    for (let i = 1; i >= 0; --i) {
      builder.writeInt64(BigInt(e?.[i] ?? 0));
    }
    for (let i = 4; i >= 0; --i) {
      const item = d_outer?.[i];
      if (item instanceof OuterStructT) {
        item.pack(builder);
        continue;
      }
      OuterStruct.createOuterStruct(builder, item?.a, item?.b, item?.cUnderscore?.a ?? 0, item?.cUnderscore?.b ?? [], item?.cUnderscore?.c ?? 0, item?.cUnderscore?.dUnderscore ?? BigInt(0), item?.d, item?.e?.a ?? 0, item?.e?.b ?? [], item?.e?.c ?? 0, item?.e?.dUnderscore ?? BigInt(0), item?.f);
    }
    builder.pad(5);
    for (let i = 1; i >= 0; --i) {
      builder.writeInt8(c_underscore?.[i] ?? 0);
    }
    builder.writeInt8(b);
    for (let i = 1; i >= 0; --i) {
      builder.writeInt32(a?.[i] ?? 0);
    }
    return builder.offset();
  }
  unpack() {
    return new NestedStructT(this.bb.createScalarList(this.a.bind(this), 2), this.b(), this.bb.createScalarList(this.cUnderscore.bind(this), 2), this.bb.createObjList(this.dOuter.bind(this), 5), this.bb.createScalarList(this.e.bind(this), 2));
  }
  unpackTo(_o) {
    _o.a = this.bb.createScalarList(this.a.bind(this), 2);
    _o.b = this.b();
    _o.cUnderscore = this.bb.createScalarList(this.cUnderscore.bind(this), 2);
    _o.dOuter = this.bb.createObjList(this.dOuter.bind(this), 5);
    _o.e = this.bb.createScalarList(this.e.bind(this), 2);
  }
};
var NestedStructT = class {
  constructor(a = [], b = TestEnum.A, cUnderscore = [TestEnum.A, TestEnum.A], dOuter = [], e = []) {
    this.a = a;
    this.b = b;
    this.cUnderscore = cUnderscore;
    this.dOuter = dOuter;
    this.e = e;
  }
  pack(builder) {
    return NestedStruct.createNestedStruct(builder, this.a, this.b, this.cUnderscore, this.dOuter, this.e);
  }
};

// arrays_test_complex/my-game/example/array-struct.js
var ArrayStruct = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  aUnderscore() {
    return this.bb.readFloat32(this.bb_pos);
  }
  bUnderscore(index) {
    return this.bb.readInt32(this.bb_pos + 4 + index * 4);
  }
  c() {
    return this.bb.readInt8(this.bb_pos + 64);
  }
  d(index, obj) {
    return (obj || new NestedStruct()).__init(this.bb_pos + 72 + index * 1072, this.bb);
  }
  e() {
    return this.bb.readInt32(this.bb_pos + 2216);
  }
  f(index, obj) {
    return (obj || new OuterStruct()).__init(this.bb_pos + 2224 + index * 208, this.bb);
  }
  g(index) {
    return this.bb.readInt64(this.bb_pos + 2640 + index * 8);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.ArrayStruct";
  }
  static sizeOf() {
    return 2656;
  }
  static createArrayStruct(builder, a_underscore, b_underscore, c, d, e, f, g) {
    builder.prep(8, 2656);
    for (let i = 1; i >= 0; --i) {
      builder.writeInt64(BigInt(g?.[i] ?? 0));
    }
    for (let i = 1; i >= 0; --i) {
      const item = f?.[i];
      if (item instanceof OuterStructT) {
        item.pack(builder);
        continue;
      }
      OuterStruct.createOuterStruct(builder, item?.a, item?.b, item?.cUnderscore?.a ?? 0, item?.cUnderscore?.b ?? [], item?.cUnderscore?.c ?? 0, item?.cUnderscore?.dUnderscore ?? BigInt(0), item?.d, item?.e?.a ?? 0, item?.e?.b ?? [], item?.e?.c ?? 0, item?.e?.dUnderscore ?? BigInt(0), item?.f);
    }
    builder.pad(4);
    builder.writeInt32(e);
    for (let i = 1; i >= 0; --i) {
      const item = d?.[i];
      if (item instanceof NestedStructT) {
        item.pack(builder);
        continue;
      }
      NestedStruct.createNestedStruct(builder, item?.a, item?.b, item?.cUnderscore, item?.dOuter, item?.e);
    }
    builder.pad(7);
    builder.writeInt8(c);
    for (let i = 14; i >= 0; --i) {
      builder.writeInt32(b_underscore?.[i] ?? 0);
    }
    builder.writeFloat32(a_underscore);
    return builder.offset();
  }
  unpack() {
    return new ArrayStructT(this.aUnderscore(), this.bb.createScalarList(this.bUnderscore.bind(this), 15), this.c(), this.bb.createObjList(this.d.bind(this), 2), this.e(), this.bb.createObjList(this.f.bind(this), 2), this.bb.createScalarList(this.g.bind(this), 2));
  }
  unpackTo(_o) {
    _o.aUnderscore = this.aUnderscore();
    _o.bUnderscore = this.bb.createScalarList(this.bUnderscore.bind(this), 15);
    _o.c = this.c();
    _o.d = this.bb.createObjList(this.d.bind(this), 2);
    _o.e = this.e();
    _o.f = this.bb.createObjList(this.f.bind(this), 2);
    _o.g = this.bb.createScalarList(this.g.bind(this), 2);
  }
};
var ArrayStructT = class {
  constructor(aUnderscore = 0, bUnderscore = [], c = 0, d = [], e = 0, f = [], g = []) {
    this.aUnderscore = aUnderscore;
    this.bUnderscore = bUnderscore;
    this.c = c;
    this.d = d;
    this.e = e;
    this.f = f;
    this.g = g;
  }
  pack(builder) {
    return ArrayStruct.createArrayStruct(builder, this.aUnderscore, this.bUnderscore, this.c, this.d, this.e, this.f, this.g);
  }
};

// arrays_test_complex/my-game/example/array-table.js
var flatbuffers = __toESM(require("flatbuffers"), 1);
var ArrayTable = class _ArrayTable {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsArrayTable(bb, obj) {
    return (obj || new _ArrayTable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsArrayTable(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
    return (obj || new _ArrayTable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static bufferHasIdentifier(bb) {
    return bb.__has_identifier("RHUB");
  }
  a(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  cUnderscore(obj) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? (obj || new ArrayStruct()).__init(this.bb_pos + offset, this.bb) : null;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.ArrayTable";
  }
  static startArrayTable(builder) {
    builder.startObject(2);
  }
  static addA(builder, aOffset) {
    builder.addFieldOffset(0, aOffset, 0);
  }
  static addCUnderscore(builder, cUnderscoreOffset) {
    builder.addFieldStruct(1, cUnderscoreOffset, 0);
  }
  static endArrayTable(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static finishArrayTableBuffer(builder, offset) {
    builder.finish(offset, "RHUB");
  }
  static finishSizePrefixedArrayTableBuffer(builder, offset) {
    builder.finish(offset, "RHUB", true);
  }
  unpack() {
    return new ArrayTableT(this.a(), this.cUnderscore() !== null ? this.cUnderscore().unpack() : null);
  }
  unpackTo(_o) {
    _o.a = this.a();
    _o.cUnderscore = this.cUnderscore() !== null ? this.cUnderscore().unpack() : null;
  }
};
var ArrayTableT = class {
  constructor(a = null, cUnderscore = null) {
    this.a = a;
    this.cUnderscore = cUnderscore;
  }
  pack(builder) {
    const a = this.a !== null ? builder.createString(this.a) : 0;
    ArrayTable.startArrayTable(builder);
    ArrayTable.addA(builder, a);
    ArrayTable.addCUnderscore(builder, this.cUnderscore !== null ? this.cUnderscore.pack(builder) : 0);
    return ArrayTable.endArrayTable(builder);
  }
};
