const wasm = require("wasm-bindgen-test.js");

exports.create_garbage = async function () {
  for (let i = 0; i < 100; i++) {
    new wasm.OwnedValue(1).add(new wasm.OwnedValue(2)).n();
  }

  if ("gc" in global) {
    global.gc();
  } else {
    console.warn("test runner doesn't expose GC function");
  }
};
