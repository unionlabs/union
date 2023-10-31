const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.call_throw_one = function() {
  try {
    wasm.throw_one();
  } catch (e) {
    assert.strictEqual(e, 1);
  }
};

exports.call_ok = function() {
  wasm.nothrow();
};
