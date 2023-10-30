const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.isize_js_identity = a => a;
exports.usize_js_identity = a => a;

exports.js_works = async () => {
    assert.strictEqual(wasm.usize_zero(), 0);
    assert.strictEqual(wasm.usize_one(), 1);
    assert.strictEqual(wasm.isize_neg_one(), -1);
    assert.strictEqual(wasm.isize_i32_min(), -2147483648);
    assert.strictEqual(wasm.isize_min(), -2147483648);
    assert.strictEqual(wasm.usize_u32_max(), 4294967295);
    assert.strictEqual(wasm.usize_max(), 4294967295);

    assert.strictEqual(wasm.isize_rust_identity(0), 0);
    assert.strictEqual(wasm.isize_rust_identity(1), 1);
    assert.strictEqual(wasm.isize_rust_identity(-1), -1);
    assert.strictEqual(wasm.usize_rust_identity(0), 0);
    assert.strictEqual(wasm.usize_rust_identity(1), 1);

    const usize_max = 4294967295;
    const isize_min = -2147483648;
    assert.strictEqual(wasm.isize_rust_identity(isize_min), isize_min);
    assert.strictEqual(wasm.usize_rust_identity(usize_max), usize_max);

    assert.deepStrictEqual(wasm.usize_slice([]), new Uint32Array());
    assert.deepStrictEqual(wasm.isize_slice([]), new Int32Array());
    const arr1 = new Uint32Array([1, 2]);
    assert.deepStrictEqual(wasm.usize_slice([1, 2]), arr1);
    const arr2 = new Int32Array([1, 2]);
    assert.deepStrictEqual(wasm.isize_slice([1, 2]), arr2);

    assert.deepStrictEqual(wasm.isize_slice([isize_min]), new Int32Array([isize_min]));
    assert.deepStrictEqual(wasm.usize_slice([usize_max]), new Uint32Array([usize_max]));

    assert.deepStrictEqual(await wasm.async_usize_one(), 1);
    assert.deepStrictEqual(await wasm.async_isize_neg_one(), -1);
};
