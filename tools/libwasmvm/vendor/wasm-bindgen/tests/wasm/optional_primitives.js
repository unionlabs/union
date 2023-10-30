const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.optional_i32_js_identity = a => a;
exports.optional_u32_js_identity = a => a;
exports.optional_isize_js_identity = a => a;
exports.optional_usize_js_identity = a => a;
exports.optional_f32_js_identity = a => a;
exports.optional_f64_js_identity = a => a;
exports.optional_i8_js_identity = a => a;
exports.optional_u8_js_identity = a => a;
exports.optional_i16_js_identity = a => a;
exports.optional_u16_js_identity = a => a;
exports.optional_i64_js_identity = a => a;
exports.optional_u64_js_identity = a => a;
exports.optional_bool_js_identity = a => a;
exports.optional_char_js_identity = a => a;

exports.js_works = () => {
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_none()), undefined);
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_zero()), 0);
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_one()), 1);
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_neg_one()), -1);
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_min()), -2147483648);
    assert.strictEqual(wasm.optional_i32_identity(wasm.optional_i32_max()), 2147483647);

    assert.strictEqual(wasm.optional_u32_identity(wasm.optional_u32_none()), undefined);
    assert.strictEqual(wasm.optional_u32_identity(wasm.optional_u32_zero()), 0);
    assert.strictEqual(wasm.optional_u32_identity(wasm.optional_u32_one()), 1);
    assert.strictEqual(wasm.optional_u32_identity(wasm.optional_u32_min()), 0);
    assert.strictEqual(wasm.optional_u32_identity(wasm.optional_u32_max()), 4294967295);

    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_none()), undefined);
    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_zero()), 0);
    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_one()), 1);
    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_neg_one()), -1);
    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_min()), -2147483648);
    assert.strictEqual(wasm.optional_isize_identity(wasm.optional_isize_max()), 2147483647);

    assert.strictEqual(wasm.optional_usize_identity(wasm.optional_usize_none()), undefined);
    assert.strictEqual(wasm.optional_usize_identity(wasm.optional_usize_zero()), 0);
    assert.strictEqual(wasm.optional_usize_identity(wasm.optional_usize_one()), 1);
    assert.strictEqual(wasm.optional_usize_identity(wasm.optional_usize_min()), 0);
    assert.strictEqual(wasm.optional_usize_identity(wasm.optional_usize_max()), 4294967295);

    assert.strictEqual(wasm.optional_f32_identity(wasm.optional_f32_none()), undefined);
    assert.strictEqual(wasm.optional_f32_identity(wasm.optional_f32_zero()), 0);
    assert.strictEqual(wasm.optional_f32_identity(wasm.optional_f32_one()), 1);
    assert.strictEqual(wasm.optional_f32_identity(wasm.optional_f32_neg_one()), -1);

    assert.strictEqual(wasm.optional_f64_identity(wasm.optional_f64_none()), undefined);
    assert.strictEqual(wasm.optional_f64_identity(wasm.optional_f64_zero()), 0);
    assert.strictEqual(wasm.optional_f64_identity(wasm.optional_f64_one()), 1);
    assert.strictEqual(wasm.optional_f64_identity(wasm.optional_f64_neg_one()), -1);

    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_none()), undefined);
    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_zero()), 0);
    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_one()), 1);
    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_neg_one()), -1);
    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_min()), -128);
    assert.strictEqual(wasm.optional_i8_identity(wasm.optional_i8_max()), 127);

    assert.strictEqual(wasm.optional_u8_identity(wasm.optional_u8_none()), undefined);
    assert.strictEqual(wasm.optional_u8_identity(wasm.optional_u8_zero()), 0);
    assert.strictEqual(wasm.optional_u8_identity(wasm.optional_u8_one()), 1);
    assert.strictEqual(wasm.optional_u8_identity(wasm.optional_u8_min()), 0);
    assert.strictEqual(wasm.optional_u8_identity(wasm.optional_u8_max()), 255);

    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_none()), undefined);
    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_zero()), 0);
    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_one()), 1);
    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_neg_one()), -1);
    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_min()), -32768);
    assert.strictEqual(wasm.optional_i16_identity(wasm.optional_i16_max()), 32767);

    assert.strictEqual(wasm.optional_u16_identity(wasm.optional_u16_none()), undefined);
    assert.strictEqual(wasm.optional_u16_identity(wasm.optional_u16_zero()), 0);
    assert.strictEqual(wasm.optional_u16_identity(wasm.optional_u16_one()), 1);
    assert.strictEqual(wasm.optional_u16_identity(wasm.optional_u16_min()), 0);
    assert.strictEqual(wasm.optional_u16_identity(wasm.optional_u16_max()), 65535);

    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_none()), undefined);
    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_zero()), BigInt('0'));
    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_one()), BigInt('1'));
    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_neg_one()), BigInt('-1'));
    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_min()), BigInt('-9223372036854775808'));
    assert.strictEqual(wasm.optional_i64_identity(wasm.optional_i64_max()), BigInt('9223372036854775807'));

    assert.strictEqual(wasm.optional_u64_identity(wasm.optional_u64_none()), undefined);
    assert.strictEqual(wasm.optional_u64_identity(wasm.optional_u64_zero()), BigInt('0'));
    assert.strictEqual(wasm.optional_u64_identity(wasm.optional_u64_one()), BigInt('1'));
    assert.strictEqual(wasm.optional_u64_identity(wasm.optional_u64_min()), BigInt('0'));
    assert.strictEqual(wasm.optional_u64_identity(wasm.optional_u64_max()), BigInt('18446744073709551615'));

    assert.strictEqual(wasm.optional_bool_identity(wasm.optional_bool_none()), undefined);
    assert.strictEqual(wasm.optional_bool_identity(wasm.optional_bool_false()), false);
    assert.strictEqual(wasm.optional_bool_identity(wasm.optional_bool_true()), true);

    assert.strictEqual(wasm.optional_char_identity(wasm.optional_char_none()), undefined);
    assert.strictEqual(wasm.optional_char_identity(wasm.optional_char_letter()), 'a');
    assert.strictEqual(wasm.optional_char_identity(wasm.optional_char_face()), '😀');
};
