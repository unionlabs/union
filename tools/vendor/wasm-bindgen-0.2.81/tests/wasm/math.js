const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.js_auto_bind_math = () => {
    wasm.math(1.0, 2.0);
};

exports.roundtrip = x => x;

exports.test_js_roundtrip = () => {
  assert.strictEqual(wasm.rust_roundtrip_i8(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_i8(0x80), -128);
  assert.strictEqual(wasm.rust_roundtrip_i8(0x7f), 127);

  assert.strictEqual(wasm.rust_roundtrip_i16(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_i16(0x8000), -32768);
  assert.strictEqual(wasm.rust_roundtrip_i16(0x7fff), 32767);

  assert.strictEqual(wasm.rust_roundtrip_i32(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_i32(0x80000000), -2147483648);
  assert.strictEqual(wasm.rust_roundtrip_i32(0x7fffffff), 2147483647);

  assert.strictEqual(wasm.rust_roundtrip_u8(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_u8(0x80), 128);
  assert.strictEqual(wasm.rust_roundtrip_u8(0x7f), 127);
  assert.strictEqual(wasm.rust_roundtrip_u8(0xff), 255);

  assert.strictEqual(wasm.rust_roundtrip_u16(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_u16(0x8000), 32768);
  assert.strictEqual(wasm.rust_roundtrip_u16(0x7fff), 32767);
  assert.strictEqual(wasm.rust_roundtrip_u16(0xffff), 65535);

  assert.strictEqual(wasm.rust_roundtrip_u32(0), 0);
  assert.strictEqual(wasm.rust_roundtrip_u32(0x80000000), 2147483648);
  assert.strictEqual(wasm.rust_roundtrip_u32(0x7fffffff), 2147483647);
  assert.strictEqual(wasm.rust_roundtrip_u32(0xffffffff), 4294967295);
};
