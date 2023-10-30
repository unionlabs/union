const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

const ARGUMENT_NAMES = /([^\s,]+)/g;
const STRIP_COMMENTS = /((\/\/.*$)|(\/\*[\s\S]*?\*\/))/mg;

// https://stackoverflow.com/q/1007981/210304
function getArgNames(func) {
    let fnStr = func.toString().replace(STRIP_COMMENTS, '');
    let result = fnStr.slice(fnStr.indexOf('(')+1, fnStr.indexOf(')')).match(ARGUMENT_NAMES);
    return result === null ? [] : result;
}

exports.js_arg_names = () => {
    assert.deepEqual(getArgNames(wasm.fn_with_many_args), ['_a', '_b', '_c', '_d']);
};
