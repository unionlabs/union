// Run this using JavaScriptTest.sh
import assert from 'assert'
import fs from 'fs'
import * as flexbuffers from 'flatbuffers/js/flexbuffers.js'

function main() {
  testSingleValueBuffers();
  testGoldBuffer();
  testEncode();
  testIndirectAdd();
  testIndirectWithCache();
  testMapBuilder();
  testRoundTrip();
  testRoundTripWithBuilder();
  testDeduplicationOff();
  testBugWhereOffestWereStoredAsIntInsteadOfUInt();

  console.log('FlexBuffers test: completed successfully');
}

function testSingleValueBuffers() {
  {
    const ref = flexbuffers.toReference(new Uint8Array([0, 0, 1]).buffer);
    assert.strictEqual(true, ref.isNull());
  }

  function _assert(object, buffer) {
    assert.deepStrictEqual(flexbuffers.toObject(new Uint8Array(buffer).buffer), object);
  }
  _assert(true, [1, 104, 1]);
  _assert(false, [0, 104, 1]);
  _assert(25, [25, 4, 1]);
  _assert(-25, [231, 4, 1]);
  _assert(230, [230, 8, 1]);
  _assert(230, [230, 0, 5, 2]);
  _assert(-1025, [255, 251, 5, 2]);
  _assert(1025, [1, 4, 9, 2]);
  _assert(2147483647, [255, 255, 255, 127, 6, 4]);
  _assert(-2147483648, [0, 0, 0, 128, 6, 4]);
  _assert(4294967295n, [255, 255, 255, 255, 0, 0, 0, 0, 7, 8]);
  _assert(9223372036854775807n, [255, 255, 255, 255, 255, 255, 255, 127, 7, 8]);
  _assert(-9223372036854775808n, [0, 0, 0, 0, 0, 0, 0, 128, 7, 8]);
  _assert(18446744073709551615n, [255, 255, 255, 255, 255, 255, 255, 255, 11, 8]);
  _assert(4.5, [0, 0, 144, 64, 14, 4]);
  _assert(0.10000000149011612, [205, 204, 204, 61, 14, 4]);
  _assert(0.1, [154, 153, 153, 153, 153, 153, 185, 63, 15, 8]);
  _assert(-1025, [255, 251, 5, 2]);
  _assert("Maxim", [5, 77, 97, 120, 105, 109, 0, 6, 20, 1]);
  _assert("hello 😱", [10, 104, 101, 108, 108, 111, 32, 240, 159, 152, 177, 0, 11, 20, 1]);
  _assert({a:12}, [97, 0, 1, 3, 1, 1, 1, 12, 4, 2, 36, 1]);
  _assert({"":45, "a": 12}, [0, 97, 0, 2, 4, 4, 2, 1, 2, 45, 12, 4, 4, 4, 36, 1]);
}

function testEncode() {
  function _assert(value, buffer) {
    assert.deepStrictEqual(flexbuffers.encode(value), new Uint8Array(buffer));
  }
  _assert(null, [0, 0, 1]);
  _assert(true, [1, 104, 1]);
  _assert(false, [0, 104, 1]);
  _assert(1, [1, 4, 1]);
  _assert(230, [230, 0, 5, 2]);
  _assert(1025, [1, 4, 5, 2]);
  _assert(-1025, [255, 251, 5, 2]);
  _assert(0x100000001, [1, 0, 0, 0, 1, 0, 0, 0, 7, 8]);
  _assert(0.1, [154, 153, 153, 153, 153, 153, 185, 63, 15, 8]);
  _assert(0.5, [0, 0, 0, 63, 14, 4]);
  _assert(new Uint8Array([1, 2, 3]), [3, 1, 2, 3, 3, 100, 1]);
  _assert("Maxim", [5, 77, 97, 120, 105, 109, 0, 6, 20, 1]);
  _assert("hello 😱", [10, 104, 101, 108, 108, 111, 32, 240, 159, 152, 177, 0, 11, 20, 1]);
  _assert([1, 2], [1, 2, 2, 64, 1]);
  _assert([-1, 256], [255, 255, 0, 1, 4, 65, 1]);
  _assert([-45, 256000], [211, 255, 255, 255, 0, 232, 3, 0, 8, 66, 1]);
  _assert([1.1, -256.0], [2, 0, 0, 0, 0, 0, 0, 0, 154, 153, 153, 153, 153, 153, 241, 63, 0, 255, 255, 255, 255, 255, 255, 255, 15, 5, 18, 43, 1]);
  _assert([1, 2, 4], [1, 2, 4, 3, 76, 1]);
  _assert([-1, 256, 4], [255, 255, 0, 1, 4, 0, 6, 77, 1]);
  _assert([[61], 64], [1, 61, 2, 2, 64, 44, 4, 4, 40, 1]);
  _assert(["foo", "bar", "baz"], [3, 102, 111, 111, 0, 3, 98, 97, 114, 0, 3, 98, 97, 122, 0, 3, 15, 11, 7, 3, 60, 1]);
  _assert(["foo", "bar", "baz", "foo", "bar", "baz"], [3, 102, 111, 111, 0, 3, 98, 97, 114, 0, 3, 98, 97, 122, 0, 6, 15, 11, 7, 18, 14, 10, 6, 60, 1]);
  _assert([true, false, true], [3, 1, 0, 1, 3, 144, 1]);
  _assert(['foo', 1, -5, 1.3, true], [
    3, 102, 111, 111, 0, 0, 0, 0,
    5, 0, 0, 0, 0, 0, 0, 0,
    15, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0,
    251, 255, 255, 255, 255, 255, 255, 255,
    205, 204, 204, 204, 204, 204, 244, 63,
    1, 0, 0, 0, 0, 0, 0, 0,
    20, 4, 4, 15, 104, 45, 43, 1
  ]);
  _assert([1, 3.3, 'max', true, null, false], [
    3, 109, 97, 120, 0, 0, 0, 0,
    6, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0,
    102, 102, 102, 102, 102, 102, 10, 64,
    31, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    4, 15, 20, 104, 0, 104, 54, 43, 1
  ]);
  _assert({"a": 12}, [97, 0, 1, 3, 1, 1, 1, 12, 4, 2, 36, 1]);
  _assert({"a": 12, "":45}, [0, 97, 0, 2, 4, 4, 2, 1, 2, 45, 12, 4, 4, 4, 36, 1]);
  // JS currently does not support key vector offset sharing
  _assert([{'something':12}, {'something': 45}], [
    115, 111, 109, 101, 116, 104, 105, 110, 103, 0,
    1, 11, 1, 1, 1, 12, 4, 6, 1, 1, 45, 4, 2, 8, 4, 36, 36, 4, 40, 1
  ]);
}

function testDeduplicationOff() {
  let buffer = flexbuffers.encode([{'something':12}, {'something': 45}], 1, true, true, false);
  assert.deepStrictEqual(buffer, new Uint8Array([
    115, 111, 109, 101, 116, 104, 105, 110, 103,
    0,   1,  11,   1,   1,   1,  12,   4,   1,
    18,   1,   1,   1,  45,   4,   2,  10,   4,
    36,  36,   4,  40,   1
  ]));

  buffer = flexbuffers.encode([{'something':12}, {'something': 45}], 1, true, false, false);
  assert.deepStrictEqual(buffer, new Uint8Array([
    115, 111, 109, 101, 116, 104, 105, 110, 103,   0,
    1,  11,   1,   1,   1,  12,   4, 115, 111, 109,
    101, 116, 104, 105, 110, 103,   0,   1,  11,   1,
    1,   1,  45,   4,   2,  20,   4,  36,  36,   4,
    40,   1
  ]));

  buffer = flexbuffers.encode(['something', 'something', 'dark'], 1, true, false, false);
  assert.deepStrictEqual(buffer, new Uint8Array([
    9, 115, 111, 109, 101, 116, 104,
    105, 110, 103,   0,   4, 100,  97,
    114, 107,   0,   3,  17,  18,   8,
    3,  60,   1
  ]));

  buffer = flexbuffers.encode(['something', 'something', 'dark'], 1, false, false, false);
  assert.deepStrictEqual(buffer, new Uint8Array([
    9, 115, 111, 109, 101, 116, 104, 105, 110,
    103,   0,   9, 115, 111, 109, 101, 116, 104,
    105, 110, 103,   0,   4, 100,  97, 114, 107,
    0,   3,  28,  18,   8,   3,  60,   1
  ]));
}

function testIndirectAdd() {
  function _assertInt(buffer, value, indirect = false, cache = false) {
    const builder = flexbuffers.builder();
    builder.addInt(value, indirect, cache);
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }
  function _assertUInt(buffer, value, indirect = false, cache = false) {
    const builder = flexbuffers.builder();
    builder.addUInt(value, indirect, cache);
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }
  function _assertFloat(buffer, value, indirect = false, cache = false) {
    const builder = flexbuffers.builder();
    builder.addFloat(value, indirect, cache);
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }
  _assertInt([0, 4, 1], 0);
  _assertInt([0, 1, 24, 1], 0, true);
  _assertInt([255, 0, 5, 2], 255);

  _assertUInt([0, 8, 1], 0);
  _assertUInt([0, 1, 28, 1], 0, true);
  _assertUInt([255, 8, 1], 255);

  _assertUInt([185, 115, 175, 118, 250, 84, 8, 0, 11, 8], 2345234523452345);
  _assertUInt([185, 115, 175, 118, 250, 84, 8, 0, 8, 31, 1], 2345234523452345, true);
  _assertInt([185, 115, 175, 118, 250, 84, 8, 0, 7, 8], 2345234523452345);
  _assertInt([185, 115, 175, 118, 250, 84, 8, 0, 8, 27, 1], 2345234523452345, true);

  _assertFloat([154, 153, 153, 153, 153, 153, 185, 63, 15, 8], 0.1);
  _assertFloat([154, 153, 153, 153, 153, 153, 185, 63, 8, 35, 1], 0.1, true);
  _assertFloat([0, 0, 0, 0, 14, 4], 0);
  _assertFloat([0, 0, 0, 0, 4, 34, 1], 0, true);
}

function testIndirectWithCache() {
  function _assertInt(buffer, values) {
    const builder = flexbuffers.builder();
    builder.startVector();
    values.forEach(v => {
      builder.addInt(v, true, true)
    });
    builder.end();
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }

  function _assertUInt(buffer, values) {
    const builder = flexbuffers.builder();
    builder.startVector();
    values.forEach(v => {
      builder.addUInt(v, true, true);
    });
    builder.end();
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }

  function _assertFloat(buffer, values) {
    const builder = flexbuffers.builder();
    builder.startVector();
    values.forEach(v => {
      builder.addFloat(v, true, true);
    });
    builder.end();
    const data = builder.finish();
    assert.deepStrictEqual(data, new Uint8Array(buffer));
  }

  _assertInt(
    [185, 115, 175, 118, 250, 84, 8, 0, 4, 9, 10, 11, 12, 27, 27, 27, 27, 8, 40, 1],
    [2345234523452345, 2345234523452345, 2345234523452345, 2345234523452345]
  );

  _assertUInt(
    [185, 115, 175, 118, 250, 84, 8, 0, 4, 9, 10, 11, 12, 31, 31, 31, 31, 8, 40, 1],
    [2345234523452345, 2345234523452345, 2345234523452345, 2345234523452345]
  );

  _assertFloat(
    [154, 153, 153, 153, 153, 153, 185, 63, 4, 9, 10, 11, 12, 35, 35, 35, 35, 8, 40, 1],
    [0.1, 0.1, 0.1, 0.1]
  );
}

function testMapBuilder() {
  const builder = flexbuffers.builder();
  builder.startMap();
  builder.addKey('a');
  builder.add(12);
  builder.addKey('');
  builder.add(45);
  builder.end();
  const data = builder.finish();
  assert.deepStrictEqual(data, new Uint8Array([97, 0, 0, 2, 2, 5, 2, 1, 2, 45, 12, 4, 4, 4, 36, 1]));
}

function testRoundTrip() {
  const example = {
    "age": 35,
    "flags": [true, false, true, true],
    "weight": 72.5,
    "name": "Maxim",
    "address": {
      "city": "Bla",
      "zip": "12345",
      "countryCode": "XX",
    }
  };

  function _assert(value) {
    let buffer = flexbuffers.encode(value, 1);
    let o = flexbuffers.toObject(buffer.buffer);
    assert.deepStrictEqual(o, value);
  }

  _assert(example);
  _assert(0x100000001n);
  _assert({ test_number: 72.6 })
}

function testRoundTripWithBuilder() {
  const example = {
    "age": 35,
    "flags": [true, false, true, true],
    "weight": 72.5,
    "name": "Maxim",
    "address": {
      "city": "Bla",
      "zip": "12345",
      "countryCode": "XX",
    }
  };

  const builder = flexbuffers.builder();
  builder.startMap();

  builder.addKey('age');
  builder.add(35);

  builder.addKey('flags');
  builder.startVector();
  builder.add(true);
  builder.add(false);
  builder.add(true);
  builder.add(true);
  builder.end();

  builder.addKey("weight");
  builder.add(72.5);

  builder.addKey("name");
  builder.add("Maxim");

  builder.addKey("address");

  builder.startMap();
  builder.addKey("city");
  builder.add("Bla");
  builder.addKey("zip");
  builder.add("12345");
  builder.addKey("countryCode");
  builder.add("XX");
  builder.end();

  builder.end();

  const data = builder.finish();
  let o = flexbuffers.toObject(data.buffer);
  assert.deepStrictEqual(o, example);

  let root = flexbuffers.toReference(data.buffer);
  assert.strictEqual(root.isMap(), true);
  assert.strictEqual(root.get("age").numericValue(), 35);
  assert.strictEqual(root.get("age").intValue(), 35);
  assert.strictEqual(root.get("name").stringValue(), "Maxim");
  assert.strictEqual(root.get("weight").floatValue(), 72.5);
  assert.strictEqual(root.get("weight").numericValue(), 72.5);
  let flags = root.get("flags");
  assert.strictEqual(flags.isVector(), true);
  assert.strictEqual(flags.length(), 4);
  assert.strictEqual(flags.get(0).boolValue(), true);
  assert.strictEqual(flags.get(1).boolValue(), false);
  assert.strictEqual(flags.get(2).boolValue(), true);
  assert.strictEqual(flags.get(3).boolValue(), true);

  let address = root.get("address");
  assert.strictEqual(address.isMap(), true);
  assert.strictEqual(address.length(), 3);
  assert.strictEqual(address.get("city").stringValue(), "Bla");
  assert.strictEqual(address.get("zip").stringValue(), "12345");
  assert.strictEqual(address.get("countryCode").stringValue(), "XX");
}

function testGoldBuffer() {
  const data = new Uint8Array(fs.readFileSync('../gold_flexbuffer_example.bin')).buffer;
  const b1 = flexbuffers.toReference(data).get("bools").get(1);
  assert.strictEqual(b1.isBool(), true);
  assert.strictEqual(b1.boolValue(), false);

  const blob = flexbuffers.toReference(data).get("vec").get(3);
  assert.strictEqual(blob.isBlob(), true);
  assert.deepStrictEqual(blob.blobValue(), new Uint8Array([77]));

  const o = flexbuffers.toObject(data);
  assert.deepStrictEqual(o, {
    bool: true,
    bools: [true, false, true, false],
    bar: [1, 2, 3],
    bar3: [1, 2, 3],
    foo: 100,
    mymap: {foo:'Fred'},
    vec: [-100, 'Fred', 4, new Uint8Array([77]), false, 4]
  });
}

function testBugWhereOffestWereStoredAsIntInsteadOfUInt() {
  // Reported in https://github.com/google/flatbuffers/issues/5949#issuecomment-688421193
  const object = {'channels_in': 64, 'dilation_height_factor': 1, 'dilation_width_factor': 1, 'fused_activation_function': 1, 'pad_values': 1, 'padding': 0, 'stride_height': 1, 'stride_width': 1};
  let data1 = flexbuffers.encode(object);
  const data = [99, 104, 97, 110, 110, 101, 108, 115, 95, 105, 110, 0,
    100, 105, 108, 97, 116, 105, 111, 110, 95, 104, 101, 105, 103, 104, 116, 95, 102, 97, 99, 116, 111, 114, 0,
    100, 105, 108, 97, 116, 105, 111, 110, 95, 119, 105, 100, 116, 104, 95, 102, 97, 99, 116, 111, 114, 0,
    102, 117, 115, 101, 100, 95, 97, 99, 116, 105, 118, 97, 116, 105, 111, 110, 95, 102, 117, 110, 99, 116, 105, 111, 110, 0,
    112, 97, 100, 95, 118, 97, 108, 117, 101, 115, 0, 112, 97, 100, 100, 105, 110, 103, 0,
    115, 116, 114, 105, 100, 101, 95, 104, 101, 105, 103, 104, 116, 0,
    115, 116, 114, 105, 100, 101, 95, 119, 105, 100, 116, 104, 0,
    8, 130, 119, 97, 76, 51, 41, 34, 21, 8, 1, 8, 64, 1, 1, 1, 1, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 16, 36, 1];
  let object2 = flexbuffers.toObject(new Uint8Array(data).buffer);
  let object1 = flexbuffers.toObject(new Uint8Array(data1).buffer);
  assert.deepStrictEqual(object, object2);
  assert.deepStrictEqual(object, object1);
  assert.strictEqual(data.length, data1.length);
  let ref = flexbuffers.toReference(new Uint8Array(data).buffer);
  assert.strictEqual(ref.isMap(), true);
  assert.strictEqual(ref.length(), 8);
  assert.strictEqual(ref.get("channels_in").numericValue(), 64);
  assert.strictEqual(ref.get("padding").isNumber(), true);
}

main();

