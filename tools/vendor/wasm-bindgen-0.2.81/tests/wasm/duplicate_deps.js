const assert = require('assert');

let next = null;

exports.assert_next_undefined = function() {
  next = undefined;
};

exports.assert_next_ten = function() {
  next = 10;
};

exports.foo = function(a) {
  console.log(a, next);
  assert.strictEqual(a, next);
  next = null;
};
