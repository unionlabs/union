const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.i64_js_identity = a => a;
exports.u64_js_identity = a => a;

exports.js_works = () => {
    assert.strictEqual(wasm.zero(), BigInt('0'));
    assert.strictEqual(wasm.one(), BigInt('1'));
    assert.strictEqual(wasm.neg_one(), BigInt('-1'));
    assert.strictEqual(wasm.i32_min(), BigInt('-2147483648'));
    assert.strictEqual(wasm.u32_max(), BigInt('4294967295'));
    assert.strictEqual(wasm.i64_min(), BigInt('-9223372036854775808'));
    assert.strictEqual(wasm.u64_max(), BigInt('18446744073709551615'));

    assert.strictEqual(wasm.i64_rust_identity(BigInt('0')), BigInt('0'));
    assert.strictEqual(wasm.i64_rust_identity(BigInt('1')), BigInt('1'));
    assert.strictEqual(wasm.i64_rust_identity(BigInt('-1')), BigInt('-1'));
    assert.strictEqual(wasm.u64_rust_identity(BigInt('0')), BigInt('0'));
    assert.strictEqual(wasm.u64_rust_identity(BigInt('1')), BigInt('1'));
    assert.strictEqual(wasm.u64_rust_identity(BigInt('1') << BigInt('64')), BigInt('0'));

    const u64_max = BigInt('18446744073709551615');
    const i64_min = BigInt('-9223372036854775808');
    assert.strictEqual(wasm.i64_rust_identity(i64_min), i64_min);
    assert.strictEqual(wasm.u64_rust_identity(u64_max), u64_max);

    assert.deepStrictEqual(wasm.u64_slice([]), new BigUint64Array());
    assert.deepStrictEqual(wasm.i64_slice([]), new BigInt64Array());
    const arr1 = new BigUint64Array([BigInt('1'), BigInt('2')]);
    assert.deepStrictEqual(wasm.u64_slice([BigInt('1'), BigInt('2')]), arr1);
    const arr2 = new BigInt64Array([BigInt('1'), BigInt('2')]);
    assert.deepStrictEqual(wasm.i64_slice([BigInt('1'), BigInt('2')]), arr2);

    assert.deepStrictEqual(wasm.i64_slice([i64_min]), new BigInt64Array([i64_min]));
    assert.deepStrictEqual(wasm.u64_slice([u64_max]), new BigUint64Array([u64_max]));
};
