import 'dart:typed_data';

import 'package:flat_buffers/flex_buffers.dart' show Reference, Builder;
import 'package:test/test.dart';

void main() {
  test('is null', () {
    expect(Reference.fromBuffer(b([0, 0, 1])).isNull, isTrue);
  });

  test('bool value', () {
    expect(Reference.fromBuffer(b([1, 104, 1])).boolValue, isTrue);
    expect(Reference.fromBuffer(b([0, 104, 1])).boolValue, isFalse);
  });
  test('int value', () {
    expect(Reference.fromBuffer(b([25, 4, 1])).intValue, 25);
    expect(Reference.fromBuffer(b([231, 4, 1])).intValue, -25);
    expect(Reference.fromBuffer(b([230, 8, 1])).intValue, 230);
    expect(Reference.fromBuffer(b([230, 0, 5, 2])).intValue, 230);
    expect(Reference.fromBuffer(b([1, 4, 5, 2])).intValue, 1025);
    expect(Reference.fromBuffer(b([255, 251, 5, 2])).intValue, -1025);
    expect(Reference.fromBuffer(b([1, 4, 9, 2])).intValue, 1025);
    expect(Reference.fromBuffer(b([255, 255, 255, 127, 6, 4])).intValue,
        2147483647);
    expect(Reference.fromBuffer(b([0, 0, 0, 128, 6, 4])).intValue, -2147483648);
    expect(
        Reference.fromBuffer(b([255, 255, 255, 255, 0, 0, 0, 0, 7, 8]))
            .intValue,
        4294967295);
    expect(
        Reference.fromBuffer(b([255, 255, 255, 255, 255, 255, 255, 127, 7, 8]))
            .intValue,
        9223372036854775807);
    expect(Reference.fromBuffer(b([0, 0, 0, 0, 0, 0, 0, 128, 7, 8])).intValue,
        -9223372036854775808);
    // Dart does not really support UInt64
//      expect(FlxValue.fromBuffer(b([255, 255, 255, 255, 255, 255, 255, 255, 11, 8])).intValue, 18446744073709551615);
  });
  test('double value', () {
    expect(Reference.fromBuffer(b([0, 0, 128, 63, 14, 4])).doubleValue, 1.0);
    expect(Reference.fromBuffer(b([0, 0, 144, 64, 14, 4])).doubleValue, 4.5);
    expect(Reference.fromBuffer(b([205, 204, 204, 61, 14, 4])).doubleValue,
        closeTo(.1, .001));
    expect(
        Reference.fromBuffer(b([154, 153, 153, 153, 153, 153, 185, 63, 15, 8]))
            .doubleValue,
        .1);
  });
  test('num value', () {
    expect(Reference.fromBuffer(b([0, 0, 144, 64, 14, 4])).numValue, 4.5);
    expect(Reference.fromBuffer(b([205, 204, 204, 61, 14, 4])).numValue,
        closeTo(.1, .001));
    expect(
        Reference.fromBuffer(b([154, 153, 153, 153, 153, 153, 185, 63, 15, 8]))
            .numValue,
        .1);
    expect(Reference.fromBuffer(b([255, 251, 5, 2])).numValue, -1025);
  });
  test('string value', () {
    expect(
        Reference.fromBuffer(b([5, 77, 97, 120, 105, 109, 0, 6, 20, 1]))
            .stringValue,
        'Maxim');
    expect(
        Reference.fromBuffer(b([
          10,
          104,
          101,
          108,
          108,
          111,
          32,
          240,
          159,
          152,
          177,
          0,
          11,
          20,
          1
        ])).stringValue,
        'hello 😱');
  });
  test('blob value', () {
    expect(
        Reference.fromBuffer(b([3, 1, 2, 3, 3, 100, 1])).blobValue, [1, 2, 3]);
  });
  test('bool vector', () {
    var flx = Reference.fromBuffer(b([3, 1, 0, 1, 3, 144, 1]));
    expect(flx[0].boolValue, true);
    expect(flx[1].boolValue, false);
    expect(flx[2].boolValue, true);
  });
  test('number vector', () {
    testNumbers([3, 1, 2, 3, 3, 44, 1], [1, 2, 3]);
    testNumbers([3, 255, 2, 3, 3, 44, 1], [-1, 2, 3]);
    testNumbers([3, 0, 1, 0, 43, 2, 3, 0, 6, 45, 1], [1, 555, 3]);
    testNumbers([3, 0, 0, 0, 1, 0, 0, 0, 204, 216, 0, 0, 3, 0, 0, 0, 12, 46, 1],
        [1, 55500, 3]);
    testNumbers([
      3,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      172,
      128,
      94,
      239,
      12,
      0,
      0,
      0,
      3,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      24,
      47,
      1
    ], [
      1,
      55555555500,
      3
    ]);
    testNumbers(
        [3, 0, 0, 0, 0, 0, 192, 63, 0, 0, 32, 64, 0, 0, 96, 64, 12, 54, 1],
        [1.5, 2.5, 3.5]);
    testNumbers([
      3,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      154,
      153,
      153,
      153,
      153,
      153,
      241,
      63,
      154,
      153,
      153,
      153,
      153,
      153,
      1,
      64,
      102,
      102,
      102,
      102,
      102,
      102,
      10,
      64,
      24,
      55,
      1
    ], [
      1.1,
      2.2,
      3.3
    ]);
  });
  test('number vector, fixed type', () {
    testNumbers([1, 2, 2, 64, 1], [1, 2]);
    testNumbers([255, 255, 0, 1, 4, 65, 1], [-1, 256]);
    testNumbers([211, 255, 255, 255, 0, 232, 3, 0, 8, 66, 1], [-45, 256000]);
    testNumbers([
      211,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      127,
      16,
      67,
      1
    ], [
      -45,
      9223372036854775807
    ]);

    testNumbers([1, 2, 2, 68, 1], [1, 2]);
    testNumbers([1, 0, 0, 1, 4, 69, 1], [1, 256]);
    testNumbers([45, 0, 0, 0, 0, 232, 3, 0, 8, 70, 1], [45, 256000]);

    testNumbers([205, 204, 140, 63, 0, 0, 0, 192, 8, 74, 1], [1.1, -2]);
    testNumbers([
      154,
      153,
      153,
      153,
      153,
      153,
      241,
      63,
      0,
      0,
      0,
      0,
      0,
      0,
      112,
      192,
      16,
      75,
      1
    ], [
      1.1,
      -256
    ]);

    testNumbers([211, 255, 255, 255, 0, 232, 3, 0, 4, 0, 0, 0, 12, 78, 1],
        [-45, 256000, 4]);

    testNumbers([
      211,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      127,
      4,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      9,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      32,
      91,
      1
    ], [
      -45,
      9223372036854775807,
      4,
      9
    ]);

    testNumbers([
      45,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      127,
      4,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      9,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      32,
      95,
      1
    ], [
      45,
      9223372036854775807,
      4,
      9
    ]);

    testNumbers([
      154,
      153,
      153,
      153,
      153,
      153,
      241,
      63,
      0,
      0,
      0,
      0,
      0,
      0,
      112,
      64,
      0,
      0,
      0,
      0,
      0,
      0,
      16,
      64,
      24,
      87,
      1
    ], [
      1.1,
      256,
      4
    ]);

    testNumbers([
      154,
      153,
      153,
      153,
      153,
      153,
      241,
      63,
      0,
      0,
      0,
      0,
      0,
      0,
      112,
      64,
      0,
      0,
      0,
      0,
      0,
      0,
      16,
      64,
      0,
      0,
      0,
      0,
      0,
      0,
      34,
      64,
      32,
      99,
      1
    ], [
      1.1,
      256,
      4,
      9
    ]);
  });
  test('string vector', () {
    testStrings([
      3,
      102,
      111,
      111,
      0,
      3,
      98,
      97,
      114,
      0,
      3,
      98,
      97,
      122,
      0,
      3,
      15,
      11,
      7,
      3,
      60,
      1
    ], [
      'foo',
      'bar',
      'baz'
    ]);
    testStrings([
      3,
      102,
      111,
      111,
      0,
      3,
      98,
      97,
      114,
      0,
      3,
      98,
      97,
      122,
      0,
      6,
      15,
      11,
      7,
      18,
      14,
      10,
      6,
      60,
      1
    ], [
      'foo',
      'bar',
      'baz',
      'foo',
      'bar',
      'baz'
    ]);
  });
  test('mixed vector', () {
    var flx = Reference.fromBuffer(b([
      3,
      102,
      111,
      111,
      0,
      0,
      0,
      0,
      5,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      15,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      251,
      255,
      255,
      255,
      255,
      255,
      255,
      255,
      205,
      204,
      204,
      204,
      204,
      204,
      244,
      63,
      1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      20,
      4,
      4,
      15,
      104,
      45,
      43,
      1
    ]));
    expect(flx.length, 5);
    expect(flx[0].stringValue, 'foo');
    expect(flx[1].numValue, 1);
    expect(flx[2].numValue, -5);
    expect(flx[3].numValue, 1.3);
    expect(flx[4].boolValue, true);
  });

  test('single value map', () {
    var flx = Reference.fromBuffer(b([97, 0, 1, 3, 1, 1, 1, 12, 4, 2, 36, 1]));
    expect(flx.length, 1);
    expect(flx['a'].numValue, 12);
  });
  test('two value map', () {
    var flx = Reference.fromBuffer(
        b([0, 97, 0, 2, 4, 4, 2, 1, 2, 45, 12, 4, 4, 4, 36, 1]));
    expect(flx.length, 2);
    expect(flx['a'].numValue, 12);
    expect(flx[''].numValue, 45);
  });
  test('complex map', () {
    var flx = complexMap();
    expect(flx.length, 5);
    expect(flx['age'].numValue, 35);
    expect(flx['weight'].numValue, 72.5);
    expect(flx['name'].stringValue, 'Maxim');

    expect(flx['flags'].length, 4);
    expect(flx['flags'][0].boolValue, true);
    expect(flx['flags'][1].boolValue, false);
    expect(flx['flags'][2].boolValue, true);
    expect(flx['flags'][3].boolValue, true);

    expect(flx['address'].length, 3);
    expect(flx['address']['city'].stringValue, 'Bla');
    expect(flx['address']['zip'].stringValue, '12345');
    expect(flx['address']['countryCode'].stringValue, 'XX');

    expect(
        () => flx['address']['country'].stringValue,
        throwsA(predicate((dynamic e) =>
            e is ArgumentError &&
            e.message ==
                'Key: [country] is not applicable on: //address of: ValueType.Map')));
    expect(
        () => flx['address']['countryCode'][0],
        throwsA(predicate((dynamic e) =>
            e is ArgumentError &&
            e.message ==
                'Key: [0] is not applicable on: //address/countryCode of: ValueType.String')));
    expect(
        () => flx[1],
        throwsA(predicate((dynamic e) =>
            e is ArgumentError &&
            e.message ==
                'Key: [1] is not applicable on: / of: ValueType.Map')));
    expect(
        () => flx['flags'][4],
        throwsA(predicate((dynamic e) =>
            e is ArgumentError &&
            e.message ==
                'Key: [4] is not applicable on: //flags of: ValueType.VectorBool length: 4')));
    expect(
        () => flx['flags'][-1],
        throwsA(predicate((dynamic e) =>
            e is ArgumentError &&
            e.message ==
                'Key: [-1] is not applicable on: //flags of: ValueType.VectorBool length: 4')));
  });
  test('complex map to json', () {
    var flx = complexMap();
    expect(flx.json,
        '{"address":{"city":"Bla","countryCode":"XX","zip":"12345"},"age":35,"flags":[true,false,true,true],"name":"Maxim","weight":72.5}');
  });

  test('complex map iterators', () {
    var flx = complexMap();
    expect(flx.mapKeyIterable.map((e) => e).toList(),
        ['address', 'age', 'flags', 'name', 'weight']);
    expect(flx.mapValueIterable.map((e) => e.json).toList(), [
      flx['address'].json,
      flx['age'].json,
      flx['flags'].json,
      flx['name'].json,
      flx['weight'].json
    ]);
    expect(flx['flags'].vectorIterable.map((e) => e.boolValue).toList(),
        [true, false, true, true]);
  });

  test('bug where offest were stored as int instead of uint', () {
    const data = [
      99,
      104,
      97,
      110,
      110,
      101,
      108,
      115,
      95,
      105,
      110,
      0,
      100,
      105,
      108,
      97,
      116,
      105,
      111,
      110,
      95,
      104,
      101,
      105,
      103,
      104,
      116,
      95,
      102,
      97,
      99,
      116,
      111,
      114,
      0,
      100,
      105,
      108,
      97,
      116,
      105,
      111,
      110,
      95,
      119,
      105,
      100,
      116,
      104,
      95,
      102,
      97,
      99,
      116,
      111,
      114,
      0,
      102,
      117,
      115,
      101,
      100,
      95,
      97,
      99,
      116,
      105,
      118,
      97,
      116,
      105,
      111,
      110,
      95,
      102,
      117,
      110,
      99,
      116,
      105,
      111,
      110,
      0,
      112,
      97,
      100,
      95,
      118,
      97,
      108,
      117,
      101,
      115,
      0,
      112,
      97,
      100,
      100,
      105,
      110,
      103,
      0,
      115,
      116,
      114,
      105,
      100,
      101,
      95,
      104,
      101,
      105,
      103,
      104,
      116,
      0,
      115,
      116,
      114,
      105,
      100,
      101,
      95,
      119,
      105,
      100,
      116,
      104,
      0,
      8,
      130,
      119,
      97,
      76,
      51,
      41,
      34,
      21,
      8,
      1,
      8,
      64,
      1,
      1,
      1,
      1,
      0,
      1,
      1,
      4,
      4,
      4,
      4,
      4,
      4,
      4,
      4,
      16,
      36,
      1
    ];
    var flx = Reference.fromBuffer(b(data));
    expect(flx.json,
        '{"channels_in":64,"dilation_height_factor":1,"dilation_width_factor":1,"fused_activation_function":1,"pad_values":1,"padding":0,"stride_height":1,"stride_width":1}');
    const object = {
      "channels_in": 64,
      "dilation_height_factor": 1,
      "dilation_width_factor": 1,
      "fused_activation_function": 1,
      "pad_values": 1,
      "padding": 0,
      "stride_height": 1,
      "stride_width": 1
    };
    var data1 = Builder.buildFromObject(object).asUint8List();
    expect(data1.length, data.length);
    var flx1 = Reference.fromBuffer(b(data1));
    expect(flx1.json,
        '{"channels_in":64,"dilation_height_factor":1,"dilation_width_factor":1,"fused_activation_function":1,"pad_values":1,"padding":0,"stride_height":1,"stride_width":1}');
  });
}

ByteBuffer b(List<int> values) {
  var data = Uint8List.fromList(values);
  return data.buffer;
}

void testNumbers(List<int> buffer, List<num> numbers) {
  var flx = Reference.fromBuffer(b(buffer));
  expect(flx.length, numbers.length);
  for (var i = 0; i < flx.length; i++) {
    expect(flx[i].numValue, closeTo(numbers[i], 0.001));
  }
}

void testStrings(List<int> buffer, List<String> numbers) {
  var flx = Reference.fromBuffer(b(buffer));
  expect(flx.length, numbers.length);
  for (var i = 0; i < flx.length; i++) {
    expect(flx[i].stringValue, numbers[i]);
  }
}

Reference complexMap() {
//  {
//    "age": 35,
//    "flags": [True, False, True, True],
//    "weight": 72.5,
//    "name": "Maxim",
//    "address": {
//      "city": "Bla",
//      "zip": "12345",
//      "countryCode": "XX",
//    }
//  }
  return Reference.fromBuffer(b([
    97,
    100,
    100,
    114,
    101,
    115,
    115,
    0,
    99,
    105,
    116,
    121,
    0,
    3,
    66,
    108,
    97,
    0,
    99,
    111,
    117,
    110,
    116,
    114,
    121,
    67,
    111,
    100,
    101,
    0,
    2,
    88,
    88,
    0,
    122,
    105,
    112,
    0,
    5,
    49,
    50,
    51,
    52,
    53,
    0,
    3,
    38,
    29,
    14,
    3,
    1,
    3,
    38,
    22,
    15,
    20,
    20,
    20,
    97,
    103,
    101,
    0,
    102,
    108,
    97,
    103,
    115,
    0,
    4,
    1,
    0,
    1,
    1,
    110,
    97,
    109,
    101,
    0,
    5,
    77,
    97,
    120,
    105,
    109,
    0,
    119,
    101,
    105,
    103,
    104,
    116,
    0,
    5,
    93,
    36,
    33,
    23,
    12,
    0,
    0,
    7,
    0,
    0,
    0,
    1,
    0,
    0,
    0,
    5,
    0,
    0,
    0,
    60,
    0,
    0,
    0,
    35,
    0,
    0,
    0,
    51,
    0,
    0,
    0,
    45,
    0,
    0,
    0,
    0,
    0,
    145,
    66,
    36,
    4,
    144,
    20,
    14,
    25,
    38,
    1
  ]));
}
