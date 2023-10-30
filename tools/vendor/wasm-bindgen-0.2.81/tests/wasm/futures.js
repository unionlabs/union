const assert = require('assert');
const wasm = require('wasm-bindgen-test');

exports.call_exports = async function() {
  await wasm.async_do_nothing();
  assert.strictEqual(1, await wasm.async_return_1());
  assert.strictEqual(2, await wasm.async_return_2());
  await wasm.async_nothing_again();
  assert.strictEqual(3, await wasm.async_return_3());
  assert.strictEqual(4, await wasm.async_return_4());
  assert.strictEqual(5, (await wasm.async_return_5()).val);
  assert.strictEqual(6, (await wasm.async_return_6()).val);
  await assert.rejects(wasm.async_throw_7(), /7/);
  await assert.rejects(wasm.async_throw_custom(), /\[object Object\]/);
  await assert.rejects(wasm.async_throw_message(), /async message/);
  await assert.rejects(wasm.async_throw_jserror(), /async message/);
  await assert.rejects(wasm.async_throw_custom_error(), /custom error/);
};

exports.call_promise = async function() {
    return "ok";
}

exports.call_promise_ok = async function() {
    return "ok";
}

exports.call_promise_err = async function() {
    throw "error";
}

exports.call_promise_unit = async function() {
    console.log("asdfasdf");
}

exports.call_promise_ok_unit = async function() {
}

exports.call_promise_err_unit = async function() {
    throw "error";
}
