const fs = require('fs');
const assert = require('assert');

exports.assert_comments_exist = function() {
  const bindings_file = require.resolve('wasm-bindgen-test');
  const contents = fs.readFileSync(bindings_file);
  assert.ok(contents.includes("* annotated function ✔️ \" \\ ' {"));
  assert.ok(contents.includes("* annotated struct type"));
  assert.ok(contents.includes("* annotated struct field b"));
  assert.ok(contents.includes("* annotated struct field c"));
  assert.ok(contents.includes("* annotated struct constructor"));
  assert.ok(contents.includes("* annotated struct method"));
  assert.ok(contents.includes("* annotated struct getter"));
  assert.ok(contents.includes("* annotated struct setter"));
  assert.ok(contents.includes("* annotated struct static method"));
  assert.ok(contents.includes("* annotated enum type"));
  assert.ok(contents.includes("* annotated enum variant 1"));
  assert.ok(contents.includes("* annotated enum variant 2"));
};
