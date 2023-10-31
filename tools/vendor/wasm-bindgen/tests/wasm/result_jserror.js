const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

exports.call_ok = function() {
    assert.doesNotThrow(() => {
        let five = wasm.return_ok();
        assert.strictEqual(five, 5);
    })
}

exports.call_err = function() {
    assert.throws(() => wasm.return_err(), {
        message: "MyError::Variant"
    });
}

exports.call_make_an_error = function() {
    assert.doesNotThrow(() => {
        let e = wasm.make_an_error()
        assert.strictEqual(e.message, "un-thrown error");
    });
}

function check_inflight(struct) {
    assert.strictEqual(struct.is_inflight(), false);
}

exports.all_struct_methods = function() {
    let struct;
    assert.throws(() => wasm.MyStruct.new_err(), {
        message: "MyError::Variant"
    });
    assert.doesNotThrow(() => {
        struct = wasm.MyStruct.new();
    });
    check_inflight(struct);
    assert.doesNotThrow(() => {
        let five = struct.return_ok();
        assert.strictEqual(five, 5);
    });
    check_inflight(struct);
    assert.throws(() => struct.return_err(), {
        message: "MyError::Variant"
    });
    check_inflight(struct);
}

exports.call_return_string = function() {
    assert.doesNotThrow(() => {
        let ok = wasm.jserror_return_string();
        assert.strictEqual(ok, "string here");
    })
}

