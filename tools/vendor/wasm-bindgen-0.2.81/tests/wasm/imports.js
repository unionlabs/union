const assert = require('assert');
const wasm = require('wasm-bindgen-test');
const fs = require('fs');

let ARG = null;
let ANOTHER_ARG = null;
let SYM = Symbol('a');

exports.simple_foo = function(s) {
  assert.strictEqual(ARG, null);
  assert.strictEqual(s, "foo");
  ARG = s;
};

exports.simple_another = function(s) {
  assert.strictEqual(ANOTHER_ARG, null);
  assert.strictEqual(s, 21);
  ANOTHER_ARG = s;
  return 35;
};

exports.simple_take_and_return_bool = function(s) {
  return s;
};
exports.simple_return_object = function() {
  return SYM;
};
exports.test_simple = function() {
  assert.strictEqual(ARG, null);
  wasm.simple_take_str("foo");
  assert.strictEqual(ARG, "foo");

  assert.strictEqual(ANOTHER_ARG, null);
  assert.strictEqual(wasm.simple_another_thunk(21), 35);
  assert.strictEqual(ANOTHER_ARG, 21);

  assert.strictEqual(wasm.simple_bool_thunk(true), true);
  assert.strictEqual(wasm.simple_bool_thunk(false), false);

  assert.strictEqual(wasm.simple_get_the_object(), SYM);
};

exports.return_string = function() {
  return 'bar';
};

exports.take_and_ret_string = function(a) {
  return a + 'b';
};

exports.exceptions_throw = function() {
  throw new Error('error!');
};
exports.exceptions_throw2 = function() {
  throw new Error('error2');
};
exports.test_exception_propagates = function() {
  assert.throws(wasm.exceptions_propagate, /error!/);
};

exports.assert_valid_error = function(obj) {
  assert.strictEqual(obj instanceof Error, true);
  assert.strictEqual(obj.message, 'error2');
};

exports.IMPORT = 1.0;

exports.return_three = function() { return 3; };

exports.underscore = function(x) {};

exports.pub = function() { return 2; };

exports.bar = { foo: 3 };

let CUSTOM_TYPE = null;

exports.take_custom_type = function(f) {
  CUSTOM_TYPE = f;
  return f;
};

exports.custom_type_return_2 = function() {
  return 2;
};

exports.touch_custom_type = function() {
  assert.throws(() => CUSTOM_TYPE.touch(),
    /Attempt to use a moved value|null pointer passed to rust/);
};

exports.interpret_2_as_custom_type = function() {
  assert.throws(wasm.interpret_2_as_custom_type, /expected instance of CustomType/);
};

exports.baz$ = function() {};
exports.$foo = 1.0;

exports.assert_dead_import_not_generated = function() {
  const filename = require.resolve("wasm-bindgen-test");
  const bindings = fs.readFileSync(filename);
  assert.ok(!bindings.includes("unused_import"));
};

exports.import_inside_function_works = function() {};
exports.import_inside_private_module = function() {};
exports.should_call_undefined_functions = () => false;

exports.STATIC_STRING = 'x';

class StaticMethodCheck {
  static static_method_of_right_this() {
    assert.ok(this === StaticMethodCheck);
  }
}

exports.StaticMethodCheck = StaticMethodCheck;

exports.receive_undefined = val => {
  assert.strictEqual(val, undefined);
};

const VAL = {};

exports.receive_some = val => {
  assert.strictEqual(val, VAL);
};

exports.get_some_val = () => VAL;

exports.Math = {
  func_from_module_math: (a) => a * 2
}

exports.Number = {
  func_from_module_number: () => 3.0
}

exports.same_name_from_import = (a) => a * 3;

exports.same_js_namespace_from_module = {
  func_from_module_1_same_js_namespace: (a) => a * 5
}