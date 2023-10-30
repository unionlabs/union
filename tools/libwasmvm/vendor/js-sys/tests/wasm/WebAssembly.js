const { TextEncoder } = require("util");

const data =
  "\u0000asm\u0001\u0000\u0000\u0000\u0001\b\u0002`\u0001\u007f\u0000`\u0000" +
  "\u0000\u0002\u0019\u0001\u0007imports\rimported_func\u0000\u0000\u0003" +
  "\u0002\u0001\u0001\u0007\u0011\u0001\rexported_func\u0000\u0001\n\b\u0001" +
  "\u0006\u0000A*\u0010\u0000\u000b";

const encoder = new TextEncoder();
const wasmArray = encoder.encode(data);

function getWasmArray() {
  return wasmArray;
}

function getTableObject() {
  return { element: "anyfunc", initial: 1 }
}

function getInvalidTableObject() {
  return { element: "anyfunc", initial: 1, maximum: 0 }
}

function getImports() {
  return {
    imports: {
      imported_func: function () {
        return 1;
      }
    }
  };
}

// Polyfill `WebAssembly.instantiateStreaming` for node.
if (!global.WebAssembly.instantiateStreaming) {
  global.WebAssembly.instantiateStreaming =
    (response, imports) => response.then(buf => WebAssembly.instantiate(buf, imports));
}

module.exports = {
  getInvalidTableObject,
  getTableObject,
  getWasmArray,
  getImports,
};
