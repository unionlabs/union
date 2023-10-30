const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.js_simple = () => {
    const r = new wasm.ClassesSimple();
    assert.strictEqual(r.add(0), 0);
    assert.strictEqual(r.add(1), 1);
    assert.strictEqual(r.add(1), 2);
    r.add(2);
    assert.strictEqual(r.consume(), 4);
    assert.throws(() => r.free(), /null pointer passed to rust/);

    const r2 = wasm.ClassesSimple.with_contents(10);
    assert.strictEqual(r2.add(1), 11);
    assert.strictEqual(r2.add(2), 13);
    assert.strictEqual(r2.add(3), 16);
    r2.free();

    const r3 = new wasm.ClassesSimple();
    assert.strictEqual(r3.add(42), 42);
    r3.free();
};

exports.js_strings = () => {
    const r = wasm.ClassesStrings1.new();
    r.set(3);
    let bar = r.bar('baz');
    r.free();
    assert.strictEqual(bar.name(), 'foo-baz-3');
    bar.free();
};

exports.js_exceptions = () => {
    // this test only works when `--debug` is passed to `wasm-bindgen` (or the
    // equivalent thereof)
    if (require('process').env.WASM_BINDGEN_NO_DEBUG)
        return;
    assert.throws(() => new wasm.ClassesExceptions1(), /cannot invoke `new` directly/);
    let a = wasm.ClassesExceptions1.new();
    a.free();
    assert.throws(() => a.free(), /null pointer passed to rust/);

    let b = wasm.ClassesExceptions1.new();
    b.foo(b);
    assert.throws(() => b.bar(b), /recursive use of an object/);
    // TODO: throws because it tries to borrow_mut, but the throw_str from the previous line doesn't clean up the
    // RefMut so the object is left in a broken state.
    // We still try to call free here so the object is removed from the FinalizationRegistry when weak refs are enabled.
    assert.throws(() => b.free(), /recursive use of an object/);

    let c = wasm.ClassesExceptions1.new();
    let d = wasm.ClassesExceptions2.new();
    assert.throws(() => c.foo(d), /expected instance of ClassesExceptions1/);
    d.free();
    c.free();
};

exports.js_pass_one_to_another = () => {
    let a = wasm.ClassesPassA.new();
    let b = wasm.ClassesPassB.new();
    a.foo(b);
    a.bar(b);
    a.free();
};

exports.take_class = foo => {
    assert.strictEqual(foo.inner(), 13);
    foo.free();
    assert.throws(() => foo.free(), /null pointer passed to rust/);
};

exports.js_constructors = () => {
    const foo = new wasm.ConstructorsFoo(1);
    assert.strictEqual(foo.get_number(), 1);
    foo.free();

    assert.strictEqual(wasm.ConstructorsBar.new, undefined);
    const foo2 = new wasm.ConstructorsFoo(2);
    assert.strictEqual(foo2.get_number(), 2);
    foo2.free();

    const bar = new wasm.ConstructorsBar(3, 4);
    assert.strictEqual(bar.get_sum(), 7);
    bar.free();

    assert.strictEqual(wasm.ConstructorsBar.other_name, undefined);
    const bar2 = new wasm.ConstructorsBar(5, 6);
    assert.strictEqual(bar2.get_sum(), 11);
    bar2.free();

    assert.strictEqual(wasm.cross_item_construction().get_sum(), 15);
};

exports.js_empty_structs = () => {
    wasm.OtherEmpty.return_a_value();
};

exports.js_public_fields = () => {
    const a = wasm.PublicFields.new();
    assert.strictEqual(a.a, 0);
    a.a = 3;
    assert.strictEqual(a.a, 3);

    assert.strictEqual(a.b, 0);
    a.b = 7;
    assert.strictEqual(a.b, 7);

    assert.strictEqual(a.c, 0);
    a.c = 8;
    assert.strictEqual(a.c, 8);

    assert.strictEqual(a.d, 0);
    a.d = 3.3;
    assert.strictEqual(a.d, 3);

    assert.strictEqual(a.skipped, undefined);
};

exports.js_getter_with_clone = () => {
    const a = wasm.GetterWithCloneStruct.new();
    assert.strictEqual(a.a, '');
    a.a = 'foo';
    assert.strictEqual(a.a, 'foo');

    const b = wasm.GetterWithCloneStructField.new();
    assert.strictEqual(b.a, '');
    b.a = 'foo';
    assert.strictEqual(b.a, 'foo');
};

exports.js_using_self = () => {
    wasm.UseSelf.new().free();
};

exports.js_readonly_fields = () => {
    const a = wasm.Readonly.new();
    assert.strictEqual(a.a, 0);
    a.a = 3;
    assert.strictEqual(a.a, 0);
    a.free();
};

exports.js_double_consume = () => {
    const r = new wasm.DoubleConsume();
    assert.throws(() => r.consume(r));
};


exports.js_js_rename = () => {
    (new wasm.JsRename()).bar();
    wasm.classes_foo();
};

exports.js_access_fields = () => {
    assert.ok((new wasm.AccessFieldFoo()).bar instanceof wasm.AccessFieldBar);
    assert.ok((new wasm.AccessField0())[0] instanceof wasm.AccessFieldBar);
};

exports.js_renamed_export = () => {
    const x = new wasm.JsRenamedExport();
    assert.ok(x.x === 3);
    x.foo();
    x.bar(x);
};

exports.js_renamed_field = () => {
    const x = new wasm.RenamedField();
    assert.ok(x.bar === 3);

    x.foo();
}

exports.js_conditional_bindings = () => {
    const x = new wasm.ConditionalBindings();
    x.free();
};

exports.js_assert_none = x => {
  assert.strictEqual(x, undefined);
};
exports.js_assert_some = x => {
  assert.ok(x instanceof wasm.OptionClass);
};
exports.js_return_none1 = () => null;
exports.js_return_none2 = () => undefined;
exports.js_return_some = x => x;

exports.js_test_option_classes = () => {
  assert.strictEqual(wasm.option_class_none(), undefined);
  wasm.option_class_assert_none(undefined);
  wasm.option_class_assert_none(null);
  const c = wasm.option_class_some();
  assert.ok(c instanceof wasm.OptionClass);
  wasm.option_class_assert_some(c);
};

/**
 * Invokes `console.log`, but logs to a string rather than stdout
 * @param {any} data Data to pass to `console.log`
 * @returns {string} Output from `console.log`, without color or trailing newlines
 */
const console_log_to_string = data => {
    // Store the original stdout.write and create a console that logs without color
    const original_write = process.stdout.write;
    const colorless_console = new console.Console({
      stdout: process.stdout,
      colorMode: false
    });
    let output = '';

    // Change stdout.write to append to our string, then restore the original function
    process.stdout.write = chunk => output += chunk.trim();
    colorless_console.log(data);
    process.stdout.write = original_write;

    return output;
};

exports.js_test_inspectable_classes = () => {
    const inspectable = wasm.Inspectable.new();
    const not_inspectable = wasm.NotInspectable.new();
    // Inspectable classes have a toJSON and toString implementation generated
    assert.deepStrictEqual(inspectable.toJSON(), { a: inspectable.a });
    assert.strictEqual(inspectable.toString(), `{"a":${inspectable.a}}`);
    // Inspectable classes in Node.js have improved console.log formatting as well
    assert(console_log_to_string(inspectable).endsWith(`{ a: ${inspectable.a} }`));
    // Non-inspectable classes do not have a toJSON or toString generated
    assert.strictEqual(not_inspectable.toJSON, undefined);
    assert.strictEqual(not_inspectable.toString(), '[object Object]');
    // Non-inspectable classes in Node.js have no special console.log formatting
    assert.strictEqual(console_log_to_string(not_inspectable), `NotInspectable { __wbg_ptr: ${not_inspectable.__wbg_ptr} }`);
    inspectable.free();
    not_inspectable.free();
};

exports.js_test_inspectable_classes_can_override_generated_methods = () => {
    const overridden_inspectable = wasm.OverriddenInspectable.new();
    // Inspectable classes can have the generated toJSON and toString overwritten
    assert.strictEqual(overridden_inspectable.a, 0);
    assert.deepStrictEqual(overridden_inspectable.toJSON(), 'JSON was overwritten');
    assert.strictEqual(overridden_inspectable.toString(), 'string was overwritten');
    overridden_inspectable.free();
};
