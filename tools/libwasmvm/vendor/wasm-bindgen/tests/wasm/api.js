const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.assert_null = x => {
    assert.strictEqual(x, null);
};

exports.js_works = () => {
    assert.strictEqual(wasm.api_foo(), 'foo');
    assert.strictEqual(wasm.api_bar('a'), 'a');
    assert.strictEqual(wasm.api_baz(), 1);
    wasm.api_baz2(2, 'a');

    assert.strictEqual(wasm.api_js_null(), null);
    assert.strictEqual(wasm.api_js_undefined(), undefined);

    wasm.api_test_is_null_undefined(null, undefined, 1.0);

    assert.strictEqual(wasm.api_get_true(), true);
    assert.strictEqual(wasm.api_get_false(), false);
    wasm.api_test_bool(true, false, 1.0);

    assert.strictEqual(typeof (wasm.api_mk_symbol()), 'symbol');
    assert.strictEqual(typeof (wasm.api_mk_symbol2('a')), 'symbol');
    assert.strictEqual(Symbol.keyFor(wasm.api_mk_symbol()), undefined);
    assert.strictEqual(Symbol.keyFor(wasm.api_mk_symbol2('b')), undefined);

    wasm.api_assert_symbols(Symbol(), 'a');
    wasm.api_acquire_string('foo', null);
    assert.strictEqual(wasm.api_acquire_string2(''), '');
    assert.strictEqual(wasm.api_acquire_string2('a'), 'a');

    let arr = [1, 2, 3, 4, {}, ['a', 'b', 'c']]
    wasm.api_completely_variadic(...arr).forEach((element, index) => {
        assert.strictEqual(element, arr[index]);
    });
    assert.strictEqual(
        wasm.api_completely_variadic().length,
        0
    );
    wasm.api_variadic_with_prefixed_params([], {}, ...arr).forEach((element, index) => {
        assert.strictEqual(element, arr[index]);
    });
};

exports.js_eq_works = () => {
    assert.strictEqual(wasm.eq_test('a', 'a'), true);
    assert.strictEqual(wasm.eq_test('a', 'b'), false);
    assert.strictEqual(wasm.eq_test(NaN, NaN), false);
    assert.strictEqual(wasm.eq_test({ a: 'a' }, { a: 'a' }), false);
    assert.strictEqual(wasm.eq_test1(NaN), false);
    let x = { a: 'a' };
    assert.strictEqual(wasm.eq_test(x, x), true);
    assert.strictEqual(wasm.eq_test1(x), true);
};

exports.debug_values = () => ([
    null,
    undefined,
    0,
    1.0,
    true,
    [1, 2, 3],
    "string",
    { test: "object" },
    [1.0, [2.0, 3.0]],
    () => (null),
    new Set(),
]);

exports.assert_function_table = (x, i) => {
    const rawWasm = require('wasm-bindgen-test.js').__wasm;
    assert.ok(x instanceof WebAssembly.Table);
    assert.strictEqual(x.get(i), rawWasm.function_table_lookup);
};
