import 'dart:typed_data';
import 'dart:io' as io;

import 'package:path/path.dart' as path;

import 'package:flat_buffers/flat_buffers.dart';
import 'package:test/test.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';

import './monster_test_my_game.example_generated.dart' as example;
import './monster_test_my_game.example2_generated.dart' as example2;
import './list_of_enums_generated.dart' as example3;
import './bool_structs_generated.dart' as example4;

main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(BuilderTest);
    defineReflectiveTests(ObjectAPITest);
    defineReflectiveTests(CheckOtherLangaugesData);
    defineReflectiveTests(GeneratorTest);
    defineReflectiveTests(ListOfEnumsTest);
  });
}

int indexToField(int index) {
  return (1 + 1 + index) * 2;
}

@reflectiveTest
class CheckOtherLangaugesData {
  test_cppData() async {
    List<int> data = await io.File(path.join(
      path.context.current,
      'test',
      'monsterdata_test.mon',
    )).readAsBytes();
    example.Monster mon = example.Monster(data);
    expect(mon.hp, 80);
    expect(mon.mana, 150);
    expect(mon.name, 'MyMonster');
    expect(mon.pos!.x, 1.0);
    expect(mon.pos!.y, 2.0);
    expect(mon.pos!.z, 3.0);
    expect(mon.pos!.test1, 3.0);
    expect(mon.pos!.test2.value, 2.0);
    expect(mon.pos!.test3.a, 5);
    expect(mon.pos!.test3.b, 6);
    expect(mon.testType!.value, example.AnyTypeId.Monster.value);
    expect(mon.test is example.Monster, true);
    final monster2 = mon.test as example.Monster;
    expect(monster2.name, "Fred");

    expect(mon.inventory!.length, 5);
    expect(mon.inventory!.reduce((cur, next) => cur + next), 10);
    final test4 = mon.test4!;
    expect(test4.length, 2);
    expect(test4[0].a + test4[0].b + test4[1].a + test4[1].b, 100);
    expect(mon.testarrayofstring!.length, 2);
    expect(mon.testarrayofstring![0], "test1");
    expect(mon.testarrayofstring![1], "test2");

    // this will fail if accessing any field fails.
    expect(
        mon.toString(),
        'Monster{'
        'pos: Vec3{x: 1.0, y: 2.0, z: 3.0, test1: 3.0, test2: Color{value: 2}, test3: Test{a: 5, b: 6}}, '
        'mana: 150, hp: 80, name: MyMonster, inventory: [0, 1, 2, 3, 4], '
        'color: Color{value: 8}, testType: AnyTypeId{value: 1}, '
        'test: Monster{pos: null, mana: 150, hp: 100, name: Fred, '
        'inventory: null, color: Color{value: 8}, testType: null, '
        'test: null, test4: null, testarrayofstring: null, '
        'testarrayoftables: null, enemy: null, testnestedflatbuffer: null, '
        'testempty: null, testbool: false, testhashs32Fnv1: 0, '
        'testhashu32Fnv1: 0, testhashs64Fnv1: 0, testhashu64Fnv1: 0, '
        'testhashs32Fnv1a: 0, testhashu32Fnv1a: 0, testhashs64Fnv1a: 0, '
        'testhashu64Fnv1a: 0, testarrayofbools: null, testf: 3.14159, '
        'testf2: 3.0, testf3: 0.0, testarrayofstring2: null, '
        'testarrayofsortedstruct: null, flex: null, test5: null, '
        'vectorOfLongs: null, vectorOfDoubles: null, parentNamespaceTest: null, '
        'vectorOfReferrables: null, singleWeakReference: 0, '
        'vectorOfWeakReferences: null, vectorOfStrongReferrables: null, '
        'coOwningReference: 0, vectorOfCoOwningReferences: null, '
        'nonOwningReference: 0, vectorOfNonOwningReferences: null, '
        'anyUniqueType: null, anyUnique: null, anyAmbiguousType: null, '
        'anyAmbiguous: null, vectorOfEnums: null, signedEnum: Race{value: -1}, '
        'testrequirednestedflatbuffer: null, scalarKeySortedTables: null, '
        'nativeInline: null, '
        'longEnumNonEnumDefault: LongEnum{value: 0}, '
        'longEnumNormalDefault: LongEnum{value: 2}, nanDefault: NaN, '
        'infDefault: Infinity, positiveInfDefault: Infinity, infinityDefault: '
        'Infinity, positiveInfinityDefault: Infinity, negativeInfDefault: '
        '-Infinity, negativeInfinityDefault: -Infinity, doubleInfDefault: Infinity}, '
        'test4: [Test{a: 10, b: 20}, Test{a: 30, b: 40}], '
        'testarrayofstring: [test1, test2], testarrayoftables: null, '
        'enemy: Monster{pos: null, mana: 150, hp: 100, name: Fred, '
        'inventory: null, color: Color{value: 8}, testType: null, '
        'test: null, test4: null, testarrayofstring: null, '
        'testarrayoftables: null, enemy: null, testnestedflatbuffer: null, '
        'testempty: null, testbool: false, testhashs32Fnv1: 0, '
        'testhashu32Fnv1: 0, testhashs64Fnv1: 0, testhashu64Fnv1: 0, '
        'testhashs32Fnv1a: 0, testhashu32Fnv1a: 0, testhashs64Fnv1a: 0, '
        'testhashu64Fnv1a: 0, testarrayofbools: null, testf: 3.14159, '
        'testf2: 3.0, testf3: 0.0, testarrayofstring2: null, '
        'testarrayofsortedstruct: null, flex: null, test5: null, '
        'vectorOfLongs: null, vectorOfDoubles: null, parentNamespaceTest: null, '
        'vectorOfReferrables: null, singleWeakReference: 0, '
        'vectorOfWeakReferences: null, vectorOfStrongReferrables: null, '
        'coOwningReference: 0, vectorOfCoOwningReferences: null, '
        'nonOwningReference: 0, vectorOfNonOwningReferences: null, '
        'anyUniqueType: null, anyUnique: null, anyAmbiguousType: null, '
        'anyAmbiguous: null, vectorOfEnums: null, signedEnum: Race{value: -1}, '
        'testrequirednestedflatbuffer: null, scalarKeySortedTables: null, '
        'nativeInline: null, '
        'longEnumNonEnumDefault: LongEnum{value: 0}, '
        'longEnumNormalDefault: LongEnum{value: 2}, nanDefault: NaN, '
        'infDefault: Infinity, positiveInfDefault: Infinity, infinityDefault: '
        'Infinity, positiveInfinityDefault: Infinity, negativeInfDefault: '
        '-Infinity, negativeInfinityDefault: -Infinity, doubleInfDefault: Infinity}, '
        'testnestedflatbuffer: null, testempty: null, testbool: true, '
        'testhashs32Fnv1: -579221183, testhashu32Fnv1: 3715746113, '
        'testhashs64Fnv1: 7930699090847568257, '
        'testhashu64Fnv1: 7930699090847568257, '
        'testhashs32Fnv1a: -1904106383, testhashu32Fnv1a: 2390860913, '
        'testhashs64Fnv1a: 4898026182817603057, '
        'testhashu64Fnv1a: 4898026182817603057, '
        'testarrayofbools: [true, false, true], testf: 3.14159, testf2: 3.0, '
        'testf3: 0.0, testarrayofstring2: null, testarrayofsortedstruct: ['
        'Ability{id: 0, distance: 45}, Ability{id: 1, distance: 21}, '
        'Ability{id: 5, distance: 12}], '
        'flex: null, test5: [Test{a: 10, b: 20}, Test{a: 30, b: 40}], '
        'vectorOfLongs: [1, 100, 10000, 1000000, 100000000], '
        'vectorOfDoubles: [-1.7976931348623157e+308, 0.0, 1.7976931348623157e+308], '
        'parentNamespaceTest: null, vectorOfReferrables: null, '
        'singleWeakReference: 0, vectorOfWeakReferences: null, '
        'vectorOfStrongReferrables: null, coOwningReference: 0, '
        'vectorOfCoOwningReferences: null, nonOwningReference: 0, '
        'vectorOfNonOwningReferences: null, '
        'anyUniqueType: null, anyUnique: null, '
        'anyAmbiguousType: null, '
        'anyAmbiguous: null, vectorOfEnums: null, signedEnum: Race{value: -1}, '
        'testrequirednestedflatbuffer: null, scalarKeySortedTables: [Stat{id: '
        'miss, val: 0, count: 0}, Stat{id: hit, val: 10, count: 1}], '
        'nativeInline: Test{a: 1, b: 2}, '
        'longEnumNonEnumDefault: LongEnum{value: 0}, '
        'longEnumNormalDefault: LongEnum{value: 2}, nanDefault: NaN, '
        'infDefault: Infinity, positiveInfDefault: Infinity, infinityDefault: '
        'Infinity, positiveInfinityDefault: Infinity, negativeInfDefault: '
        '-Infinity, negativeInfinityDefault: -Infinity, doubleInfDefault: Infinity}');
  }
}

/// Test a custom, fixed-memory allocator (no actual allocations performed)
class CustomAllocator extends Allocator {
  final _memory = ByteData(10 * 1024);
  int _used = 0;

  Uint8List buffer(int size) => _memory.buffer.asUint8List(_used - size, size);

  @override
  ByteData allocate(int size) {
    if (size > _memory.lengthInBytes) {
      throw UnsupportedError('Trying to allocate too much');
    }
    _used = size;
    return ByteData.sublistView(_memory, 0, size);
  }

  @override
  void deallocate(ByteData _) {}
}

@reflectiveTest
class BuilderTest {
  void test_monsterBuilder([Builder? builder]) {
    final fbBuilder = builder ?? Builder();
    final str = fbBuilder.writeString('MyMonster');

    fbBuilder.writeString('test1');
    fbBuilder.writeString('test2', asciiOptimization: true);
    final testArrayOfString = fbBuilder.endStructVector(2);

    final fred = fbBuilder.writeString('Fred');

    final List<int> treasure = [0, 1, 2, 3, 4];
    final inventory = fbBuilder.writeListUint8(treasure);

    final monBuilder = example.MonsterBuilder(fbBuilder)
      ..begin()
      ..addNameOffset(fred);
    final mon2 = monBuilder.finish();

    final testBuilder = example.TestBuilder(fbBuilder);
    testBuilder.finish(10, 20);
    testBuilder.finish(30, 40);
    final test4 = fbBuilder.endStructVector(2);

    monBuilder
      ..begin()
      ..addPos(
        example.Vec3Builder(fbBuilder).finish(
          1.0,
          2.0,
          3.0,
          3.0,
          example.Color.Green,
          () => testBuilder.finish(5, 6),
        ),
      )
      ..addHp(80)
      ..addNameOffset(str)
      ..addInventoryOffset(inventory)
      ..addTestType(example.AnyTypeId.Monster)
      ..addTestOffset(mon2)
      ..addTest4Offset(test4)
      ..addTestarrayofstringOffset(testArrayOfString);
    final mon = monBuilder.finish();
    fbBuilder.finish(mon);
  }

  void test_error_addInt32_withoutStartTable([Builder? builder]) {
    builder ??= Builder();
    expect(() {
      builder!.addInt32(0, 0);
    }, throwsA(isA<AssertionError>()));
  }

  void test_error_addOffset_withoutStartTable() {
    Builder builder = Builder();
    expect(() {
      builder.addOffset(0, 0);
    }, throwsA(isA<AssertionError>()));
  }

  void test_error_endTable_withoutStartTable() {
    Builder builder = Builder();
    expect(() {
      builder.endTable();
    }, throwsA(isA<AssertionError>()));
  }

  void test_error_startTable_duringTable() {
    Builder builder = Builder();
    builder.startTable(0);
    expect(() {
      builder.startTable(0);
    }, throwsA(isA<AssertionError>()));
  }

  void test_error_writeString_duringTable() {
    Builder builder = Builder();
    builder.startTable(1);
    expect(() {
      builder.writeString('12345');
    }, throwsA(isA<AssertionError>()));
  }

  void test_file_identifier() {
    Uint8List byteList;
    {
      Builder builder = Builder(initialSize: 0);
      builder.startTable(0);
      int offset = builder.endTable();
      builder.finish(offset, 'Az~ÿ');
      byteList = builder.buffer;
    }
    // Convert byteList to a ByteData so that we can read data from it.
    ByteData byteData = byteList.buffer.asByteData(byteList.offsetInBytes);
    // First 4 bytes are an offset to the table data.
    int tableDataLoc = byteData.getUint32(0, Endian.little);
    // Next 4 bytes are the file identifier.
    expect(byteData.getUint8(4), 65); // 'a'
    expect(byteData.getUint8(5), 122); // 'z'
    expect(byteData.getUint8(6), 126); // '~'
    expect(byteData.getUint8(7), 255); // 'ÿ'
    // First 4 bytes of the table data are a backwards offset to the vtable.
    int vTableLoc =
        tableDataLoc - byteData.getInt32(tableDataLoc, Endian.little);
    // First 2 bytes of the vtable are the size of the vtable in bytes, which
    // should be 4.
    expect(byteData.getUint16(vTableLoc, Endian.little), 4);
    // Next 2 bytes are the size of the object in bytes (including the vtable
    // pointer), which should be 4.
    expect(byteData.getUint16(vTableLoc + 2, Endian.little), 4);
  }

  void test_low() {
    final allocator = CustomAllocator();
    final builder = Builder(initialSize: 0, allocator: allocator);

    builder.putUint8(1);
    expect(allocator.buffer(builder.size()), [1]);

    builder.putUint32(2);
    expect(allocator.buffer(builder.size()), [2, 0, 0, 0, 0, 0, 0, 1]);

    builder.putUint8(3);
    expect(
        allocator.buffer(builder.size()), [0, 0, 0, 3, 2, 0, 0, 0, 0, 0, 0, 1]);

    builder.putUint8(4);
    expect(
        allocator.buffer(builder.size()), [0, 0, 4, 3, 2, 0, 0, 0, 0, 0, 0, 1]);

    builder.putUint8(5);
    expect(
        allocator.buffer(builder.size()), [0, 5, 4, 3, 2, 0, 0, 0, 0, 0, 0, 1]);

    builder.putUint32(6);
    expect(allocator.buffer(builder.size()),
        [6, 0, 0, 0, 0, 5, 4, 3, 2, 0, 0, 0, 0, 0, 0, 1]);
  }

  void test_table_default() {
    List<int> byteList;
    {
      final builder = Builder(initialSize: 0, allocator: CustomAllocator());
      builder.startTable(2);
      builder.addInt32(0, 10, 10);
      builder.addInt32(1, 20, 10);
      int offset = builder.endTable();
      builder.finish(offset);
      byteList = builder.buffer;
      expect(builder.size(), byteList.length);
    }
    // read and verify
    BufferContext buffer = BufferContext.fromBytes(byteList);
    int objectOffset = buffer.derefObject(0);
    // was not written, so uses the new default value
    expect(
        const Int32Reader()
            .vTableGet(buffer, objectOffset, indexToField(0), 15),
        15);
    // has the written value
    expect(
        const Int32Reader()
            .vTableGet(buffer, objectOffset, indexToField(1), 15),
        20);
  }

  void test_table_format([Builder? builder]) {
    Uint8List byteList;
    {
      builder ??= Builder(initialSize: 0);
      builder.startTable(3);
      builder.addInt32(0, 10);
      builder.addInt32(1, 20);
      builder.addInt32(2, 30);
      builder.finish(builder.endTable());
      byteList = builder.buffer;
    }
    // Convert byteList to a ByteData so that we can read data from it.
    ByteData byteData = byteList.buffer.asByteData(byteList.offsetInBytes);
    // First 4 bytes are an offset to the table data.
    int tableDataLoc = byteData.getUint32(0, Endian.little);
    // First 4 bytes of the table data are a backwards offset to the vtable.
    int vTableLoc =
        tableDataLoc - byteData.getInt32(tableDataLoc, Endian.little);
    // First 2 bytes of the vtable are the size of the vtable in bytes, which
    // should be 10.
    expect(byteData.getUint16(vTableLoc, Endian.little), 10);
    // Next 2 bytes are the size of the object in bytes (including the vtable
    // pointer), which should be 16.
    expect(byteData.getUint16(vTableLoc + 2, Endian.little), 16);
    // Remaining 6 bytes are the offsets within the object where the ints are
    // located.
    for (int i = 0; i < 3; i++) {
      int offset = byteData.getUint16(vTableLoc + 4 + 2 * i, Endian.little);
      expect(
          byteData.getInt32(tableDataLoc + offset, Endian.little), 10 + 10 * i);
    }
  }

  void test_table_string() {
    String latinString = 'test';
    String unicodeString = 'Проба пера';
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int? latinStringOffset =
          builder.writeString(latinString, asciiOptimization: true);
      int? unicodeStringOffset =
          builder.writeString(unicodeString, asciiOptimization: true);
      builder.startTable(2);
      builder.addOffset(0, latinStringOffset);
      builder.addOffset(1, unicodeStringOffset);
      int offset = builder.endTable();
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    int objectOffset = buf.derefObject(0);
    expect(
        const StringReader()
            .vTableGetNullable(buf, objectOffset, indexToField(0)),
        latinString);
    expect(
        const StringReader(asciiOptimization: true)
            .vTableGetNullable(buf, objectOffset, indexToField(1)),
        unicodeString);
  }

  void test_table_types([Builder? builder]) {
    List<int> byteList;
    {
      builder ??= Builder(initialSize: 0);
      int? stringOffset = builder.writeString('12345');
      builder.startTable(7);
      builder.addBool(0, true);
      builder.addInt8(1, 10);
      builder.addInt32(2, 20);
      builder.addOffset(3, stringOffset);
      builder.addInt32(4, 40);
      builder.addUint32(5, 0x9ABCDEF0);
      builder.addUint8(6, 0x9A);
      int offset = builder.endTable();
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    int objectOffset = buf.derefObject(0);
    expect(
        const BoolReader()
            .vTableGetNullable(buf, objectOffset, indexToField(0)),
        true);
    expect(
        const Int8Reader()
            .vTableGetNullable(buf, objectOffset, indexToField(1)),
        10);
    expect(
        const Int32Reader()
            .vTableGetNullable(buf, objectOffset, indexToField(2)),
        20);
    expect(
        const StringReader()
            .vTableGetNullable(buf, objectOffset, indexToField(3)),
        '12345');
    expect(
        const Int32Reader()
            .vTableGetNullable(buf, objectOffset, indexToField(4)),
        40);
    expect(
        const Uint32Reader()
            .vTableGetNullable(buf, objectOffset, indexToField(5)),
        0x9ABCDEF0);
    expect(
        const Uint8Reader()
            .vTableGetNullable(buf, objectOffset, indexToField(6)),
        0x9A);
  }

  void test_writeList_of_Uint32() {
    List<int> values = <int>[10, 100, 12345, 0x9abcdef0];
    // write
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListUint32(values);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<int> items = const Uint32ListReader().read(buf, 0);
    expect(items, hasLength(4));
    expect(items, orderedEquals(values));
  }

  void test_writeList_ofBool() {
    void verifyListBooleans(int len, List<int> trueBits) {
      // write
      List<int> byteList;
      {
        Builder builder = Builder(initialSize: 0);
        List<bool> values = List<bool>.filled(len, false);
        for (int bit in trueBits) {
          values[bit] = true;
        }
        int offset = builder.writeListBool(values);
        builder.finish(offset);
        byteList = builder.buffer;
      }
      // read and verify
      BufferContext buf = BufferContext.fromBytes(byteList);
      List<bool> items = const BoolListReader().read(buf, 0);
      expect(items, hasLength(len));
      for (int i = 0; i < items.length; i++) {
        expect(items[i], trueBits.contains(i), reason: 'bit $i of $len');
      }
    }

    verifyListBooleans(0, <int>[]);
    verifyListBooleans(1, <int>[]);
    verifyListBooleans(1, <int>[0]);
    verifyListBooleans(31, <int>[0, 1]);
    verifyListBooleans(31, <int>[1, 2, 24, 25, 30]);
    verifyListBooleans(31, <int>[0, 30]);
    verifyListBooleans(32, <int>[1, 2, 24, 25, 31]);
    verifyListBooleans(33, <int>[1, 2, 24, 25, 32]);
    verifyListBooleans(33, <int>[1, 2, 24, 25, 31, 32]);
    verifyListBooleans(63, <int>[]);
    verifyListBooleans(63, <int>[0, 1, 2, 61, 62]);
    verifyListBooleans(63, List<int>.generate(63, (i) => i));
    verifyListBooleans(64, <int>[]);
    verifyListBooleans(64, <int>[0, 1, 2, 61, 62, 63]);
    verifyListBooleans(64, <int>[1, 2, 62]);
    verifyListBooleans(64, <int>[0, 1, 2, 63]);
    verifyListBooleans(64, List<int>.generate(64, (i) => i));
    verifyListBooleans(100, <int>[0, 3, 30, 60, 90, 99]);
  }

  void test_writeList_ofInt32() {
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListInt32(<int>[1, 2, 3, 4, 5]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<int> items = const ListReader<int>(Int32Reader()).read(buf, 0);
    expect(items, hasLength(5));
    expect(items, orderedEquals(<int>[1, 2, 3, 4, 5]));
  }

  void test_writeList_ofFloat64() {
    List<double> values = <double>[-1.234567, 3.4E+9, -5.6E-13, 7.8, 12.13];
    // write
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListFloat64(values);
      builder.finish(offset);
      byteList = builder.buffer;
    }

    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<double> items = const Float64ListReader().read(buf, 0);

    expect(items, hasLength(values.length));
    for (int i = 0; i < values.length; i++) {
      expect(values[i], closeTo(items[i], .001));
    }
  }

  void test_writeList_ofFloat32() {
    List<double> values = [1.0, 2.23, -3.213, 7.8, 12.13];
    // write
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListFloat32(values);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<double> items = const Float32ListReader().read(buf, 0);
    expect(items, hasLength(5));
    for (int i = 0; i < values.length; i++) {
      expect(values[i], closeTo(items[i], .001));
    }
  }

  void test_writeList_ofObjects([Builder? builder]) {
    List<int> byteList;
    {
      builder ??= Builder(initialSize: 0);
      // write the object #1
      int object1;
      {
        builder.startTable(2);
        builder.addInt32(0, 10);
        builder.addInt32(1, 20);
        object1 = builder.endTable();
      }
      // write the object #1
      int object2;
      {
        builder.startTable(2);
        builder.addInt32(0, 100);
        builder.addInt32(1, 200);
        object2 = builder.endTable();
      }
      // write the list
      int offset = builder.writeList([object1, object2]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<TestPointImpl> items =
        const ListReader<TestPointImpl>(TestPointReader()).read(buf, 0);
    expect(items, hasLength(2));
    expect(items[0].x, 10);
    expect(items[0].y, 20);
    expect(items[1].x, 100);
    expect(items[1].y, 200);
  }

  void test_writeList_ofStrings_asRoot() {
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int? str1 = builder.writeString('12345');
      int? str2 = builder.writeString('ABC');
      int offset = builder.writeList([str1, str2]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<String> items = const ListReader<String>(StringReader()).read(buf, 0);
    expect(items, hasLength(2));
    expect(items, contains('12345'));
    expect(items, contains('ABC'));
  }

  void test_writeList_ofStrings_inObject([Builder? builder]) {
    List<int> byteList;
    {
      builder ??= Builder(initialSize: 0);
      int listOffset = builder.writeList(
          [builder.writeString('12345'), builder.writeString('ABC')]);
      builder.startTable(1);
      builder.addOffset(0, listOffset);
      int offset = builder.endTable();
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    StringListWrapperImpl reader = StringListWrapperReader().read(buf, 0);
    List<String>? items = reader.items;
    expect(items, hasLength(2));
    expect(items, contains('12345'));
    expect(items, contains('ABC'));
  }

  void test_writeList_ofUint32() {
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListUint32(<int>[1, 2, 0x9ABCDEF0]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<int> items = const Uint32ListReader().read(buf, 0);
    expect(items, hasLength(3));
    expect(items, orderedEquals(<int>[1, 2, 0x9ABCDEF0]));
  }

  void test_writeList_ofUint16() {
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListUint16(<int>[1, 2, 60000]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    List<int> items = const Uint16ListReader().read(buf, 0);
    expect(items, hasLength(3));
    expect(items, orderedEquals(<int>[1, 2, 60000]));
  }

  void test_writeList_ofUint8() {
    List<int> byteList;
    {
      Builder builder = Builder(initialSize: 0);
      int offset = builder.writeListUint8(<int>[1, 2, 3, 4, 0x9A, 0xFA]);
      builder.finish(offset);
      byteList = builder.buffer;
    }
    // read and verify
    BufferContext buf = BufferContext.fromBytes(byteList);
    const buffOffset = 8; // 32-bit offset to the list, + 32-bit length
    for (final lazy in [true, false]) {
      List<int> items = Uint8ListReader(lazy: lazy).read(buf, 0);
      expect(items, hasLength(6));
      expect(items, orderedEquals(<int>[1, 2, 3, 4, 0x9A, 0xFA]));

      // overwrite the buffer to verify the laziness
      buf.buffer.setUint8(buffOffset + 1, 99);
      expect(items, orderedEquals(<int>[1, lazy ? 99 : 2, 3, 4, 0x9A, 0xFA]));

      // restore the previous value for the next loop
      buf.buffer.setUint8(buffOffset + 1, 2);
    }
  }

  void test_reset() {
    // We'll run a selection of tests , reusing the builder between them.
    final testCases = <void Function(Builder?)>[
      test_monsterBuilder,
      test_error_addInt32_withoutStartTable,
      test_table_format,
      test_table_types,
      test_writeList_ofObjects,
      test_writeList_ofStrings_inObject
    ];

    // Execute all test cases in all permutations of their order.
    // To do that, we generate permutations of test case indexes.
    final testCasesPermutations =
        _permutationsOf(List.generate(testCases.length, (index) => index));
    expect(testCasesPermutations.length, _factorial(testCases.length));

    for (var indexes in testCasesPermutations) {
      // print the order so failures are reproducible
      printOnFailure('Running reset() test cases in order: $indexes');

      Builder? builder;
      for (var index in indexes) {
        if (builder == null) {
          // Initial size small enough so at least one test case increases it.
          // On the other hand, it's large enough so that some test cases don't.
          builder = Builder(initialSize: 32);
        } else {
          builder.reset();
        }
        testCases[index](builder);
      }
    }
  }

  // Generate permutations of the given list
  List<List<T>> _permutationsOf<T>(List<T> source) {
    final result = <List<T>>[];

    void permutate(List<T> items, int startAt) {
      for (var i = startAt; i < items.length; i++) {
        List<T> permutation = items.toList(growable: false);
        permutation[i] = items[startAt];
        permutation[startAt] = items[i];

        // add the current list upon reaching the end
        if (startAt == items.length - 1) {
          result.add(items);
        } else {
          permutate(permutation, startAt + 1);
        }
      }
    }

    permutate(source, 0);
    return result;
  }

  // a very simple implementation of n!
  int _factorial(int n) {
    var result = 1;
    for (var i = 2; i <= n; i++) {
      result *= i;
    }
    return result;
  }
}

@reflectiveTest
class ObjectAPITest {
  void test_tableStat() {
    final object1 = example.StatT(count: 3, id: "foo", val: 4);
    final fbb = Builder();
    fbb.finish(object1.pack(fbb));
    final object2 = example.Stat(fbb.buffer).unpack();
    expect(object2.count, object1.count);
    expect(object2.id, object1.id);
    expect(object2.val, object1.val);
    expect(object2.toString(), object1.toString());
  }

  void test_tableMonster() {
    final monster = example.MonsterT()
      ..pos = example.Vec3T(
          x: 1,
          y: 2,
          z: 3,
          test1: 4.0,
          test2: example.Color.Red,
          test3: example.TestT(a: 1, b: 2))
      ..mana = 2
      ..name = 'Monstrous'
      ..inventory = [24, 42]
      ..color = example.Color.Green
      // TODO be smarter for unions and automatically set the `type` field?
      ..testType = example.AnyTypeId.MyGame_Example2_Monster
      ..test = example2.MonsterT()
      ..test4 = [example.TestT(a: 3, b: 4), example.TestT(a: 5, b: 6)]
      ..testarrayofstring = ["foo", "bar"]
      ..testarrayoftables = [example.MonsterT(name: 'Oof')]
      ..enemy = example.MonsterT(name: 'Enemy')
      ..testarrayofbools = [false, true, false]
      ..testf = 42.24
      ..testarrayofsortedstruct = [
        example.AbilityT(id: 1, distance: 5),
        example.AbilityT(id: 3, distance: 7)
      ]
      ..vectorOfLongs = [5, 6, 7]
      ..vectorOfDoubles = [8.9, 9.0, 10.1, 11.2]
      ..anyAmbiguousType = example.AnyAmbiguousAliasesTypeId.M2
      ..anyAmbiguous = null
      ..vectorOfEnums = [example.Color.Blue, example.Color.Green]
      ..signedEnum = example.Race.None;

    final fbBuilder = Builder();
    final offset = monster.pack(fbBuilder);
    expect(offset, isNonZero);
    fbBuilder.finish(offset);
    final data = fbBuilder.buffer;

    // TODO currently broken because of struct builder issue, see #6688
    // final monster2 = example.Monster(data); // Monster (reader)
    // expect(
    //     // map Monster => MonsterT, Vec3 => Vec3T, ...
    //     monster2.toString().replaceAllMapped(
    //         RegExp('([a-zA-z0-9]+){'), (match) => match.group(1) + 'T{'),
    //     monster.toString());
    //
    // final monster3 = monster2.unpack(); // MonsterT
    // expect(monster3.toString(), monster.toString());
  }

  void test_Lists() {
    // Ensure unpack() reads lists eagerly by reusing the same builder and
    // overwriting data. Why: because standard reader reads lists lazily...
    final fbb = Builder();

    final object1 = example.TypeAliasesT(v8: [1, 2, 3], vf64: [5, 6]);
    fbb.finish(object1.pack(fbb));
    final object1Read = example.TypeAliases(fbb.buffer).unpack();

    // overwrite the original buffer by writing to the same builder
    fbb.reset();
    final object2 = example.TypeAliasesT(v8: [7, 8, 9], vf64: [10, 11]);
    fbb.finish(object2.pack(fbb));
    final object2Read = example.TypeAliases(fbb.buffer).unpack();

    // this is fine even with lazy lists:
    expect(object2.toString(), object2Read.toString());

    // this fails with lazy lists:
    expect(object1.toString(), object1Read.toString());

    // empty list must be serialized as such (were stored NULL before v2.0)
    fbb.reset();
    final object3 = example.TypeAliasesT(v8: [], vf64: null);
    fbb.finish(object3.pack(fbb));
    final object3Read = example.TypeAliases(fbb.buffer).unpack();
    expect(object3.toString(), object3Read.toString());
  }
}

class StringListWrapperImpl {
  final BufferContext bp;
  final int offset;

  StringListWrapperImpl(this.bp, this.offset);

  List<String>? get items => const ListReader<String>(StringReader())
      .vTableGetNullable(bp, offset, indexToField(0));
}

class StringListWrapperReader extends TableReader<StringListWrapperImpl> {
  const StringListWrapperReader();

  @override
  StringListWrapperImpl createObject(BufferContext object, int offset) {
    return StringListWrapperImpl(object, offset);
  }
}

class TestPointImpl {
  final BufferContext bp;
  final int offset;

  TestPointImpl(this.bp, this.offset);

  int get x => const Int32Reader().vTableGet(bp, offset, indexToField(0), 0);

  int get y => const Int32Reader().vTableGet(bp, offset, indexToField(1), 0);
}

class TestPointReader extends TableReader<TestPointImpl> {
  const TestPointReader();

  @override
  TestPointImpl createObject(BufferContext object, int offset) {
    return TestPointImpl(object, offset);
  }
}

@reflectiveTest
class GeneratorTest {
  void test_constantEnumValues() async {
    expect(example.Color.values, same(example.Color.values));
    expect(example.Race.values, same(example.Race.values));
    expect(example.AnyTypeId.values, same(example.AnyTypeId.values));
    expect(example.AnyUniqueAliasesTypeId.values,
        same(example.AnyUniqueAliasesTypeId.values));
    expect(example.AnyAmbiguousAliasesTypeId.values,
        same(example.AnyAmbiguousAliasesTypeId.values));
  }
}

// See #6869
@reflectiveTest
class ListOfEnumsTest {
  void test_listOfEnums() async {
    var mytable = example3.MyTableObjectBuilder(options: [
      example3.OptionsEnum.A,
      example3.OptionsEnum.B,
      example3.OptionsEnum.C
    ]);
    var bytes = mytable.toBytes();
    var mytable_read = example3.MyTable(bytes);
    expect(mytable_read.options![0].value, example3.OptionsEnum.A.value);
    expect(mytable_read.options![1].value, example3.OptionsEnum.B.value);
    expect(mytable_read.options![2].value, example3.OptionsEnum.C.value);
  }
}

@reflectiveTest
class BoolInStructTest {
  void test_boolInStruct() async {
    var mystruct = example4.FooObjectBuilder(
        myFoo: example4.FooPropertiesObjectBuilder(a: true, b: false));
    var bytes = mystruct.toBytes();
    var mystruct_read = example4.Foo(bytes);
    expect(mystruct_read.myFoo!.a, true);
    expect(mystruct_read.myFoo!.b, false);
  }
}
