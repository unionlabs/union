const assert = require('assert');

exports.test_has_instance = function(sym) {
  class Array1 {
    static [sym](instance) {
      return Array.isArray(instance);
    }
  }

  assert.equal(typeof sym, "symbol");
  assert.ok([] instanceof Array1);
};

exports.test_is_concat_spreadable = function(sym) {
  const alpha = ['a', 'b', 'c'];
  const numeric = [1, 2, 3];
  let alphaNumeric = alpha.concat(numeric);

  assert.deepEqual(alphaNumeric, ["a", "b", "c", 1, 2, 3]);

  numeric[sym] = false;
  alphaNumeric = alpha.concat(numeric);

  assert.deepEqual(alphaNumeric, ["a", "b", "c", numeric]);
};

exports.test_iterator = function(sym) {
  const iterable1 = new Object();

  iterable1[sym] = function* () {
    yield 1;
    yield 2;
    yield 3;
  };

  assert.deepEqual([...iterable1], [1, 2, 3]);
};

exports.test_async_iterator = async function(sym) {
  const iterable1 = new Object();

  iterable1[sym] = function () {
    let done = false;

    return {
      next() {
        if (done) {
          return Promise.resolve({
            done: true,
            value: 1
          });

        } else {
          done = true;

          return Promise.resolve({
            done: false,
            value: 0
          });
        }
      }
    };
  };

  const values = [];

  for await (let value of iterable1) {
    values.push(value);
  }

  assert.deepEqual(values, [0]);
};

exports.test_match = function(sym) {
  const regexp1 = /foo/;
  assert.throws(() => '/foo/'.startsWith(regexp1));

  regexp1[sym] = false;

  assert.ok('/foo/'.startsWith(regexp1));

  assert.equal('/baz/'.endsWith(regexp1), false);
};

exports.test_replace = function(sym) {
  class Replace1 {
    constructor(value) {
      this.value = value;
    }
    [sym](string) {
      return `s/${string}/${this.value}/g`;
    }
  }

  assert.equal('foo'.replace(new Replace1('bar')), 's/foo/bar/g');
};

exports.test_search = function(sym) {
  class Search1 {
    constructor(value) {
      this.value = value;
    }

    [sym](string) {
      return string.indexOf(this.value);
    }
  }

  assert.equal('foobar'.search(new Search1('bar')), 3);
};

exports.test_species = function(sym) {
  class Array1 extends Array {
    static get [sym]() { return Array; }
  }

  const a = new Array1(1, 2, 3);
  const mapped = a.map(x => x * x);

  assert.equal(mapped instanceof Array1, false);

  assert.ok(mapped instanceof Array);
};

exports.test_split = function(sym) {
  class Split1 {
    constructor(value) {
      this.value = value;
    }

    [sym](string) {
      var index = string.indexOf(this.value);
      return this.value + string.substr(0, index) + "/"
        + string.substr(index + this.value.length);
    }
  }

  assert.equal('foobar'.split(new Split1('foo')), 'foo/bar');
};

exports.test_to_primitive = function(sym) {
  const object1 = {
    [sym](hint) {
      if (hint == 'number') {
        return 42;
      }
      return null;
    }
  };

  assert.equal(+object1, 42);
};

exports.test_to_string_tag = function(sym) {
  class ValidatorClass {
    get [sym]() {
      return 'Validator';
    }
  }

  assert.equal(Object.prototype.toString.call(new ValidatorClass()), '[object Validator]');
};
