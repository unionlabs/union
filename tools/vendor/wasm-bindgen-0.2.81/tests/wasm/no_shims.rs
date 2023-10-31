//! A collection of tests to exercise imports where we don't need to generate a
//! JS shim to convert arguments/returns even when Web IDL bindings is not
//! implemented.

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(inline_js = "
    function assert_eq(a, b) {
        if (a !== b) {
            throw new Error(`assert_eq failed: ${a} != ${b}`);
        }
    }

    module.exports.trivial = function () {};

    module.exports.incoming_bool = function () { return true; };
    module.exports.incoming_u8 = function () { return 255; };
    module.exports.incoming_i8 = function () { return -127; };
    module.exports.incoming_u16 = function () { return 65535; };
    module.exports.incoming_i16 = function () { return 32767; };
    module.exports.incoming_u32 = function () { return 4294967295; };
    module.exports.incoming_i32 = function () { return 0; };
    module.exports.incoming_f32 = function () { return 1.5; };
    module.exports.incoming_f64 = function () { return 13.37; };

    module.exports.outgoing_u8 = function (k) { assert_eq(k, 255); };
    module.exports.outgoing_i8 = function (i) { assert_eq(i, -127); };
    module.exports.outgoing_u16 = function (l) { assert_eq(l, 65535); };
    module.exports.outgoing_i16 = function (j) { assert_eq(j, 32767); };
    module.exports.outgoing_i32 = function (x) { assert_eq(x, 0); };
    module.exports.outgoing_f32 = function (y) { assert_eq(y, 1.5); };
    module.exports.outgoing_f64 = function (z) { assert_eq(z, 13.37); };

    module.exports.many = function (x, y, z) {
        assert_eq(x, 0);
        assert_eq(y, 1.5);
        assert_eq(z, 13.37);
        return 42;
    };

    module.exports.works_when_externref_support_is_enabled = function (v) {
        assert_eq(v, 'hello');
        return v;
    };

    module.exports.MyNamespace = {};
    module.exports.MyNamespace.incoming_namespaced = function () { return 3.14; };
    module.exports.MyNamespace.outgoing_namespaced = function (pi) { assert_eq(3.14, pi); };
")]
extern "C" {
    #[wasm_bindgen(assert_no_shim)]
    fn trivial();

    #[wasm_bindgen(assert_no_shim)]
    fn incoming_bool() -> bool;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_u8() -> u8;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_i8() -> i8;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_u16() -> u16;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_i16() -> i16;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_u32() -> u32;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_i32() -> i32;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_f32() -> f32;
    #[wasm_bindgen(assert_no_shim)]
    fn incoming_f64() -> f64;

    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_u8(k: u8);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_i8(i: i8);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_u16(l: u16);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_i16(j: i16);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_i32(x: i32);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_f32(y: f32);
    #[wasm_bindgen(assert_no_shim)]
    fn outgoing_f64(z: f64);

    #[wasm_bindgen(assert_no_shim)]
    fn many(x: i32, y: f32, z: f64) -> i32;

    #[wasm_bindgen(assert_no_shim, js_namespace = MyNamespace)]
    fn incoming_namespaced() -> f64;
    #[wasm_bindgen(assert_no_shim, js_namespace = MyNamespace)]
    fn outgoing_namespaced(x: f64);

    // Note that this should only skip the JS shim if we have externref support
    // enabled.
    //
    // #[wasm_bindgen(assert_no_shim)]
    fn works_when_externref_support_is_enabled(v: JsValue) -> JsValue;
}

#[wasm_bindgen_test]
fn no_shims() {
    trivial();

    let k = incoming_u8();
    assert_eq!(k, 255);
    outgoing_u8(k);

    let l = incoming_u16();
    assert_eq!(l, 65535);
    outgoing_u16(l);

    let m = incoming_u32();
    assert_eq!(m, 4294967295);

    let i = incoming_i8();
    assert_eq!(i, -127);
    outgoing_i8(i);

    let j = incoming_i16();
    assert_eq!(j, 32767);
    outgoing_i16(j);

    let x = incoming_i32();
    assert_eq!(x, 0);
    outgoing_i32(x);

    let y = incoming_f32();
    assert_eq!(y, 1.5);
    outgoing_f32(y);

    let z = incoming_f64();
    assert_eq!(z, 13.37);
    outgoing_f64(z);

    let w = many(x, y, z);
    assert_eq!(w, 42);

    let pi = incoming_namespaced();
    assert_eq!(pi, 3.14);
    outgoing_namespaced(pi);

    let b = incoming_bool();
    assert!(b);

    let v = JsValue::from("hello");
    let vv = works_when_externref_support_is_enabled(v.clone());
    assert_eq!(v, vv);
}
