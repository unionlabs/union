const assert = require('assert');

exports.MyType = class {
  static foo(y) {
    assert.equal(y, 'x');
    return y + 'y';
  }

  constructor(x) {
    assert.equal(x, 2);
    this._a = 1;
  }

  bar(x) {
    assert.equal(x, true);
    return 3.2;
  }

  get a() {
    return this._a;
  }
  set a(v) {
    this._a = v;
  }
};
