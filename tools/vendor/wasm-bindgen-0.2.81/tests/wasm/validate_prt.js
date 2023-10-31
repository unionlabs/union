const wasm = require('wasm-bindgen-test.js');
const assert = require('assert');

// NB: `wasm-pack` uses the presence of checks for moved values as a way to test
// whether it is correctly enabling `--debug` when configured to do so, so don't
// change this expected debug output without also updating `wasm-pack`'s tests.
const assertMovedPtrThrows = process.env.WASM_BINDGEN_NO_DEBUG == null
    ? f => assert.throws(f, /Attempt to use a moved value/)
    : f => assert.throws(f, /null pointer passed to rust/);

const useMoved = () => {
    const apple = new wasm.Fruit('apple');
    apple.name();
    wasm.eat(apple);
    assertMovedPtrThrows(() => apple.name());
};

const moveMoved = () => {
    const pear = new wasm.Fruit('pear');
    pear.name();
    wasm.eat(pear);
    assertMovedPtrThrows(() => wasm.eat(pear));
};

const methodMoved = () => {
    const quince = new wasm.Fruit('quince');
    quince.name();
    quince.rot();
    assertMovedPtrThrows(() => quince.rot());
};

exports.js_works = () => {
    useMoved();
    moveMoved();
    methodMoved();

    const a = new wasm.Fruit('a');
    a.prop;
    assertMovedPtrThrows(() => a.prop);
    const b = new wasm.Fruit('a');
    b.prop = 3;
    assertMovedPtrThrows(() => { b.prop = 4; });
};
