const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

class MyType {
}

exports.MyType = MyType;

exports.take_none_byval = x => {
    assert.strictEqual(x, undefined);
};
exports.take_some_byval = x => {
    assert.ok(x !== null && x !== undefined);
    assert.ok(x instanceof MyType);
};
exports.return_undef_byval = () => undefined;
exports.return_null_byval = () => null;
exports.return_some_byval = () => new MyType();

exports.test_option_values = () => {
    wasm.rust_take_none_byval(null);
    wasm.rust_take_none_byval(undefined);
    wasm.rust_take_some_byval(new MyType());
    assert.strictEqual(wasm.rust_return_none_byval(), undefined);
    const x = wasm.rust_return_some_byval();
    assert.ok(x !== null && x !== undefined);
    assert.ok(x instanceof MyType);
};
