const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

let SIMPLE_ARG = null;

exports.simple_foo = s => {
    assert.strictEqual(SIMPLE_ARG, null);
    SIMPLE_ARG = s;
};

exports.js_simple = () => {
    assert.strictEqual(SIMPLE_ARG, null);
    let sym = Symbol('test');
    wasm.simple_bar(sym);
    assert.strictEqual(SIMPLE_ARG, sym);
};

let OWNED_ARG = null;

exports.owned_foo = s => {
    assert.strictEqual(OWNED_ARG, null);
    OWNED_ARG = s;
};

exports.js_owned = () => {
    assert.strictEqual(OWNED_ARG, null);
    let sym = Symbol('test');
    wasm.owned_bar(sym);
    assert.strictEqual(OWNED_ARG, sym);
};

let CLONE_ARG = Symbol('test');

exports.clone_foo1 = s => {
    assert.strictEqual(s, CLONE_ARG);
};
exports.clone_foo2 = s => {
    assert.strictEqual(s, CLONE_ARG);
};
exports.clone_foo3 = s => {
    assert.strictEqual(s, CLONE_ARG);
};
exports.clone_foo4 = s => {
    assert.strictEqual(s, CLONE_ARG);
};
exports.clone_foo5 = s => {
    assert.strictEqual(s, CLONE_ARG);
};

exports.js_clone = () => {
    wasm.clone_bar(CLONE_ARG);
};


let PROMOTE_ARG = Symbol('test');

exports.promote_foo1 = s => {
    assert.strictEqual(s, PROMOTE_ARG);
};
exports.promote_foo2 = s => {
    assert.strictEqual(s, PROMOTE_ARG);
};
exports.promote_foo3 = s => {
    assert.strictEqual(s, PROMOTE_ARG);
};
exports.promote_foo4 = s => {
    assert.strictEqual(s, PROMOTE_ARG);
};

exports.js_promote = () => {
    wasm.promote_bar(PROMOTE_ARG);
};

exports.returning_vector_foo = () => {
    return {'foo': 'bar'};
};

exports.js_returning_vector = () => {
    assert.strictEqual(wasm.returning_vector_bar().length, 10);
};

exports.js_another_vector_return = () => {
    assert.deepStrictEqual(wasm.another_vector_return_get_array(), [1, 2, 3, 4, 5, 6]);
};

exports.returning_vector_string_foo = () => {
    return "This is the mostest awesomest string that can possibly exist.";
};

exports.js_returning_vector_string = () => {
    assert.strictEqual(wasm.returning_vector_string_bar().length, 10);
};

exports.js_another_vector_string_return = () => {
    assert.deepStrictEqual(wasm.another_vector_string_return_get_array(), ["1", "2", "3", "4", "5", "6"]);
};

exports.verify_serde = function(a) {
  assert.deepStrictEqual(a, {
    a: 0,
    b: 'foo',
    c: null,
    d: { a: 1 }
  });

  return {
    a: 2,
    b: 'bar',
    c: { a: 3 },
    d: { a: 4 },
  }
};
