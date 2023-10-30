const assert = require('assert');
const wasm = require('wasm-bindgen-test');

exports.test_add = function() {
  assert.strictEqual(wasm.simple_add(1, 2), 3);
  assert.strictEqual(wasm.simple_add(2, 3), 5);
  assert.strictEqual(wasm.simple_add3(2), 5);
  assert.strictEqual(wasm.simple_get2(true), 2);
  assert.strictEqual(wasm.simple_return_and_take_bool(true, false), false);
};

exports.test_string_arguments = function() {
  wasm.simple_assert_foo("foo");
  wasm.simple_assert_foo_and_bar("foo2", "bar");
};

exports.test_return_a_string = function() {
  assert.strictEqual(wasm.simple_clone("foo"), "foo");
  assert.strictEqual(wasm.simple_clone("another"), "another");
  assert.strictEqual(wasm.simple_concat("a", "b", 3), "a b 3");
  assert.strictEqual(wasm.simple_concat("c", "d", -2), "c d -2");
};

exports.test_wrong_types = function() {
  // this test only works when `--debug` is passed to `wasm-bindgen` (or the
  // equivalent thereof)
  if (require('process').env.WASM_BINDGEN_NO_DEBUG)
    return;
  assert.throws(() => wasm.simple_int('a'), /expected a number argument/);
  assert.throws(() => wasm.simple_str(3), /expected a string argument/);
};

exports.test_other_exports_still_available = function() {
  require('wasm-bindgen-test').__wasm.foo(3);
};

exports.test_jsvalue_typeof = function() {
  assert.ok(wasm.is_object({}));
  assert.ok(!wasm.is_object(42));
  assert.ok(wasm.is_function(function() {}));
  assert.ok(!wasm.is_function(42));
  assert.ok(wasm.is_string("2b or !2b"));
  assert.ok(!wasm.is_string(42));
};

exports.optional_str_none = function(x) {
  assert.strictEqual(x, undefined);
};

exports.optional_str_some = function(x) {
  assert.strictEqual(x, 'x');
};

exports.optional_slice_none = function(x) {
  assert.strictEqual(x, undefined);
};

exports.optional_slice_some = function(x) {
  assert.strictEqual(x.length, 3);
  assert.strictEqual(x[0], 1);
  assert.strictEqual(x[1], 2);
  assert.strictEqual(x[2], 3);
}

exports.optional_string_none = function(x) {
  assert.strictEqual(x, undefined);
};

exports.optional_string_some = function(x) {
  assert.strictEqual(x, 'abcd');
};

exports.optional_string_some_empty = function(x) {
  assert.strictEqual(x, '');
};

exports.return_string_none = function() {};
exports.return_string_some = function() {
  return 'foo';
};

exports.test_rust_optional = function() {
  wasm.take_optional_str_none();
  wasm.take_optional_str_none(null);
  wasm.take_optional_str_none(undefined);
  wasm.take_optional_str_some('hello');
  assert.strictEqual(wasm.return_optional_str_none(), undefined);
  assert.strictEqual(wasm.return_optional_str_some(), 'world');
};

exports.RenamedInRust = class {};
exports.new_renamed = () => new exports.RenamedInRust;

exports.import_export_same_name = () => {};

exports.test_string_roundtrip = () => {
  const test = s => {
    assert.strictEqual(wasm.do_string_roundtrip(s), s);
  };

  test('');
  test('a');
  test('💖');

  test('a longer string');
  test('a longer 💖 string');
};
