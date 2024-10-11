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

// monster_test.ts
var monster_test_exports = {};
__export(monster_test_exports, {
  MyGame: () => my_game_exports,
  TableA: () => TableA,
  TableAT: () => TableAT
});
module.exports = __toCommonJS(monster_test_exports);

// table-a.js
var flatbuffers2 = __toESM(require("flatbuffers"), 1);

// my-game/other-name-space/table-b.js
var flatbuffers = __toESM(require("flatbuffers"), 1);
var TableB = class _TableB {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsTableB(bb, obj) {
    return (obj || new _TableB()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsTableB(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
    return (obj || new _TableB()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  a(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new TableA()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static getFullyQualifiedName() {
    return "MyGame.OtherNameSpace.TableB";
  }
  static startTableB(builder) {
    builder.startObject(1);
  }
  static addA(builder, aOffset) {
    builder.addFieldOffset(0, aOffset, 0);
  }
  static endTableB(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createTableB(builder, aOffset) {
    _TableB.startTableB(builder);
    _TableB.addA(builder, aOffset);
    return _TableB.endTableB(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _TableB.getRootAsTableB(new flatbuffers.ByteBuffer(buffer));
  }
  unpack() {
    return new TableBT(this.a() !== null ? this.a().unpack() : null);
  }
  unpackTo(_o) {
    _o.a = this.a() !== null ? this.a().unpack() : null;
  }
};
var TableBT = class {
  constructor(a = null) {
    this.a = a;
  }
  pack(builder) {
    const a = this.a !== null ? this.a.pack(builder) : 0;
    return TableB.createTableB(builder, a);
  }
};

// table-a.js
var TableA = class _TableA {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsTableA(bb, obj) {
    return (obj || new _TableA()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsTableA(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers2.SIZE_PREFIX_LENGTH);
    return (obj || new _TableA()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  b(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new TableB()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  static getFullyQualifiedName() {
    return "TableA";
  }
  static startTableA(builder) {
    builder.startObject(1);
  }
  static addB(builder, bOffset) {
    builder.addFieldOffset(0, bOffset, 0);
  }
  static endTableA(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createTableA(builder, bOffset) {
    _TableA.startTableA(builder);
    _TableA.addB(builder, bOffset);
    return _TableA.endTableA(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _TableA.getRootAsTableA(new flatbuffers2.ByteBuffer(buffer));
  }
  unpack() {
    return new TableAT(this.b() !== null ? this.b().unpack() : null);
  }
  unpackTo(_o) {
    _o.b = this.b() !== null ? this.b().unpack() : null;
  }
};
var TableAT = class {
  constructor(b = null) {
    this.b = b;
  }
  pack(builder) {
    const b = this.b !== null ? this.b.pack(builder) : 0;
    return TableA.createTableA(builder, b);
  }
};

// my-game.js
var my_game_exports = {};
__export(my_game_exports, {
  Example: () => example_exports,
  Example2: () => example2_exports,
  InParentNamespace: () => InParentNamespace,
  InParentNamespaceT: () => InParentNamespaceT,
  OtherNameSpace: () => other_name_space_exports
});

// my-game/in-parent-namespace.js
var flatbuffers3 = __toESM(require("flatbuffers"), 1);
var InParentNamespace = class _InParentNamespace {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsInParentNamespace(bb, obj) {
    return (obj || new _InParentNamespace()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsInParentNamespace(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers3.SIZE_PREFIX_LENGTH);
    return (obj || new _InParentNamespace()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getFullyQualifiedName() {
    return "MyGame.InParentNamespace";
  }
  static startInParentNamespace(builder) {
    builder.startObject(0);
  }
  static endInParentNamespace(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createInParentNamespace(builder) {
    _InParentNamespace.startInParentNamespace(builder);
    return _InParentNamespace.endInParentNamespace(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _InParentNamespace.getRootAsInParentNamespace(new flatbuffers3.ByteBuffer(buffer));
  }
  unpack() {
    return new InParentNamespaceT();
  }
  unpackTo(_o) {
  }
};
var InParentNamespaceT = class {
  constructor() {
  }
  pack(builder) {
    return InParentNamespace.createInParentNamespace(builder);
  }
};

// my-game/example.js
var example_exports = {};
__export(example_exports, {
  Ability: () => Ability,
  AbilityT: () => AbilityT,
  Any: () => Any,
  AnyAmbiguousAliases: () => AnyAmbiguousAliases,
  AnyUniqueAliases: () => AnyUniqueAliases,
  Color: () => Color,
  LongEnum: () => LongEnum,
  Monster: () => Monster2,
  MonsterT: () => MonsterT2,
  Race: () => Race,
  Referrable: () => Referrable,
  ReferrableT: () => ReferrableT,
  Stat: () => Stat,
  StatT: () => StatT,
  StructOfStructs: () => StructOfStructs,
  StructOfStructsOfStructs: () => StructOfStructsOfStructs,
  StructOfStructsOfStructsT: () => StructOfStructsOfStructsT,
  StructOfStructsT: () => StructOfStructsT,
  Test: () => Test,
  TestSimpleTableWithEnum: () => TestSimpleTableWithEnum,
  TestSimpleTableWithEnumT: () => TestSimpleTableWithEnumT,
  TestT: () => TestT,
  TypeAliases: () => TypeAliases,
  TypeAliasesT: () => TypeAliasesT,
  Vec3: () => Vec3,
  Vec3T: () => Vec3T
});

// my-game/example/ability.js
var Ability = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  id() {
    return this.bb.readUint32(this.bb_pos);
  }
  mutate_id(value) {
    this.bb.writeUint32(this.bb_pos + 0, value);
    return true;
  }
  distance() {
    return this.bb.readUint32(this.bb_pos + 4);
  }
  mutate_distance(value) {
    this.bb.writeUint32(this.bb_pos + 4, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Ability";
  }
  static sizeOf() {
    return 8;
  }
  static createAbility(builder, id, distance) {
    builder.prep(4, 8);
    builder.writeInt32(distance);
    builder.writeInt32(id);
    return builder.offset();
  }
  unpack() {
    return new AbilityT(this.id(), this.distance());
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.distance = this.distance();
  }
};
var AbilityT = class {
  constructor(id = 0, distance = 0) {
    this.id = id;
    this.distance = distance;
  }
  pack(builder) {
    return Ability.createAbility(builder, this.id, this.distance);
  }
};

// my-game/example2/monster.js
var flatbuffers4 = __toESM(require("flatbuffers"), 1);
var Monster = class _Monster {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsMonster(bb, obj) {
    return (obj || new _Monster()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsMonster(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers4.SIZE_PREFIX_LENGTH);
    return (obj || new _Monster()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example2.Monster";
  }
  static startMonster(builder) {
    builder.startObject(0);
  }
  static endMonster(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createMonster(builder) {
    _Monster.startMonster(builder);
    return _Monster.endMonster(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _Monster.getRootAsMonster(new flatbuffers4.ByteBuffer(buffer));
  }
  unpack() {
    return new MonsterT();
  }
  unpackTo(_o) {
  }
};
var MonsterT = class {
  constructor() {
  }
  pack(builder) {
    return Monster.createMonster(builder);
  }
};

// my-game/example/monster.js
var flatbuffers8 = __toESM(require("flatbuffers"), 1);

// my-game/example/any-ambiguous-aliases.js
var AnyAmbiguousAliases;
(function(AnyAmbiguousAliases2) {
  AnyAmbiguousAliases2[AnyAmbiguousAliases2["NONE"] = 0] = "NONE";
  AnyAmbiguousAliases2[AnyAmbiguousAliases2["M1"] = 1] = "M1";
  AnyAmbiguousAliases2[AnyAmbiguousAliases2["M2"] = 2] = "M2";
  AnyAmbiguousAliases2[AnyAmbiguousAliases2["M3"] = 3] = "M3";
})(AnyAmbiguousAliases || (AnyAmbiguousAliases = {}));
function unionToAnyAmbiguousAliases(type, accessor) {
  switch (AnyAmbiguousAliases[type]) {
    case "NONE":
      return null;
    case "M1":
      return accessor(new Monster2());
    case "M2":
      return accessor(new Monster2());
    case "M3":
      return accessor(new Monster2());
    default:
      return null;
  }
}

// my-game/example/test-simple-table-with-enum.js
var flatbuffers5 = __toESM(require("flatbuffers"), 1);

// my-game/example/color.js
var Color;
(function(Color2) {
  Color2[Color2["Red"] = 1] = "Red";
  Color2[Color2["Green"] = 2] = "Green";
  Color2[Color2["Blue"] = 8] = "Blue";
})(Color || (Color = {}));

// my-game/example/test-simple-table-with-enum.js
var TestSimpleTableWithEnum = class _TestSimpleTableWithEnum {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsTestSimpleTableWithEnum(bb, obj) {
    return (obj || new _TestSimpleTableWithEnum()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsTestSimpleTableWithEnum(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers5.SIZE_PREFIX_LENGTH);
    return (obj || new _TestSimpleTableWithEnum()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  color() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : Color.Green;
  }
  mutate_color(value) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint8(this.bb_pos + offset, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.TestSimpleTableWithEnum";
  }
  static startTestSimpleTableWithEnum(builder) {
    builder.startObject(1);
  }
  static addColor(builder, color) {
    builder.addFieldInt8(0, color, Color.Green);
  }
  static endTestSimpleTableWithEnum(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createTestSimpleTableWithEnum(builder, color) {
    _TestSimpleTableWithEnum.startTestSimpleTableWithEnum(builder);
    _TestSimpleTableWithEnum.addColor(builder, color);
    return _TestSimpleTableWithEnum.endTestSimpleTableWithEnum(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _TestSimpleTableWithEnum.getRootAsTestSimpleTableWithEnum(new flatbuffers5.ByteBuffer(buffer));
  }
  unpack() {
    return new TestSimpleTableWithEnumT(this.color());
  }
  unpackTo(_o) {
    _o.color = this.color();
  }
};
var TestSimpleTableWithEnumT = class {
  constructor(color = Color.Green) {
    this.color = color;
  }
  pack(builder) {
    return TestSimpleTableWithEnum.createTestSimpleTableWithEnum(builder, this.color);
  }
};

// my-game/example/any-unique-aliases.js
var AnyUniqueAliases;
(function(AnyUniqueAliases2) {
  AnyUniqueAliases2[AnyUniqueAliases2["NONE"] = 0] = "NONE";
  AnyUniqueAliases2[AnyUniqueAliases2["M"] = 1] = "M";
  AnyUniqueAliases2[AnyUniqueAliases2["TS"] = 2] = "TS";
  AnyUniqueAliases2[AnyUniqueAliases2["M2"] = 3] = "M2";
})(AnyUniqueAliases || (AnyUniqueAliases = {}));
function unionToAnyUniqueAliases(type, accessor) {
  switch (AnyUniqueAliases[type]) {
    case "NONE":
      return null;
    case "M":
      return accessor(new Monster2());
    case "TS":
      return accessor(new TestSimpleTableWithEnum());
    case "M2":
      return accessor(new Monster());
    default:
      return null;
  }
}

// my-game/example/race.js
var Race;
(function(Race2) {
  Race2[Race2["None"] = -1] = "None";
  Race2[Race2["Human"] = 0] = "Human";
  Race2[Race2["Dwarf"] = 1] = "Dwarf";
  Race2[Race2["Elf"] = 2] = "Elf";
})(Race || (Race = {}));

// my-game/example/referrable.js
var flatbuffers6 = __toESM(require("flatbuffers"), 1);
var Referrable = class _Referrable {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsReferrable(bb, obj) {
    return (obj || new _Referrable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsReferrable(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers6.SIZE_PREFIX_LENGTH);
    return (obj || new _Referrable()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_id(value) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Referrable";
  }
  static startReferrable(builder) {
    builder.startObject(1);
  }
  static addId(builder, id) {
    builder.addFieldInt64(0, id, BigInt("0"));
  }
  static endReferrable(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createReferrable(builder, id) {
    _Referrable.startReferrable(builder);
    _Referrable.addId(builder, id);
    return _Referrable.endReferrable(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _Referrable.getRootAsReferrable(new flatbuffers6.ByteBuffer(buffer));
  }
  unpack() {
    return new ReferrableT(this.id());
  }
  unpackTo(_o) {
    _o.id = this.id();
  }
};
var ReferrableT = class {
  constructor(id = BigInt("0")) {
    this.id = id;
  }
  pack(builder) {
    return Referrable.createReferrable(builder, this.id);
  }
};

// my-game/example/stat.js
var flatbuffers7 = __toESM(require("flatbuffers"), 1);
var Stat = class _Stat {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsStat(bb, obj) {
    return (obj || new _Stat()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsStat(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers7.SIZE_PREFIX_LENGTH);
    return (obj || new _Stat()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  id(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  val() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_val(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  count() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_count(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Stat";
  }
  static startStat(builder) {
    builder.startObject(3);
  }
  static addId(builder, idOffset) {
    builder.addFieldOffset(0, idOffset, 0);
  }
  static addVal(builder, val) {
    builder.addFieldInt64(1, val, BigInt("0"));
  }
  static addCount(builder, count) {
    builder.addFieldInt16(2, count, 0);
  }
  static endStat(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createStat(builder, idOffset, val, count) {
    _Stat.startStat(builder);
    _Stat.addId(builder, idOffset);
    _Stat.addVal(builder, val);
    _Stat.addCount(builder, count);
    return _Stat.endStat(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _Stat.getRootAsStat(new flatbuffers7.ByteBuffer(buffer));
  }
  unpack() {
    return new StatT(this.id(), this.val(), this.count());
  }
  unpackTo(_o) {
    _o.id = this.id();
    _o.val = this.val();
    _o.count = this.count();
  }
};
var StatT = class {
  constructor(id = null, val = BigInt("0"), count = 0) {
    this.id = id;
    this.val = val;
    this.count = count;
  }
  pack(builder) {
    const id = this.id !== null ? builder.createString(this.id) : 0;
    return Stat.createStat(builder, id, this.val, this.count);
  }
};

// my-game/example/test.js
var Test = class {
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
    return this.bb.readInt16(this.bb_pos);
  }
  mutate_a(value) {
    this.bb.writeInt16(this.bb_pos + 0, value);
    return true;
  }
  b() {
    return this.bb.readInt8(this.bb_pos + 2);
  }
  mutate_b(value) {
    this.bb.writeInt8(this.bb_pos + 2, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Test";
  }
  static sizeOf() {
    return 4;
  }
  static createTest(builder, a, b) {
    builder.prep(2, 4);
    builder.pad(1);
    builder.writeInt8(b);
    builder.writeInt16(a);
    return builder.offset();
  }
  unpack() {
    return new TestT(this.a(), this.b());
  }
  unpackTo(_o) {
    _o.a = this.a();
    _o.b = this.b();
  }
};
var TestT = class {
  constructor(a = 0, b = 0) {
    this.a = a;
    this.b = b;
  }
  pack(builder) {
    return Test.createTest(builder, this.a, this.b);
  }
};

// my-game/example/vec3.js
var Vec3 = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  x() {
    return this.bb.readFloat32(this.bb_pos);
  }
  mutate_x(value) {
    this.bb.writeFloat32(this.bb_pos + 0, value);
    return true;
  }
  y() {
    return this.bb.readFloat32(this.bb_pos + 4);
  }
  mutate_y(value) {
    this.bb.writeFloat32(this.bb_pos + 4, value);
    return true;
  }
  z() {
    return this.bb.readFloat32(this.bb_pos + 8);
  }
  mutate_z(value) {
    this.bb.writeFloat32(this.bb_pos + 8, value);
    return true;
  }
  test1() {
    return this.bb.readFloat64(this.bb_pos + 16);
  }
  mutate_test1(value) {
    this.bb.writeFloat64(this.bb_pos + 16, value);
    return true;
  }
  test2() {
    return this.bb.readUint8(this.bb_pos + 24);
  }
  mutate_test2(value) {
    this.bb.writeUint8(this.bb_pos + 24, value);
    return true;
  }
  test3(obj) {
    return (obj || new Test()).__init(this.bb_pos + 26, this.bb);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Vec3";
  }
  static sizeOf() {
    return 32;
  }
  static createVec3(builder, x, y, z, test1, test2, test3_a, test3_b) {
    builder.prep(8, 32);
    builder.pad(2);
    builder.prep(2, 4);
    builder.pad(1);
    builder.writeInt8(test3_b);
    builder.writeInt16(test3_a);
    builder.pad(1);
    builder.writeInt8(test2);
    builder.writeFloat64(test1);
    builder.pad(4);
    builder.writeFloat32(z);
    builder.writeFloat32(y);
    builder.writeFloat32(x);
    return builder.offset();
  }
  unpack() {
    return new Vec3T(this.x(), this.y(), this.z(), this.test1(), this.test2(), this.test3() !== null ? this.test3().unpack() : null);
  }
  unpackTo(_o) {
    _o.x = this.x();
    _o.y = this.y();
    _o.z = this.z();
    _o.test1 = this.test1();
    _o.test2 = this.test2();
    _o.test3 = this.test3() !== null ? this.test3().unpack() : null;
  }
};
var Vec3T = class {
  constructor(x = 0, y = 0, z = 0, test1 = 0, test2 = Color.Red, test3 = null) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.test1 = test1;
    this.test2 = test2;
    this.test3 = test3;
  }
  pack(builder) {
    return Vec3.createVec3(builder, this.x, this.y, this.z, this.test1, this.test2, this.test3?.a ?? 0, this.test3?.b ?? 0);
  }
};

// my-game/example/monster.js
var Monster2 = class _Monster {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsMonster(bb, obj) {
    return (obj || new _Monster()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsMonster(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers8.SIZE_PREFIX_LENGTH);
    return (obj || new _Monster()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static bufferHasIdentifier(bb) {
    return bb.__has_identifier("MONS");
  }
  pos(obj) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? (obj || new Vec3()).__init(this.bb_pos + offset, this.bb) : null;
  }
  mana() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readInt16(this.bb_pos + offset) : 150;
  }
  mutate_mana(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt16(this.bb_pos + offset, value);
    return true;
  }
  hp() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readInt16(this.bb_pos + offset) : 100;
  }
  mutate_hp(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt16(this.bb_pos + offset, value);
    return true;
  }
  name(optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
  }
  inventory(index) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  inventoryLength() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  inventoryArray() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  color() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : Color.Blue;
  }
  mutate_color(value) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint8(this.bb_pos + offset, value);
    return true;
  }
  testType() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : Any.NONE;
  }
  test(obj) {
    const offset = this.bb.__offset(this.bb_pos, 20);
    return offset ? this.bb.__union(obj, this.bb_pos + offset) : null;
  }
  test4(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? (obj || new Test()).__init(this.bb.__vector(this.bb_pos + offset) + index * 4, this.bb) : null;
  }
  test4Length() {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  testarrayofstring(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  testarrayofstringLength() {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  /**
   * an example documentation comment: this will end up in the generated code
   * multiline too
   */
  testarrayoftables(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? (obj || new _Monster()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  testarrayoftablesLength() {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  enemy(obj) {
    const offset = this.bb.__offset(this.bb_pos, 28);
    return offset ? (obj || new _Monster()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  testnestedflatbuffer(index) {
    const offset = this.bb.__offset(this.bb_pos, 30);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  testnestedflatbufferLength() {
    const offset = this.bb.__offset(this.bb_pos, 30);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  testnestedflatbufferArray() {
    const offset = this.bb.__offset(this.bb_pos, 30);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  testempty(obj) {
    const offset = this.bb.__offset(this.bb_pos, 32);
    return offset ? (obj || new Stat()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  testbool() {
    const offset = this.bb.__offset(this.bb_pos, 34);
    return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
  }
  mutate_testbool(value) {
    const offset = this.bb.__offset(this.bb_pos, 34);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, +value);
    return true;
  }
  testhashs32Fnv1() {
    const offset = this.bb.__offset(this.bb_pos, 36);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_testhashs32_fnv1(value) {
    const offset = this.bb.__offset(this.bb_pos, 36);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  testhashu32Fnv1() {
    const offset = this.bb.__offset(this.bb_pos, 38);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  mutate_testhashu32_fnv1(value) {
    const offset = this.bb.__offset(this.bb_pos, 38);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint32(this.bb_pos + offset, value);
    return true;
  }
  testhashs64Fnv1() {
    const offset = this.bb.__offset(this.bb_pos, 40);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_testhashs64_fnv1(value) {
    const offset = this.bb.__offset(this.bb_pos, 40);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  testhashu64Fnv1() {
    const offset = this.bb.__offset(this.bb_pos, 42);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_testhashu64_fnv1(value) {
    const offset = this.bb.__offset(this.bb_pos, 42);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  testhashs32Fnv1a() {
    const offset = this.bb.__offset(this.bb_pos, 44);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_testhashs32_fnv1a(value) {
    const offset = this.bb.__offset(this.bb_pos, 44);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  testhashu32Fnv1a() {
    const offset = this.bb.__offset(this.bb_pos, 46);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  mutate_testhashu32_fnv1a(value) {
    const offset = this.bb.__offset(this.bb_pos, 46);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint32(this.bb_pos + offset, value);
    return true;
  }
  testhashs64Fnv1a() {
    const offset = this.bb.__offset(this.bb_pos, 48);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_testhashs64_fnv1a(value) {
    const offset = this.bb.__offset(this.bb_pos, 48);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  testhashu64Fnv1a() {
    const offset = this.bb.__offset(this.bb_pos, 50);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_testhashu64_fnv1a(value) {
    const offset = this.bb.__offset(this.bb_pos, 50);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  testarrayofbools(index) {
    const offset = this.bb.__offset(this.bb_pos, 52);
    return offset ? !!this.bb.readInt8(this.bb.__vector(this.bb_pos + offset) + index) : false;
  }
  testarrayofboolsLength() {
    const offset = this.bb.__offset(this.bb_pos, 52);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  testarrayofboolsArray() {
    const offset = this.bb.__offset(this.bb_pos, 52);
    return offset ? new Int8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  testf() {
    const offset = this.bb.__offset(this.bb_pos, 54);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : 3.14159;
  }
  mutate_testf(value) {
    const offset = this.bb.__offset(this.bb_pos, 54);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  testf2() {
    const offset = this.bb.__offset(this.bb_pos, 56);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : 3;
  }
  mutate_testf2(value) {
    const offset = this.bb.__offset(this.bb_pos, 56);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  testf3() {
    const offset = this.bb.__offset(this.bb_pos, 58);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : 0;
  }
  mutate_testf3(value) {
    const offset = this.bb.__offset(this.bb_pos, 58);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  testarrayofstring2(index, optionalEncoding) {
    const offset = this.bb.__offset(this.bb_pos, 60);
    return offset ? this.bb.__string(this.bb.__vector(this.bb_pos + offset) + index * 4, optionalEncoding) : null;
  }
  testarrayofstring2Length() {
    const offset = this.bb.__offset(this.bb_pos, 60);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  testarrayofsortedstruct(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 62);
    return offset ? (obj || new Ability()).__init(this.bb.__vector(this.bb_pos + offset) + index * 8, this.bb) : null;
  }
  testarrayofsortedstructLength() {
    const offset = this.bb.__offset(this.bb_pos, 62);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  flex(index) {
    const offset = this.bb.__offset(this.bb_pos, 64);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  flexLength() {
    const offset = this.bb.__offset(this.bb_pos, 64);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  flexArray() {
    const offset = this.bb.__offset(this.bb_pos, 64);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  test5(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 66);
    return offset ? (obj || new Test()).__init(this.bb.__vector(this.bb_pos + offset) + index * 4, this.bb) : null;
  }
  test5Length() {
    const offset = this.bb.__offset(this.bb_pos, 66);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vectorOfLongs(index) {
    const offset = this.bb.__offset(this.bb_pos, 68);
    return offset ? this.bb.readInt64(this.bb.__vector(this.bb_pos + offset) + index * 8) : BigInt(0);
  }
  vectorOfLongsLength() {
    const offset = this.bb.__offset(this.bb_pos, 68);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vectorOfDoubles(index) {
    const offset = this.bb.__offset(this.bb_pos, 70);
    return offset ? this.bb.readFloat64(this.bb.__vector(this.bb_pos + offset) + index * 8) : 0;
  }
  vectorOfDoublesLength() {
    const offset = this.bb.__offset(this.bb_pos, 70);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vectorOfDoublesArray() {
    const offset = this.bb.__offset(this.bb_pos, 70);
    return offset ? new Float64Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  parentNamespaceTest(obj) {
    const offset = this.bb.__offset(this.bb_pos, 72);
    return offset ? (obj || new InParentNamespace()).__init(this.bb.__indirect(this.bb_pos + offset), this.bb) : null;
  }
  vectorOfReferrables(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 74);
    return offset ? (obj || new Referrable()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  vectorOfReferrablesLength() {
    const offset = this.bb.__offset(this.bb_pos, 74);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  singleWeakReference() {
    const offset = this.bb.__offset(this.bb_pos, 76);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_single_weak_reference(value) {
    const offset = this.bb.__offset(this.bb_pos, 76);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  vectorOfWeakReferences(index) {
    const offset = this.bb.__offset(this.bb_pos, 78);
    return offset ? this.bb.readUint64(this.bb.__vector(this.bb_pos + offset) + index * 8) : BigInt(0);
  }
  vectorOfWeakReferencesLength() {
    const offset = this.bb.__offset(this.bb_pos, 78);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vectorOfStrongReferrables(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 80);
    return offset ? (obj || new Referrable()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  vectorOfStrongReferrablesLength() {
    const offset = this.bb.__offset(this.bb_pos, 80);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  coOwningReference() {
    const offset = this.bb.__offset(this.bb_pos, 82);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_co_owning_reference(value) {
    const offset = this.bb.__offset(this.bb_pos, 82);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  vectorOfCoOwningReferences(index) {
    const offset = this.bb.__offset(this.bb_pos, 84);
    return offset ? this.bb.readUint64(this.bb.__vector(this.bb_pos + offset) + index * 8) : BigInt(0);
  }
  vectorOfCoOwningReferencesLength() {
    const offset = this.bb.__offset(this.bb_pos, 84);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  nonOwningReference() {
    const offset = this.bb.__offset(this.bb_pos, 86);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_non_owning_reference(value) {
    const offset = this.bb.__offset(this.bb_pos, 86);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  vectorOfNonOwningReferences(index) {
    const offset = this.bb.__offset(this.bb_pos, 88);
    return offset ? this.bb.readUint64(this.bb.__vector(this.bb_pos + offset) + index * 8) : BigInt(0);
  }
  vectorOfNonOwningReferencesLength() {
    const offset = this.bb.__offset(this.bb_pos, 88);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  anyUniqueType() {
    const offset = this.bb.__offset(this.bb_pos, 90);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : AnyUniqueAliases.NONE;
  }
  anyUnique(obj) {
    const offset = this.bb.__offset(this.bb_pos, 92);
    return offset ? this.bb.__union(obj, this.bb_pos + offset) : null;
  }
  anyAmbiguousType() {
    const offset = this.bb.__offset(this.bb_pos, 94);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : AnyAmbiguousAliases.NONE;
  }
  anyAmbiguous(obj) {
    const offset = this.bb.__offset(this.bb_pos, 96);
    return offset ? this.bb.__union(obj, this.bb_pos + offset) : null;
  }
  vectorOfEnums(index) {
    const offset = this.bb.__offset(this.bb_pos, 98);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  vectorOfEnumsLength() {
    const offset = this.bb.__offset(this.bb_pos, 98);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vectorOfEnumsArray() {
    const offset = this.bb.__offset(this.bb_pos, 98);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  signedEnum() {
    const offset = this.bb.__offset(this.bb_pos, 100);
    return offset ? this.bb.readInt8(this.bb_pos + offset) : Race.None;
  }
  mutate_signed_enum(value) {
    const offset = this.bb.__offset(this.bb_pos, 100);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, value);
    return true;
  }
  testrequirednestedflatbuffer(index) {
    const offset = this.bb.__offset(this.bb_pos, 102);
    return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  testrequirednestedflatbufferLength() {
    const offset = this.bb.__offset(this.bb_pos, 102);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  testrequirednestedflatbufferArray() {
    const offset = this.bb.__offset(this.bb_pos, 102);
    return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  scalarKeySortedTables(index, obj) {
    const offset = this.bb.__offset(this.bb_pos, 104);
    return offset ? (obj || new Stat()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
  }
  scalarKeySortedTablesLength() {
    const offset = this.bb.__offset(this.bb_pos, 104);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  nativeInline(obj) {
    const offset = this.bb.__offset(this.bb_pos, 106);
    return offset ? (obj || new Test()).__init(this.bb_pos + offset, this.bb) : null;
  }
  longEnumNonEnumDefault() {
    const offset = this.bb.__offset(this.bb_pos, 108);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_long_enum_non_enum_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 108);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  longEnumNormalDefault() {
    const offset = this.bb.__offset(this.bb_pos, 110);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("2");
  }
  mutate_long_enum_normal_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 110);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  nanDefault() {
    const offset = this.bb.__offset(this.bb_pos, 112);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : NaN;
  }
  mutate_nan_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 112);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  infDefault() {
    const offset = this.bb.__offset(this.bb_pos, 114);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : Infinity;
  }
  mutate_inf_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 114);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  positiveInfDefault() {
    const offset = this.bb.__offset(this.bb_pos, 116);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : Infinity;
  }
  mutate_positive_inf_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 116);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  infinityDefault() {
    const offset = this.bb.__offset(this.bb_pos, 118);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : Infinity;
  }
  mutate_infinity_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 118);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  positiveInfinityDefault() {
    const offset = this.bb.__offset(this.bb_pos, 120);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : Infinity;
  }
  mutate_positive_infinity_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 120);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  negativeInfDefault() {
    const offset = this.bb.__offset(this.bb_pos, 122);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : -Infinity;
  }
  mutate_negative_inf_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 122);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  negativeInfinityDefault() {
    const offset = this.bb.__offset(this.bb_pos, 124);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : -Infinity;
  }
  mutate_negative_infinity_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 124);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  doubleInfDefault() {
    const offset = this.bb.__offset(this.bb_pos, 126);
    return offset ? this.bb.readFloat64(this.bb_pos + offset) : Infinity;
  }
  mutate_double_inf_default(value) {
    const offset = this.bb.__offset(this.bb_pos, 126);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat64(this.bb_pos + offset, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.Monster";
  }
  static startMonster(builder) {
    builder.startObject(62);
  }
  static addPos(builder, posOffset) {
    builder.addFieldStruct(0, posOffset, 0);
  }
  static addMana(builder, mana) {
    builder.addFieldInt16(1, mana, 150);
  }
  static addHp(builder, hp) {
    builder.addFieldInt16(2, hp, 100);
  }
  static addName(builder, nameOffset) {
    builder.addFieldOffset(3, nameOffset, 0);
  }
  static addInventory(builder, inventoryOffset) {
    builder.addFieldOffset(5, inventoryOffset, 0);
  }
  static createInventoryVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startInventoryVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addColor(builder, color) {
    builder.addFieldInt8(6, color, Color.Blue);
  }
  static addTestType(builder, testType) {
    builder.addFieldInt8(7, testType, Any.NONE);
  }
  static addTest(builder, testOffset) {
    builder.addFieldOffset(8, testOffset, 0);
  }
  static addTest4(builder, test4Offset) {
    builder.addFieldOffset(9, test4Offset, 0);
  }
  static startTest4Vector(builder, numElems) {
    builder.startVector(4, numElems, 2);
  }
  static addTestarrayofstring(builder, testarrayofstringOffset) {
    builder.addFieldOffset(10, testarrayofstringOffset, 0);
  }
  static createTestarrayofstringVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startTestarrayofstringVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addTestarrayoftables(builder, testarrayoftablesOffset) {
    builder.addFieldOffset(11, testarrayoftablesOffset, 0);
  }
  static createTestarrayoftablesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startTestarrayoftablesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addEnemy(builder, enemyOffset) {
    builder.addFieldOffset(12, enemyOffset, 0);
  }
  static addTestnestedflatbuffer(builder, testnestedflatbufferOffset) {
    builder.addFieldOffset(13, testnestedflatbufferOffset, 0);
  }
  static createTestnestedflatbufferVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startTestnestedflatbufferVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addTestempty(builder, testemptyOffset) {
    builder.addFieldOffset(14, testemptyOffset, 0);
  }
  static addTestbool(builder, testbool) {
    builder.addFieldInt8(15, +testbool, 0);
  }
  static addTesthashs32Fnv1(builder, testhashs32Fnv1) {
    builder.addFieldInt32(16, testhashs32Fnv1, 0);
  }
  static addTesthashu32Fnv1(builder, testhashu32Fnv1) {
    builder.addFieldInt32(17, testhashu32Fnv1, 0);
  }
  static addTesthashs64Fnv1(builder, testhashs64Fnv1) {
    builder.addFieldInt64(18, testhashs64Fnv1, BigInt("0"));
  }
  static addTesthashu64Fnv1(builder, testhashu64Fnv1) {
    builder.addFieldInt64(19, testhashu64Fnv1, BigInt("0"));
  }
  static addTesthashs32Fnv1a(builder, testhashs32Fnv1a) {
    builder.addFieldInt32(20, testhashs32Fnv1a, 0);
  }
  static addTesthashu32Fnv1a(builder, testhashu32Fnv1a) {
    builder.addFieldInt32(21, testhashu32Fnv1a, 0);
  }
  static addTesthashs64Fnv1a(builder, testhashs64Fnv1a) {
    builder.addFieldInt64(22, testhashs64Fnv1a, BigInt("0"));
  }
  static addTesthashu64Fnv1a(builder, testhashu64Fnv1a) {
    builder.addFieldInt64(23, testhashu64Fnv1a, BigInt("0"));
  }
  static addTestarrayofbools(builder, testarrayofboolsOffset) {
    builder.addFieldOffset(24, testarrayofboolsOffset, 0);
  }
  static createTestarrayofboolsVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(+data[i]);
    }
    return builder.endVector();
  }
  static startTestarrayofboolsVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addTestf(builder, testf) {
    builder.addFieldFloat32(25, testf, 3.14159);
  }
  static addTestf2(builder, testf2) {
    builder.addFieldFloat32(26, testf2, 3);
  }
  static addTestf3(builder, testf3) {
    builder.addFieldFloat32(27, testf3, 0);
  }
  static addTestarrayofstring2(builder, testarrayofstring2Offset) {
    builder.addFieldOffset(28, testarrayofstring2Offset, 0);
  }
  static createTestarrayofstring2Vector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startTestarrayofstring2Vector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addTestarrayofsortedstruct(builder, testarrayofsortedstructOffset) {
    builder.addFieldOffset(29, testarrayofsortedstructOffset, 0);
  }
  static startTestarrayofsortedstructVector(builder, numElems) {
    builder.startVector(8, numElems, 4);
  }
  static addFlex(builder, flexOffset) {
    builder.addFieldOffset(30, flexOffset, 0);
  }
  static createFlexVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startFlexVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addTest5(builder, test5Offset) {
    builder.addFieldOffset(31, test5Offset, 0);
  }
  static startTest5Vector(builder, numElems) {
    builder.startVector(4, numElems, 2);
  }
  static addVectorOfLongs(builder, vectorOfLongsOffset) {
    builder.addFieldOffset(32, vectorOfLongsOffset, 0);
  }
  static createVectorOfLongsVector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt64(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfLongsVector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static addVectorOfDoubles(builder, vectorOfDoublesOffset) {
    builder.addFieldOffset(33, vectorOfDoublesOffset, 0);
  }
  static createVectorOfDoublesVector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addFloat64(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfDoublesVector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static addParentNamespaceTest(builder, parentNamespaceTestOffset) {
    builder.addFieldOffset(34, parentNamespaceTestOffset, 0);
  }
  static addVectorOfReferrables(builder, vectorOfReferrablesOffset) {
    builder.addFieldOffset(35, vectorOfReferrablesOffset, 0);
  }
  static createVectorOfReferrablesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfReferrablesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addSingleWeakReference(builder, singleWeakReference) {
    builder.addFieldInt64(36, singleWeakReference, BigInt("0"));
  }
  static addVectorOfWeakReferences(builder, vectorOfWeakReferencesOffset) {
    builder.addFieldOffset(37, vectorOfWeakReferencesOffset, 0);
  }
  static createVectorOfWeakReferencesVector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt64(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfWeakReferencesVector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static addVectorOfStrongReferrables(builder, vectorOfStrongReferrablesOffset) {
    builder.addFieldOffset(38, vectorOfStrongReferrablesOffset, 0);
  }
  static createVectorOfStrongReferrablesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfStrongReferrablesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addCoOwningReference(builder, coOwningReference) {
    builder.addFieldInt64(39, coOwningReference, BigInt("0"));
  }
  static addVectorOfCoOwningReferences(builder, vectorOfCoOwningReferencesOffset) {
    builder.addFieldOffset(40, vectorOfCoOwningReferencesOffset, 0);
  }
  static createVectorOfCoOwningReferencesVector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt64(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfCoOwningReferencesVector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static addNonOwningReference(builder, nonOwningReference) {
    builder.addFieldInt64(41, nonOwningReference, BigInt("0"));
  }
  static addVectorOfNonOwningReferences(builder, vectorOfNonOwningReferencesOffset) {
    builder.addFieldOffset(42, vectorOfNonOwningReferencesOffset, 0);
  }
  static createVectorOfNonOwningReferencesVector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt64(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfNonOwningReferencesVector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static addAnyUniqueType(builder, anyUniqueType) {
    builder.addFieldInt8(43, anyUniqueType, AnyUniqueAliases.NONE);
  }
  static addAnyUnique(builder, anyUniqueOffset) {
    builder.addFieldOffset(44, anyUniqueOffset, 0);
  }
  static addAnyAmbiguousType(builder, anyAmbiguousType) {
    builder.addFieldInt8(45, anyAmbiguousType, AnyAmbiguousAliases.NONE);
  }
  static addAnyAmbiguous(builder, anyAmbiguousOffset) {
    builder.addFieldOffset(46, anyAmbiguousOffset, 0);
  }
  static addVectorOfEnums(builder, vectorOfEnumsOffset) {
    builder.addFieldOffset(47, vectorOfEnumsOffset, 0);
  }
  static createVectorOfEnumsVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startVectorOfEnumsVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addSignedEnum(builder, signedEnum) {
    builder.addFieldInt8(48, signedEnum, Race.None);
  }
  static addTestrequirednestedflatbuffer(builder, testrequirednestedflatbufferOffset) {
    builder.addFieldOffset(49, testrequirednestedflatbufferOffset, 0);
  }
  static createTestrequirednestedflatbufferVector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startTestrequirednestedflatbufferVector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addScalarKeySortedTables(builder, scalarKeySortedTablesOffset) {
    builder.addFieldOffset(50, scalarKeySortedTablesOffset, 0);
  }
  static createScalarKeySortedTablesVector(builder, data) {
    builder.startVector(4, data.length, 4);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addOffset(data[i]);
    }
    return builder.endVector();
  }
  static startScalarKeySortedTablesVector(builder, numElems) {
    builder.startVector(4, numElems, 4);
  }
  static addNativeInline(builder, nativeInlineOffset) {
    builder.addFieldStruct(51, nativeInlineOffset, 0);
  }
  static addLongEnumNonEnumDefault(builder, longEnumNonEnumDefault) {
    builder.addFieldInt64(52, longEnumNonEnumDefault, BigInt("0"));
  }
  static addLongEnumNormalDefault(builder, longEnumNormalDefault) {
    builder.addFieldInt64(53, longEnumNormalDefault, BigInt("2"));
  }
  static addNanDefault(builder, nanDefault) {
    builder.addFieldFloat32(54, nanDefault, NaN);
  }
  static addInfDefault(builder, infDefault) {
    builder.addFieldFloat32(55, infDefault, Infinity);
  }
  static addPositiveInfDefault(builder, positiveInfDefault) {
    builder.addFieldFloat32(56, positiveInfDefault, Infinity);
  }
  static addInfinityDefault(builder, infinityDefault) {
    builder.addFieldFloat32(57, infinityDefault, Infinity);
  }
  static addPositiveInfinityDefault(builder, positiveInfinityDefault) {
    builder.addFieldFloat32(58, positiveInfinityDefault, Infinity);
  }
  static addNegativeInfDefault(builder, negativeInfDefault) {
    builder.addFieldFloat32(59, negativeInfDefault, -Infinity);
  }
  static addNegativeInfinityDefault(builder, negativeInfinityDefault) {
    builder.addFieldFloat32(60, negativeInfinityDefault, -Infinity);
  }
  static addDoubleInfDefault(builder, doubleInfDefault) {
    builder.addFieldFloat64(61, doubleInfDefault, Infinity);
  }
  static endMonster(builder) {
    const offset = builder.endObject();
    builder.requiredField(offset, 10);
    return offset;
  }
  static finishMonsterBuffer(builder, offset) {
    builder.finish(offset, "MONS");
  }
  static finishSizePrefixedMonsterBuffer(builder, offset) {
    builder.finish(offset, "MONS", true);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _Monster.getRootAsMonster(new flatbuffers8.ByteBuffer(buffer));
  }
  unpack() {
    return new MonsterT2(this.pos() !== null ? this.pos().unpack() : null, this.mana(), this.hp(), this.name(), this.bb.createScalarList(this.inventory.bind(this), this.inventoryLength()), this.color(), this.testType(), (() => {
      const temp = unionToAny(this.testType(), this.test.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })(), this.bb.createObjList(this.test4.bind(this), this.test4Length()), this.bb.createScalarList(this.testarrayofstring.bind(this), this.testarrayofstringLength()), this.bb.createObjList(this.testarrayoftables.bind(this), this.testarrayoftablesLength()), this.enemy() !== null ? this.enemy().unpack() : null, this.bb.createScalarList(this.testnestedflatbuffer.bind(this), this.testnestedflatbufferLength()), this.testempty() !== null ? this.testempty().unpack() : null, this.testbool(), this.testhashs32Fnv1(), this.testhashu32Fnv1(), this.testhashs64Fnv1(), this.testhashu64Fnv1(), this.testhashs32Fnv1a(), this.testhashu32Fnv1a(), this.testhashs64Fnv1a(), this.testhashu64Fnv1a(), this.bb.createScalarList(this.testarrayofbools.bind(this), this.testarrayofboolsLength()), this.testf(), this.testf2(), this.testf3(), this.bb.createScalarList(this.testarrayofstring2.bind(this), this.testarrayofstring2Length()), this.bb.createObjList(this.testarrayofsortedstruct.bind(this), this.testarrayofsortedstructLength()), this.bb.createScalarList(this.flex.bind(this), this.flexLength()), this.bb.createObjList(this.test5.bind(this), this.test5Length()), this.bb.createScalarList(this.vectorOfLongs.bind(this), this.vectorOfLongsLength()), this.bb.createScalarList(this.vectorOfDoubles.bind(this), this.vectorOfDoublesLength()), this.parentNamespaceTest() !== null ? this.parentNamespaceTest().unpack() : null, this.bb.createObjList(this.vectorOfReferrables.bind(this), this.vectorOfReferrablesLength()), this.singleWeakReference(), this.bb.createScalarList(this.vectorOfWeakReferences.bind(this), this.vectorOfWeakReferencesLength()), this.bb.createObjList(this.vectorOfStrongReferrables.bind(this), this.vectorOfStrongReferrablesLength()), this.coOwningReference(), this.bb.createScalarList(this.vectorOfCoOwningReferences.bind(this), this.vectorOfCoOwningReferencesLength()), this.nonOwningReference(), this.bb.createScalarList(this.vectorOfNonOwningReferences.bind(this), this.vectorOfNonOwningReferencesLength()), this.anyUniqueType(), (() => {
      const temp = unionToAnyUniqueAliases(this.anyUniqueType(), this.anyUnique.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })(), this.anyAmbiguousType(), (() => {
      const temp = unionToAnyAmbiguousAliases(this.anyAmbiguousType(), this.anyAmbiguous.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })(), this.bb.createScalarList(this.vectorOfEnums.bind(this), this.vectorOfEnumsLength()), this.signedEnum(), this.bb.createScalarList(this.testrequirednestedflatbuffer.bind(this), this.testrequirednestedflatbufferLength()), this.bb.createObjList(this.scalarKeySortedTables.bind(this), this.scalarKeySortedTablesLength()), this.nativeInline() !== null ? this.nativeInline().unpack() : null, this.longEnumNonEnumDefault(), this.longEnumNormalDefault(), this.nanDefault(), this.infDefault(), this.positiveInfDefault(), this.infinityDefault(), this.positiveInfinityDefault(), this.negativeInfDefault(), this.negativeInfinityDefault(), this.doubleInfDefault());
  }
  unpackTo(_o) {
    _o.pos = this.pos() !== null ? this.pos().unpack() : null;
    _o.mana = this.mana();
    _o.hp = this.hp();
    _o.name = this.name();
    _o.inventory = this.bb.createScalarList(this.inventory.bind(this), this.inventoryLength());
    _o.color = this.color();
    _o.testType = this.testType();
    _o.test = (() => {
      const temp = unionToAny(this.testType(), this.test.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })();
    _o.test4 = this.bb.createObjList(this.test4.bind(this), this.test4Length());
    _o.testarrayofstring = this.bb.createScalarList(this.testarrayofstring.bind(this), this.testarrayofstringLength());
    _o.testarrayoftables = this.bb.createObjList(this.testarrayoftables.bind(this), this.testarrayoftablesLength());
    _o.enemy = this.enemy() !== null ? this.enemy().unpack() : null;
    _o.testnestedflatbuffer = this.bb.createScalarList(this.testnestedflatbuffer.bind(this), this.testnestedflatbufferLength());
    _o.testempty = this.testempty() !== null ? this.testempty().unpack() : null;
    _o.testbool = this.testbool();
    _o.testhashs32Fnv1 = this.testhashs32Fnv1();
    _o.testhashu32Fnv1 = this.testhashu32Fnv1();
    _o.testhashs64Fnv1 = this.testhashs64Fnv1();
    _o.testhashu64Fnv1 = this.testhashu64Fnv1();
    _o.testhashs32Fnv1a = this.testhashs32Fnv1a();
    _o.testhashu32Fnv1a = this.testhashu32Fnv1a();
    _o.testhashs64Fnv1a = this.testhashs64Fnv1a();
    _o.testhashu64Fnv1a = this.testhashu64Fnv1a();
    _o.testarrayofbools = this.bb.createScalarList(this.testarrayofbools.bind(this), this.testarrayofboolsLength());
    _o.testf = this.testf();
    _o.testf2 = this.testf2();
    _o.testf3 = this.testf3();
    _o.testarrayofstring2 = this.bb.createScalarList(this.testarrayofstring2.bind(this), this.testarrayofstring2Length());
    _o.testarrayofsortedstruct = this.bb.createObjList(this.testarrayofsortedstruct.bind(this), this.testarrayofsortedstructLength());
    _o.flex = this.bb.createScalarList(this.flex.bind(this), this.flexLength());
    _o.test5 = this.bb.createObjList(this.test5.bind(this), this.test5Length());
    _o.vectorOfLongs = this.bb.createScalarList(this.vectorOfLongs.bind(this), this.vectorOfLongsLength());
    _o.vectorOfDoubles = this.bb.createScalarList(this.vectorOfDoubles.bind(this), this.vectorOfDoublesLength());
    _o.parentNamespaceTest = this.parentNamespaceTest() !== null ? this.parentNamespaceTest().unpack() : null;
    _o.vectorOfReferrables = this.bb.createObjList(this.vectorOfReferrables.bind(this), this.vectorOfReferrablesLength());
    _o.singleWeakReference = this.singleWeakReference();
    _o.vectorOfWeakReferences = this.bb.createScalarList(this.vectorOfWeakReferences.bind(this), this.vectorOfWeakReferencesLength());
    _o.vectorOfStrongReferrables = this.bb.createObjList(this.vectorOfStrongReferrables.bind(this), this.vectorOfStrongReferrablesLength());
    _o.coOwningReference = this.coOwningReference();
    _o.vectorOfCoOwningReferences = this.bb.createScalarList(this.vectorOfCoOwningReferences.bind(this), this.vectorOfCoOwningReferencesLength());
    _o.nonOwningReference = this.nonOwningReference();
    _o.vectorOfNonOwningReferences = this.bb.createScalarList(this.vectorOfNonOwningReferences.bind(this), this.vectorOfNonOwningReferencesLength());
    _o.anyUniqueType = this.anyUniqueType();
    _o.anyUnique = (() => {
      const temp = unionToAnyUniqueAliases(this.anyUniqueType(), this.anyUnique.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })();
    _o.anyAmbiguousType = this.anyAmbiguousType();
    _o.anyAmbiguous = (() => {
      const temp = unionToAnyAmbiguousAliases(this.anyAmbiguousType(), this.anyAmbiguous.bind(this));
      if (temp === null) {
        return null;
      }
      return temp.unpack();
    })();
    _o.vectorOfEnums = this.bb.createScalarList(this.vectorOfEnums.bind(this), this.vectorOfEnumsLength());
    _o.signedEnum = this.signedEnum();
    _o.testrequirednestedflatbuffer = this.bb.createScalarList(this.testrequirednestedflatbuffer.bind(this), this.testrequirednestedflatbufferLength());
    _o.scalarKeySortedTables = this.bb.createObjList(this.scalarKeySortedTables.bind(this), this.scalarKeySortedTablesLength());
    _o.nativeInline = this.nativeInline() !== null ? this.nativeInline().unpack() : null;
    _o.longEnumNonEnumDefault = this.longEnumNonEnumDefault();
    _o.longEnumNormalDefault = this.longEnumNormalDefault();
    _o.nanDefault = this.nanDefault();
    _o.infDefault = this.infDefault();
    _o.positiveInfDefault = this.positiveInfDefault();
    _o.infinityDefault = this.infinityDefault();
    _o.positiveInfinityDefault = this.positiveInfinityDefault();
    _o.negativeInfDefault = this.negativeInfDefault();
    _o.negativeInfinityDefault = this.negativeInfinityDefault();
    _o.doubleInfDefault = this.doubleInfDefault();
  }
};
var MonsterT2 = class {
  constructor(pos = null, mana = 150, hp = 100, name = null, inventory = [], color = Color.Blue, testType = Any.NONE, test = null, test4 = [], testarrayofstring = [], testarrayoftables = [], enemy = null, testnestedflatbuffer = [], testempty = null, testbool = false, testhashs32Fnv1 = 0, testhashu32Fnv1 = 0, testhashs64Fnv1 = BigInt("0"), testhashu64Fnv1 = BigInt("0"), testhashs32Fnv1a = 0, testhashu32Fnv1a = 0, testhashs64Fnv1a = BigInt("0"), testhashu64Fnv1a = BigInt("0"), testarrayofbools = [], testf = 3.14159, testf2 = 3, testf3 = 0, testarrayofstring2 = [], testarrayofsortedstruct = [], flex = [], test5 = [], vectorOfLongs = [], vectorOfDoubles = [], parentNamespaceTest = null, vectorOfReferrables = [], singleWeakReference = BigInt("0"), vectorOfWeakReferences = [], vectorOfStrongReferrables = [], coOwningReference = BigInt("0"), vectorOfCoOwningReferences = [], nonOwningReference = BigInt("0"), vectorOfNonOwningReferences = [], anyUniqueType = AnyUniqueAliases.NONE, anyUnique = null, anyAmbiguousType = AnyAmbiguousAliases.NONE, anyAmbiguous = null, vectorOfEnums = [], signedEnum = Race.None, testrequirednestedflatbuffer = [], scalarKeySortedTables = [], nativeInline = null, longEnumNonEnumDefault = BigInt("0"), longEnumNormalDefault = BigInt("2"), nanDefault = NaN, infDefault = Infinity, positiveInfDefault = Infinity, infinityDefault = Infinity, positiveInfinityDefault = Infinity, negativeInfDefault = -Infinity, negativeInfinityDefault = -Infinity, doubleInfDefault = Infinity) {
    this.pos = pos;
    this.mana = mana;
    this.hp = hp;
    this.name = name;
    this.inventory = inventory;
    this.color = color;
    this.testType = testType;
    this.test = test;
    this.test4 = test4;
    this.testarrayofstring = testarrayofstring;
    this.testarrayoftables = testarrayoftables;
    this.enemy = enemy;
    this.testnestedflatbuffer = testnestedflatbuffer;
    this.testempty = testempty;
    this.testbool = testbool;
    this.testhashs32Fnv1 = testhashs32Fnv1;
    this.testhashu32Fnv1 = testhashu32Fnv1;
    this.testhashs64Fnv1 = testhashs64Fnv1;
    this.testhashu64Fnv1 = testhashu64Fnv1;
    this.testhashs32Fnv1a = testhashs32Fnv1a;
    this.testhashu32Fnv1a = testhashu32Fnv1a;
    this.testhashs64Fnv1a = testhashs64Fnv1a;
    this.testhashu64Fnv1a = testhashu64Fnv1a;
    this.testarrayofbools = testarrayofbools;
    this.testf = testf;
    this.testf2 = testf2;
    this.testf3 = testf3;
    this.testarrayofstring2 = testarrayofstring2;
    this.testarrayofsortedstruct = testarrayofsortedstruct;
    this.flex = flex;
    this.test5 = test5;
    this.vectorOfLongs = vectorOfLongs;
    this.vectorOfDoubles = vectorOfDoubles;
    this.parentNamespaceTest = parentNamespaceTest;
    this.vectorOfReferrables = vectorOfReferrables;
    this.singleWeakReference = singleWeakReference;
    this.vectorOfWeakReferences = vectorOfWeakReferences;
    this.vectorOfStrongReferrables = vectorOfStrongReferrables;
    this.coOwningReference = coOwningReference;
    this.vectorOfCoOwningReferences = vectorOfCoOwningReferences;
    this.nonOwningReference = nonOwningReference;
    this.vectorOfNonOwningReferences = vectorOfNonOwningReferences;
    this.anyUniqueType = anyUniqueType;
    this.anyUnique = anyUnique;
    this.anyAmbiguousType = anyAmbiguousType;
    this.anyAmbiguous = anyAmbiguous;
    this.vectorOfEnums = vectorOfEnums;
    this.signedEnum = signedEnum;
    this.testrequirednestedflatbuffer = testrequirednestedflatbuffer;
    this.scalarKeySortedTables = scalarKeySortedTables;
    this.nativeInline = nativeInline;
    this.longEnumNonEnumDefault = longEnumNonEnumDefault;
    this.longEnumNormalDefault = longEnumNormalDefault;
    this.nanDefault = nanDefault;
    this.infDefault = infDefault;
    this.positiveInfDefault = positiveInfDefault;
    this.infinityDefault = infinityDefault;
    this.positiveInfinityDefault = positiveInfinityDefault;
    this.negativeInfDefault = negativeInfDefault;
    this.negativeInfinityDefault = negativeInfinityDefault;
    this.doubleInfDefault = doubleInfDefault;
  }
  pack(builder) {
    const name = this.name !== null ? builder.createString(this.name) : 0;
    const inventory = Monster2.createInventoryVector(builder, this.inventory);
    const test = builder.createObjectOffset(this.test);
    const test4 = builder.createStructOffsetList(this.test4, Monster2.startTest4Vector);
    const testarrayofstring = Monster2.createTestarrayofstringVector(builder, builder.createObjectOffsetList(this.testarrayofstring));
    const testarrayoftables = Monster2.createTestarrayoftablesVector(builder, builder.createObjectOffsetList(this.testarrayoftables));
    const enemy = this.enemy !== null ? this.enemy.pack(builder) : 0;
    const testnestedflatbuffer = Monster2.createTestnestedflatbufferVector(builder, this.testnestedflatbuffer);
    const testempty = this.testempty !== null ? this.testempty.pack(builder) : 0;
    const testarrayofbools = Monster2.createTestarrayofboolsVector(builder, this.testarrayofbools);
    const testarrayofstring2 = Monster2.createTestarrayofstring2Vector(builder, builder.createObjectOffsetList(this.testarrayofstring2));
    const testarrayofsortedstruct = builder.createStructOffsetList(this.testarrayofsortedstruct, Monster2.startTestarrayofsortedstructVector);
    const flex = Monster2.createFlexVector(builder, this.flex);
    const test5 = builder.createStructOffsetList(this.test5, Monster2.startTest5Vector);
    const vectorOfLongs = Monster2.createVectorOfLongsVector(builder, this.vectorOfLongs);
    const vectorOfDoubles = Monster2.createVectorOfDoublesVector(builder, this.vectorOfDoubles);
    const parentNamespaceTest = this.parentNamespaceTest !== null ? this.parentNamespaceTest.pack(builder) : 0;
    const vectorOfReferrables = Monster2.createVectorOfReferrablesVector(builder, builder.createObjectOffsetList(this.vectorOfReferrables));
    const vectorOfWeakReferences = Monster2.createVectorOfWeakReferencesVector(builder, this.vectorOfWeakReferences);
    const vectorOfStrongReferrables = Monster2.createVectorOfStrongReferrablesVector(builder, builder.createObjectOffsetList(this.vectorOfStrongReferrables));
    const vectorOfCoOwningReferences = Monster2.createVectorOfCoOwningReferencesVector(builder, this.vectorOfCoOwningReferences);
    const vectorOfNonOwningReferences = Monster2.createVectorOfNonOwningReferencesVector(builder, this.vectorOfNonOwningReferences);
    const anyUnique = builder.createObjectOffset(this.anyUnique);
    const anyAmbiguous = builder.createObjectOffset(this.anyAmbiguous);
    const vectorOfEnums = Monster2.createVectorOfEnumsVector(builder, this.vectorOfEnums);
    const testrequirednestedflatbuffer = Monster2.createTestrequirednestedflatbufferVector(builder, this.testrequirednestedflatbuffer);
    const scalarKeySortedTables = Monster2.createScalarKeySortedTablesVector(builder, builder.createObjectOffsetList(this.scalarKeySortedTables));
    Monster2.startMonster(builder);
    Monster2.addPos(builder, this.pos !== null ? this.pos.pack(builder) : 0);
    Monster2.addMana(builder, this.mana);
    Monster2.addHp(builder, this.hp);
    Monster2.addName(builder, name);
    Monster2.addInventory(builder, inventory);
    Monster2.addColor(builder, this.color);
    Monster2.addTestType(builder, this.testType);
    Monster2.addTest(builder, test);
    Monster2.addTest4(builder, test4);
    Monster2.addTestarrayofstring(builder, testarrayofstring);
    Monster2.addTestarrayoftables(builder, testarrayoftables);
    Monster2.addEnemy(builder, enemy);
    Monster2.addTestnestedflatbuffer(builder, testnestedflatbuffer);
    Monster2.addTestempty(builder, testempty);
    Monster2.addTestbool(builder, this.testbool);
    Monster2.addTesthashs32Fnv1(builder, this.testhashs32Fnv1);
    Monster2.addTesthashu32Fnv1(builder, this.testhashu32Fnv1);
    Monster2.addTesthashs64Fnv1(builder, this.testhashs64Fnv1);
    Monster2.addTesthashu64Fnv1(builder, this.testhashu64Fnv1);
    Monster2.addTesthashs32Fnv1a(builder, this.testhashs32Fnv1a);
    Monster2.addTesthashu32Fnv1a(builder, this.testhashu32Fnv1a);
    Monster2.addTesthashs64Fnv1a(builder, this.testhashs64Fnv1a);
    Monster2.addTesthashu64Fnv1a(builder, this.testhashu64Fnv1a);
    Monster2.addTestarrayofbools(builder, testarrayofbools);
    Monster2.addTestf(builder, this.testf);
    Monster2.addTestf2(builder, this.testf2);
    Monster2.addTestf3(builder, this.testf3);
    Monster2.addTestarrayofstring2(builder, testarrayofstring2);
    Monster2.addTestarrayofsortedstruct(builder, testarrayofsortedstruct);
    Monster2.addFlex(builder, flex);
    Monster2.addTest5(builder, test5);
    Monster2.addVectorOfLongs(builder, vectorOfLongs);
    Monster2.addVectorOfDoubles(builder, vectorOfDoubles);
    Monster2.addParentNamespaceTest(builder, parentNamespaceTest);
    Monster2.addVectorOfReferrables(builder, vectorOfReferrables);
    Monster2.addSingleWeakReference(builder, this.singleWeakReference);
    Monster2.addVectorOfWeakReferences(builder, vectorOfWeakReferences);
    Monster2.addVectorOfStrongReferrables(builder, vectorOfStrongReferrables);
    Monster2.addCoOwningReference(builder, this.coOwningReference);
    Monster2.addVectorOfCoOwningReferences(builder, vectorOfCoOwningReferences);
    Monster2.addNonOwningReference(builder, this.nonOwningReference);
    Monster2.addVectorOfNonOwningReferences(builder, vectorOfNonOwningReferences);
    Monster2.addAnyUniqueType(builder, this.anyUniqueType);
    Monster2.addAnyUnique(builder, anyUnique);
    Monster2.addAnyAmbiguousType(builder, this.anyAmbiguousType);
    Monster2.addAnyAmbiguous(builder, anyAmbiguous);
    Monster2.addVectorOfEnums(builder, vectorOfEnums);
    Monster2.addSignedEnum(builder, this.signedEnum);
    Monster2.addTestrequirednestedflatbuffer(builder, testrequirednestedflatbuffer);
    Monster2.addScalarKeySortedTables(builder, scalarKeySortedTables);
    Monster2.addNativeInline(builder, this.nativeInline !== null ? this.nativeInline.pack(builder) : 0);
    Monster2.addLongEnumNonEnumDefault(builder, this.longEnumNonEnumDefault);
    Monster2.addLongEnumNormalDefault(builder, this.longEnumNormalDefault);
    Monster2.addNanDefault(builder, this.nanDefault);
    Monster2.addInfDefault(builder, this.infDefault);
    Monster2.addPositiveInfDefault(builder, this.positiveInfDefault);
    Monster2.addInfinityDefault(builder, this.infinityDefault);
    Monster2.addPositiveInfinityDefault(builder, this.positiveInfinityDefault);
    Monster2.addNegativeInfDefault(builder, this.negativeInfDefault);
    Monster2.addNegativeInfinityDefault(builder, this.negativeInfinityDefault);
    Monster2.addDoubleInfDefault(builder, this.doubleInfDefault);
    return Monster2.endMonster(builder);
  }
};

// my-game/example/any.js
var Any;
(function(Any2) {
  Any2[Any2["NONE"] = 0] = "NONE";
  Any2[Any2["Monster"] = 1] = "Monster";
  Any2[Any2["TestSimpleTableWithEnum"] = 2] = "TestSimpleTableWithEnum";
  Any2[Any2["MyGame_Example2_Monster"] = 3] = "MyGame_Example2_Monster";
})(Any || (Any = {}));
function unionToAny(type, accessor) {
  switch (Any[type]) {
    case "NONE":
      return null;
    case "Monster":
      return accessor(new Monster2());
    case "TestSimpleTableWithEnum":
      return accessor(new TestSimpleTableWithEnum());
    case "MyGame_Example2_Monster":
      return accessor(new Monster());
    default:
      return null;
  }
}

// my-game/example/long-enum.js
var LongEnum;
(function(LongEnum2) {
  LongEnum2["LongOne"] = "2";
  LongEnum2["LongTwo"] = "4";
  LongEnum2["LongBig"] = "1099511627776";
})(LongEnum || (LongEnum = {}));

// my-game/example/struct-of-structs.js
var StructOfStructs = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  a(obj) {
    return (obj || new Ability()).__init(this.bb_pos, this.bb);
  }
  b(obj) {
    return (obj || new Test()).__init(this.bb_pos + 8, this.bb);
  }
  c(obj) {
    return (obj || new Ability()).__init(this.bb_pos + 12, this.bb);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.StructOfStructs";
  }
  static sizeOf() {
    return 20;
  }
  static createStructOfStructs(builder, a_id, a_distance, b_a, b_b, c_id, c_distance) {
    builder.prep(4, 20);
    builder.prep(4, 8);
    builder.writeInt32(c_distance);
    builder.writeInt32(c_id);
    builder.prep(2, 4);
    builder.pad(1);
    builder.writeInt8(b_b);
    builder.writeInt16(b_a);
    builder.prep(4, 8);
    builder.writeInt32(a_distance);
    builder.writeInt32(a_id);
    return builder.offset();
  }
  unpack() {
    return new StructOfStructsT(this.a() !== null ? this.a().unpack() : null, this.b() !== null ? this.b().unpack() : null, this.c() !== null ? this.c().unpack() : null);
  }
  unpackTo(_o) {
    _o.a = this.a() !== null ? this.a().unpack() : null;
    _o.b = this.b() !== null ? this.b().unpack() : null;
    _o.c = this.c() !== null ? this.c().unpack() : null;
  }
};
var StructOfStructsT = class {
  constructor(a = null, b = null, c = null) {
    this.a = a;
    this.b = b;
    this.c = c;
  }
  pack(builder) {
    return StructOfStructs.createStructOfStructs(builder, this.a?.id ?? 0, this.a?.distance ?? 0, this.b?.a ?? 0, this.b?.b ?? 0, this.c?.id ?? 0, this.c?.distance ?? 0);
  }
};

// my-game/example/struct-of-structs-of-structs.js
var StructOfStructsOfStructs = class {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  a(obj) {
    return (obj || new StructOfStructs()).__init(this.bb_pos, this.bb);
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.StructOfStructsOfStructs";
  }
  static sizeOf() {
    return 20;
  }
  static createStructOfStructsOfStructs(builder, a_a_id, a_a_distance, a_b_a, a_b_b, a_c_id, a_c_distance) {
    builder.prep(4, 20);
    builder.prep(4, 20);
    builder.prep(4, 8);
    builder.writeInt32(a_c_distance);
    builder.writeInt32(a_c_id);
    builder.prep(2, 4);
    builder.pad(1);
    builder.writeInt8(a_b_b);
    builder.writeInt16(a_b_a);
    builder.prep(4, 8);
    builder.writeInt32(a_a_distance);
    builder.writeInt32(a_a_id);
    return builder.offset();
  }
  unpack() {
    return new StructOfStructsOfStructsT(this.a() !== null ? this.a().unpack() : null);
  }
  unpackTo(_o) {
    _o.a = this.a() !== null ? this.a().unpack() : null;
  }
};
var StructOfStructsOfStructsT = class {
  constructor(a = null) {
    this.a = a;
  }
  pack(builder) {
    return StructOfStructsOfStructs.createStructOfStructsOfStructs(builder, this.a?.a?.id ?? 0, this.a?.a?.distance ?? 0, this.a?.b?.a ?? 0, this.a?.b?.b ?? 0, this.a?.c?.id ?? 0, this.a?.c?.distance ?? 0);
  }
};

// my-game/example/type-aliases.js
var flatbuffers9 = __toESM(require("flatbuffers"), 1);
var TypeAliases = class _TypeAliases {
  constructor() {
    this.bb = null;
    this.bb_pos = 0;
  }
  __init(i, bb) {
    this.bb_pos = i;
    this.bb = bb;
    return this;
  }
  static getRootAsTypeAliases(bb, obj) {
    return (obj || new _TypeAliases()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  static getSizePrefixedRootAsTypeAliases(bb, obj) {
    bb.setPosition(bb.position() + flatbuffers9.SIZE_PREFIX_LENGTH);
    return (obj || new _TypeAliases()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
  }
  i8() {
    const offset = this.bb.__offset(this.bb_pos, 4);
    return offset ? this.bb.readInt8(this.bb_pos + offset) : 0;
  }
  mutate_i8(value) {
    const offset = this.bb.__offset(this.bb_pos, 4);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt8(this.bb_pos + offset, value);
    return true;
  }
  u8() {
    const offset = this.bb.__offset(this.bb_pos, 6);
    return offset ? this.bb.readUint8(this.bb_pos + offset) : 0;
  }
  mutate_u8(value) {
    const offset = this.bb.__offset(this.bb_pos, 6);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint8(this.bb_pos + offset, value);
    return true;
  }
  i16() {
    const offset = this.bb.__offset(this.bb_pos, 8);
    return offset ? this.bb.readInt16(this.bb_pos + offset) : 0;
  }
  mutate_i16(value) {
    const offset = this.bb.__offset(this.bb_pos, 8);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt16(this.bb_pos + offset, value);
    return true;
  }
  u16() {
    const offset = this.bb.__offset(this.bb_pos, 10);
    return offset ? this.bb.readUint16(this.bb_pos + offset) : 0;
  }
  mutate_u16(value) {
    const offset = this.bb.__offset(this.bb_pos, 10);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint16(this.bb_pos + offset, value);
    return true;
  }
  i32() {
    const offset = this.bb.__offset(this.bb_pos, 12);
    return offset ? this.bb.readInt32(this.bb_pos + offset) : 0;
  }
  mutate_i32(value) {
    const offset = this.bb.__offset(this.bb_pos, 12);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt32(this.bb_pos + offset, value);
    return true;
  }
  u32() {
    const offset = this.bb.__offset(this.bb_pos, 14);
    return offset ? this.bb.readUint32(this.bb_pos + offset) : 0;
  }
  mutate_u32(value) {
    const offset = this.bb.__offset(this.bb_pos, 14);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint32(this.bb_pos + offset, value);
    return true;
  }
  i64() {
    const offset = this.bb.__offset(this.bb_pos, 16);
    return offset ? this.bb.readInt64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_i64(value) {
    const offset = this.bb.__offset(this.bb_pos, 16);
    if (offset === 0) {
      return false;
    }
    this.bb.writeInt64(this.bb_pos + offset, value);
    return true;
  }
  u64() {
    const offset = this.bb.__offset(this.bb_pos, 18);
    return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt("0");
  }
  mutate_u64(value) {
    const offset = this.bb.__offset(this.bb_pos, 18);
    if (offset === 0) {
      return false;
    }
    this.bb.writeUint64(this.bb_pos + offset, value);
    return true;
  }
  f32() {
    const offset = this.bb.__offset(this.bb_pos, 20);
    return offset ? this.bb.readFloat32(this.bb_pos + offset) : 0;
  }
  mutate_f32(value) {
    const offset = this.bb.__offset(this.bb_pos, 20);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat32(this.bb_pos + offset, value);
    return true;
  }
  f64() {
    const offset = this.bb.__offset(this.bb_pos, 22);
    return offset ? this.bb.readFloat64(this.bb_pos + offset) : 0;
  }
  mutate_f64(value) {
    const offset = this.bb.__offset(this.bb_pos, 22);
    if (offset === 0) {
      return false;
    }
    this.bb.writeFloat64(this.bb_pos + offset, value);
    return true;
  }
  v8(index) {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.readInt8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
  }
  v8Length() {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  v8Array() {
    const offset = this.bb.__offset(this.bb_pos, 24);
    return offset ? new Int8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  vf64(index) {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? this.bb.readFloat64(this.bb.__vector(this.bb_pos + offset) + index * 8) : 0;
  }
  vf64Length() {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
  }
  vf64Array() {
    const offset = this.bb.__offset(this.bb_pos, 26);
    return offset ? new Float64Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
  }
  static getFullyQualifiedName() {
    return "MyGame.Example.TypeAliases";
  }
  static startTypeAliases(builder) {
    builder.startObject(12);
  }
  static addI8(builder, i8) {
    builder.addFieldInt8(0, i8, 0);
  }
  static addU8(builder, u8) {
    builder.addFieldInt8(1, u8, 0);
  }
  static addI16(builder, i16) {
    builder.addFieldInt16(2, i16, 0);
  }
  static addU16(builder, u16) {
    builder.addFieldInt16(3, u16, 0);
  }
  static addI32(builder, i32) {
    builder.addFieldInt32(4, i32, 0);
  }
  static addU32(builder, u32) {
    builder.addFieldInt32(5, u32, 0);
  }
  static addI64(builder, i64) {
    builder.addFieldInt64(6, i64, BigInt("0"));
  }
  static addU64(builder, u64) {
    builder.addFieldInt64(7, u64, BigInt("0"));
  }
  static addF32(builder, f32) {
    builder.addFieldFloat32(8, f32, 0);
  }
  static addF64(builder, f64) {
    builder.addFieldFloat64(9, f64, 0);
  }
  static addV8(builder, v8Offset) {
    builder.addFieldOffset(10, v8Offset, 0);
  }
  static createV8Vector(builder, data) {
    builder.startVector(1, data.length, 1);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addInt8(data[i]);
    }
    return builder.endVector();
  }
  static startV8Vector(builder, numElems) {
    builder.startVector(1, numElems, 1);
  }
  static addVf64(builder, vf64Offset) {
    builder.addFieldOffset(11, vf64Offset, 0);
  }
  static createVf64Vector(builder, data) {
    builder.startVector(8, data.length, 8);
    for (let i = data.length - 1; i >= 0; i--) {
      builder.addFloat64(data[i]);
    }
    return builder.endVector();
  }
  static startVf64Vector(builder, numElems) {
    builder.startVector(8, numElems, 8);
  }
  static endTypeAliases(builder) {
    const offset = builder.endObject();
    return offset;
  }
  static createTypeAliases(builder, i8, u8, i16, u16, i32, u32, i64, u64, f32, f64, v8Offset, vf64Offset) {
    _TypeAliases.startTypeAliases(builder);
    _TypeAliases.addI8(builder, i8);
    _TypeAliases.addU8(builder, u8);
    _TypeAliases.addI16(builder, i16);
    _TypeAliases.addU16(builder, u16);
    _TypeAliases.addI32(builder, i32);
    _TypeAliases.addU32(builder, u32);
    _TypeAliases.addI64(builder, i64);
    _TypeAliases.addU64(builder, u64);
    _TypeAliases.addF32(builder, f32);
    _TypeAliases.addF64(builder, f64);
    _TypeAliases.addV8(builder, v8Offset);
    _TypeAliases.addVf64(builder, vf64Offset);
    return _TypeAliases.endTypeAliases(builder);
  }
  serialize() {
    return this.bb.bytes();
  }
  static deserialize(buffer) {
    return _TypeAliases.getRootAsTypeAliases(new flatbuffers9.ByteBuffer(buffer));
  }
  unpack() {
    return new TypeAliasesT(this.i8(), this.u8(), this.i16(), this.u16(), this.i32(), this.u32(), this.i64(), this.u64(), this.f32(), this.f64(), this.bb.createScalarList(this.v8.bind(this), this.v8Length()), this.bb.createScalarList(this.vf64.bind(this), this.vf64Length()));
  }
  unpackTo(_o) {
    _o.i8 = this.i8();
    _o.u8 = this.u8();
    _o.i16 = this.i16();
    _o.u16 = this.u16();
    _o.i32 = this.i32();
    _o.u32 = this.u32();
    _o.i64 = this.i64();
    _o.u64 = this.u64();
    _o.f32 = this.f32();
    _o.f64 = this.f64();
    _o.v8 = this.bb.createScalarList(this.v8.bind(this), this.v8Length());
    _o.vf64 = this.bb.createScalarList(this.vf64.bind(this), this.vf64Length());
  }
};
var TypeAliasesT = class {
  constructor(i8 = 0, u8 = 0, i16 = 0, u16 = 0, i32 = 0, u32 = 0, i64 = BigInt("0"), u64 = BigInt("0"), f32 = 0, f64 = 0, v8 = [], vf64 = []) {
    this.i8 = i8;
    this.u8 = u8;
    this.i16 = i16;
    this.u16 = u16;
    this.i32 = i32;
    this.u32 = u32;
    this.i64 = i64;
    this.u64 = u64;
    this.f32 = f32;
    this.f64 = f64;
    this.v8 = v8;
    this.vf64 = vf64;
  }
  pack(builder) {
    const v8 = TypeAliases.createV8Vector(builder, this.v8);
    const vf64 = TypeAliases.createVf64Vector(builder, this.vf64);
    return TypeAliases.createTypeAliases(builder, this.i8, this.u8, this.i16, this.u16, this.i32, this.u32, this.i64, this.u64, this.f32, this.f64, v8, vf64);
  }
};

// my-game/example2.js
var example2_exports = {};
__export(example2_exports, {
  Monster: () => Monster,
  MonsterT: () => MonsterT
});

// my-game/other-name-space.js
var other_name_space_exports = {};
__export(other_name_space_exports, {
  FromInclude: () => FromInclude,
  TableB: () => TableB,
  TableBT: () => TableBT,
  Unused: () => Unused,
  UnusedT: () => UnusedT
});

// my-game/other-name-space/from-include.js
var FromInclude;
(function(FromInclude2) {
  FromInclude2["IncludeVal"] = "0";
})(FromInclude || (FromInclude = {}));

// my-game/other-name-space/unused.js
var Unused = class {
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
    return this.bb.readInt32(this.bb_pos);
  }
  mutate_a(value) {
    this.bb.writeInt32(this.bb_pos + 0, value);
    return true;
  }
  static getFullyQualifiedName() {
    return "MyGame.OtherNameSpace.Unused";
  }
  static sizeOf() {
    return 4;
  }
  static createUnused(builder, a) {
    builder.prep(4, 4);
    builder.writeInt32(a);
    return builder.offset();
  }
  unpack() {
    return new UnusedT(this.a());
  }
  unpackTo(_o) {
    _o.a = this.a();
  }
};
var UnusedT = class {
  constructor(a = 0) {
    this.a = a;
  }
  pack(builder) {
    return Unused.createUnused(builder, this.a);
  }
};
