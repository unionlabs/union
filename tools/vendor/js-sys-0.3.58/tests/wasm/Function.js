// Used for `Function.rs` tests
exports.get_function_to_bind = function() {
  return function() { return this.x || 1; }
};
exports.get_value_to_bind_to = function() {
  return { x: 2 };
};
exports.list = function() {
  return function() {return Array.prototype.slice.call(arguments);}
};
exports.add_arguments = function() {
    return function(arg1, arg2) {return arg1 + arg2}
};
exports.call_function = function(f) {
  return f();
};
exports.call_function_arg =  function(f, arg1) {
  return f(arg1);
};