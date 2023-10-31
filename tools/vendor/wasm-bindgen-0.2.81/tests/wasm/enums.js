const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.js_c_style_enum = () => {
    assert.strictEqual(wasm.Color.Green, 0);
    assert.strictEqual(wasm.Color.Yellow, 1);
    assert.strictEqual(wasm.Color.Red, 2);
    assert.strictEqual(wasm.Color[0], 'Green');
    assert.strictEqual(wasm.Color[1], 'Yellow');
    assert.strictEqual(wasm.Color[2], 'Red');
    assert.strictEqual(Object.keys(wasm.Color).length, 6);

    assert.strictEqual(wasm.enum_cycle(wasm.Color.Green), wasm.Color.Yellow);
};

exports.js_c_style_enum_with_custom_values = () => {
    assert.strictEqual(wasm.ColorWithCustomValues.Green, 21);
    assert.strictEqual(wasm.ColorWithCustomValues.Yellow, 34);
    assert.strictEqual(wasm.ColorWithCustomValues.Red, 2);
    assert.strictEqual(wasm.ColorWithCustomValues[21], 'Green');
    assert.strictEqual(wasm.ColorWithCustomValues[34], 'Yellow');
    assert.strictEqual(wasm.ColorWithCustomValues[2], 'Red');
    assert.strictEqual(Object.keys(wasm.ColorWithCustomValues).length, 6);

    assert.strictEqual(wasm.enum_with_custom_values_cycle(wasm.ColorWithCustomValues.Green), wasm.ColorWithCustomValues.Yellow);
};

exports.js_handle_optional_enums = x => wasm.handle_optional_enums(x);

exports.js_expect_enum = (a, b) => {
  assert.strictEqual(a, b);
};

exports.js_expect_enum_none = a => {
  assert.strictEqual(a, undefined);
};

exports.js_renamed_enum = b => {
  assert.strictEqual(wasm.JsRenamedEnum.B, b);
};
