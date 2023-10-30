const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.js_export = () => {
    const i8 = new Int8Array(2);
    i8[0] = 1;
    i8[1] = 2;
    assert.deepStrictEqual(wasm.export_i8(i8), i8);
    const u8 = new Uint8Array(2);
    u8[0] = 1;
    u8[1] = 2;
    assert.deepStrictEqual(wasm.export_u8(u8), u8);

    const i16 = new Int16Array(2);
    i16[0] = 1;
    i16[1] = 2;
    assert.deepStrictEqual(wasm.export_i16(i16), i16);
    const u16 = new Uint16Array(2);
    u16[0] = 1;
    u16[1] = 2;
    assert.deepStrictEqual(wasm.export_u16(u16), u16);

    const i32 = new Int32Array(2);
    i32[0] = 1;
    i32[1] = 2;
    assert.deepStrictEqual(wasm.export_i32(i32), i32);
    assert.deepStrictEqual(wasm.export_isize(i32), i32);
    const u32 = new Uint32Array(2);
    u32[0] = 1;
    u32[1] = 2;
    assert.deepStrictEqual(wasm.export_u32(u32), u32);
    assert.deepStrictEqual(wasm.export_usize(u32), u32);

    const f32 = new Float32Array(2);
    f32[0] = 1;
    f32[1] = 2;
    assert.deepStrictEqual(wasm.export_f32(f32), f32);
    const f64 = new Float64Array(2);
    f64[0] = 1;
    f64[1] = 2;
    assert.deepStrictEqual(wasm.export_f64(f64), f64);
};

const test_import = (a, b, c) => {
    assert.strictEqual(a.length, 2);
    assert.strictEqual(a[0], 1);
    assert.strictEqual(a[1], 2);
    assert.strictEqual(b.length, 2);
    assert.strictEqual(b[0], 1);
    assert.strictEqual(b[1], 2);
    assert.strictEqual(c, undefined);
    return a;
};

exports.import_js_i8 = test_import;
exports.import_js_u8 = test_import;
exports.import_js_i16 = test_import;
exports.import_js_u16 = test_import;
exports.import_js_i32 = test_import;
exports.import_js_isize = test_import;
exports.import_js_u32 = test_import;
exports.import_js_usize = test_import;
exports.import_js_f32 = test_import;
exports.import_js_f64 = test_import;

exports.js_import = () => {
    const i8 = new Int8Array(2);
    i8[0] = 1;
    i8[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_i8(i8), i8);
    const u8 = new Uint8Array(2);
    u8[0] = 1;
    u8[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_u8(u8), u8);

    const i16 = new Int16Array(2);
    i16[0] = 1;
    i16[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_i16(i16), i16);
    const u16 = new Uint16Array(2);
    u16[0] = 1;
    u16[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_u16(u16), u16);

    const i32 = new Int32Array(2);
    i32[0] = 1;
    i32[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_i32(i32), i32);
    assert.deepStrictEqual(wasm.import_rust_isize(i32), i32);
    const u32 = new Uint32Array(2);
    u32[0] = 1;
    u32[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_u32(u32), u32);
    assert.deepStrictEqual(wasm.import_rust_usize(u32), u32);

    const f32 = new Float32Array(2);
    f32[0] = 1;
    f32[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_f32(f32), f32);
    const f64 = new Float64Array(2);
    f64[0] = 1;
    f64[1] = 2;
    assert.deepStrictEqual(wasm.import_rust_f64(f64), f64);
};

exports.js_pass_array = () => {
    wasm.pass_array_rust_i8([1, 2]);
    wasm.pass_array_rust_u8([1, 2]);
    wasm.pass_array_rust_i16([1, 2]);
    wasm.pass_array_rust_u16([1, 2]);
    wasm.pass_array_rust_i32([1, 2]);
    wasm.pass_array_rust_u32([1, 2]);
    wasm.pass_array_rust_isize([1, 2]);
    wasm.pass_array_rust_usize([1, 2]);
    wasm.pass_array_rust_f32([1, 2]);
    wasm.pass_array_rust_f64([1, 2]);
};

const import_mut_foo = (a, b, c) => {
    assert.strictEqual(a.length, 3);
    assert.strictEqual(a[0], 1);
    assert.strictEqual(a[1], 2);
    a[0] = 4;
    a[1] = 5;
    assert.strictEqual(b.length, 3);
    assert.strictEqual(b[0], 4);
    assert.strictEqual(b[1], 5);
    assert.strictEqual(b[2], 6);
    b[0] = 8;
    b[1] = 7;
    assert.strictEqual(c, undefined);
};

exports.import_mut_js_i8 = import_mut_foo;
exports.import_mut_js_u8 = import_mut_foo;
exports.import_mut_js_i16 = import_mut_foo;
exports.import_mut_js_u16 = import_mut_foo;
exports.import_mut_js_i32 = import_mut_foo;
exports.import_mut_js_u32 = import_mut_foo;
exports.import_mut_js_isize = import_mut_foo;
exports.import_mut_js_usize = import_mut_foo;
exports.import_mut_js_f32 = import_mut_foo;
exports.import_mut_js_f64 = import_mut_foo;

const export_mut_run = (a, rust) => {
    assert.strictEqual(a.length, 3);
    a[0] = 1;
    a[1] = 2;
    a[2] = 3;
    console.log(a);
    rust(a);
    console.log(a);
    assert.strictEqual(a.length, 3);
    assert.strictEqual(a[0], 4);
    assert.strictEqual(a[1], 5);
    assert.strictEqual(a[2], 3);
};

exports.js_export_mut = () => {
    export_mut_run(new Int8Array(3), wasm.export_mut_i8);
    export_mut_run(new Uint8Array(3), wasm.export_mut_u8);
    export_mut_run(new Int16Array(3), wasm.export_mut_i16);
    export_mut_run(new Uint16Array(3), wasm.export_mut_u16);
    export_mut_run(new Int32Array(3), wasm.export_mut_i32);
    export_mut_run(new Uint32Array(3), wasm.export_mut_u32);
    export_mut_run(new Int32Array(3), wasm.export_mut_isize);
    export_mut_run(new Uint32Array(3), wasm.export_mut_usize);
    export_mut_run(new Float32Array(3), wasm.export_mut_f32);
    export_mut_run(new Float64Array(3), wasm.export_mut_f64);
};

exports.js_return_vec = () => {
    const app = wasm.return_vec_web_main();

    for (let i = 0; i < 10; i++) {
        app.tick();
        const bad = wasm.return_vec_broken_vec();
        console.log('Received from rust:', i, bad);
        assert.strictEqual(bad[0], 1);
        assert.strictEqual(bad[1], 2);
        assert.strictEqual(bad[2], 3);
        assert.strictEqual(bad[3], 4);
        assert.strictEqual(bad[4], 5);
        assert.strictEqual(bad[5], 6);
        assert.strictEqual(bad[6], 7);
        assert.strictEqual(bad[7], 8);
        assert.strictEqual(bad[8], 9);
    }
};

exports.js_clamped = (a, offset) => {
  assert.ok(a instanceof Uint8ClampedArray);
  assert.equal(a.length, 3);
  assert.equal(a[0], offset + 0);
  assert.equal(a[1], offset + 1);
  assert.equal(a[2], offset + 2);
};
