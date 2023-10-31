const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.js_works = () => {
    let called = false;
    wasm.run({
        bar() {
            called = true;
        },
        baz: 1,
    });
    assert.strictEqual(called, true);
};
