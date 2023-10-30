exports.get_char_at = function() {
  return "foo".charAt;
};

exports.Rectangle = class {
  constructor(x, y){
    this.x = x,
    this.y = y
  }

  static eq(x, y) {
    return x === y;
  }
};

exports.Rectangle2 = class {
  constructor(x, y){
    this.x = x,
    this.y = y
  }

  static eq(x, y) {
    return x === y;
  }
};

exports.throw_all_the_time = () => new Proxy({}, {
  getPrototypeOf() { throw new Error("nope"); },
  setPrototypeOf() { throw new Error("nope"); },
  isExtensible() { throw new Error("nope"); },
  preventExtensions() { throw new Error("nope"); },
  getOwnPropertyDescriptor() { throw new Error("nope"); },
  defineProperty() { throw new Error("nope"); },
  has() { throw new Error("nope"); },
  get() { throw new Error("nope"); },
  set() { throw new Error("nope"); },
  deleteProperty() { throw new Error("nope"); },
  ownKeys() { throw new Error("nope"); },
  apply() { throw new Error("nope"); },
  construct() { throw new Error("nope"); },
});
