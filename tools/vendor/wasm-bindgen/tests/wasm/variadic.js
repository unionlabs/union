const assert = require('assert');

// a function for testing numbers
function variadic_sum(...args) {
    let answer = 0;
    for(var i=0; i<args.length; i++) {
        answer += args[i];
    }
    return answer;
}

exports.variadic_sum_u8 = variadic_sum;
exports.variadic_sum_u16 = variadic_sum;
exports.variadic_sum_u32 = variadic_sum;
exports.variadic_sum_u64 = variadic_sum;
exports.variadic_sum_i8 = variadic_sum;
exports.variadic_sum_i16 = variadic_sum;
exports.variadic_sum_i32 = variadic_sum;
exports.variadic_sum_i64 = variadic_sum;
exports.variadic_sum_f32 = variadic_sum;
exports.variadic_sum_f64 = variadic_sum;
exports.variadic_sum_rest_vec = variadic_sum;

// a function for testing nullable numbers
function variadic_sum_opt(...args) {
    let answer = 0;
    for(var i=0; i<args.length; i++) {
        if(args[i] != null) {
            answer += args[i];
        }
    }
    return answer;
}

exports.variadic_sum_opt = variadic_sum_opt;

// a function for testing strings
function variadic_concat(...args) {
    let answer = "";
    for(var i=0; i<args.length; i++) {
        answer = `${answer}${args[i]}`;
    }
    return answer;
}

exports.variadic_concat_str = variadic_concat;
exports.variadic_concat_string = variadic_concat;

// a test that takes any type of arguments (for testing JsValue).
function variadic_compare_pairs(...args) {
    assert(args.length % 2 == 0, "args must be sequence of pairs");
    for(var i=0; i<args.length; i++) {
        const first = args[i++];
        const second = args[i];
        assert.equal(first, second);
    }
}

exports.variadic_compare_pairs = variadic_compare_pairs;
