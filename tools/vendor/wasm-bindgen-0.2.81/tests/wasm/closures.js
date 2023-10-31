const assert = require('assert');
const wasm = require('wasm-bindgen-test');

exports.works_call = a => {
    a();
};

exports.works_thread = a => a(2);

let CANNOT_REUSE_CACHE = null;

exports.cannot_reuse_call = a => {
    CANNOT_REUSE_CACHE = a;
};

exports.cannot_reuse_call_again = () => {
    CANNOT_REUSE_CACHE();
};

exports.long_lived_call1 = a => {
    a();
};

exports.long_lived_call2 = a => a(2);

exports.many_arity_call1 = a => {
    a();
};
exports.many_arity_call2 = a => {
    a(1);
};
exports.many_arity_call3 = a => {
    a(1, 2);
};
exports.many_arity_call4 = a => {
    a(1, 2, 3);
};
exports.many_arity_call5 = a => {
    a(1, 2, 3, 4);
};
exports.many_arity_call6 = a => {
    a(1, 2, 3, 4, 5);
};
exports.many_arity_call7 = a => {
    a(1, 2, 3, 4, 5, 6);
};
exports.many_arity_call8 = a => {
    a(1, 2, 3, 4, 5, 6, 7);
};
exports.many_arity_call9 = a => {
    a(1, 2, 3, 4, 5, 6, 7, 8);
};

exports.option_call1 = a => {
    if (a) {
        a();
    }
};
exports.option_call2 = a => {
    if (a) {
        return a(2);
    }
};
exports.option_call3 = a => a == undefined;

let LONG_LIVED_DROPPING_CACHE = null;

exports.long_lived_dropping_cache = a => {
    LONG_LIVED_DROPPING_CACHE = a;
};
exports.long_lived_dropping_call = () => {
    LONG_LIVED_DROPPING_CACHE();
};

let LONG_LIVED_OPTION_DROPPING_CACHE = null;

exports.long_lived_option_dropping_cache = a => {
    if (a) {
        LONG_LIVED_OPTION_DROPPING_CACHE = a;
        return true;
    } else {
        return false;
    }
}
exports.long_lived_option_dropping_call = () => {
    LONG_LIVED_OPTION_DROPPING_CACHE();
}

let LONG_FNMUT_RECURSIVE_CACHE = null;

exports.long_fnmut_recursive_cache = a => {
    LONG_FNMUT_RECURSIVE_CACHE = a;
};
exports.long_fnmut_recursive_call = () => {
    LONG_FNMUT_RECURSIVE_CACHE();
};

exports.fnmut_call = a => {
    a();
};

exports.fnmut_thread = a => a(2);

let FNMUT_BAD_F = null;

exports.fnmut_bad_call = a => {
    FNMUT_BAD_F = a;
    a();
};

exports.fnmut_bad_again = x => {
    if (x) {
        FNMUT_BAD_F();
    }
};

exports.string_arguments_call = a => {
    a('foo');
};

exports.string_ret_call = a => {
    assert.strictEqual(a('foo'), 'foobar');
};

let DROP_DURING_CALL = null;
exports.drop_during_call_save = f => {
  DROP_DURING_CALL = f;
};
exports.drop_during_call_call = () => DROP_DURING_CALL();

exports.js_test_closure_returner = () => {
  wasm.closure_returner().someKey();
};

exports.calling_it_throws = a => {
  try {
    a();
    return false;
  } catch(_) {
    return true;
  }
};

exports.call_val = f => f();

exports.pass_reference_first_arg_twice = (a, b, c) => {
  b(a);
  c(a);
  a.free();
};

exports.call_destroyed = f => {
  assert.throws(f, /invoked recursively or destroyed/);
};

let FORGOTTEN_CLOSURE = null;

exports.js_store_forgotten_closure = f => {
  FORGOTTEN_CLOSURE = f;
};

exports.js_call_forgotten_closure = () => {
  FORGOTTEN_CLOSURE();
};
