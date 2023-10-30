const assert = require('assert');
const wasm = require('wasm-bindgen-test');

var called = false;

exports.hit = function() {
  called = true;
};

exports.FOO = 1.0;

exports.test_works = function() {
  assert.strictEqual(called, true);

  var r = wasm.Foo.new();
  assert.strictEqual(r.add(0), 0);
  assert.strictEqual(r.add(1), 1);
  assert.strictEqual(r.add(2), 3);
  r.free();

  var r2 = wasm.Foo.with_contents(10);
  assert.strictEqual(r2.add(0), 10);
  assert.strictEqual(r2.add(1), 11);
  assert.strictEqual(r2.add(2), 13);
  r2.free();

  assert.strictEqual(wasm.Color.Green, 0);
  assert.strictEqual(wasm.Color.Yellow, 1);
  assert.strictEqual(wasm.Color.Red, 2);
  assert.strictEqual(wasm.Color[0], 'Green');
  assert.strictEqual(wasm.Color[1], 'Yellow');
  assert.strictEqual(wasm.Color[2], 'Red');
  assert.strictEqual(Object.keys(wasm.Color).length, 6);
  assert.strictEqual(wasm.cycle(wasm.Color.Green), wasm.Color.Yellow);

  wasm.node_math(1.0, 2.0);
};
