class JsCast1 {
  constructor() {
    this.val = 1;
  }
  myval() { return this.val; }
}

class JsCast2 {
}

class JsCast3 extends JsCast1 {
  constructor() {
    super();
    this.val = 3;
  }
}

class JsCast4 extends JsCast3 {
  constructor() {
    super();
    this.val = 4;
  }
}

exports.JsCast1 = JsCast1;
exports.JsCast2 = JsCast2;
exports.JsCast3 = JsCast3;
exports.JsCast4 = JsCast4;
