exports.one_two_generator = function() {
  function* generator() {
    yield 1;
    yield 2;
  }
  return generator();
};

exports.dummy_generator = function() {
  function* generator() {
    const reply = yield '2 * 2';
    return reply === 4;
  }
  return generator();
};

exports.broken_generator = function() {
  function* brokenGenerator() {
    throw new Error('Something went wrong');
    yield 1;
  }
  return brokenGenerator();
};
