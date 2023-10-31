const symbol_key = Symbol();

exports.map_with_symbol_key = function() {
  return { [symbol_key]: 42 };
};
exports.symbol_key = function() {
  return symbol_key;
};

exports.Foo = class {};
exports.Bar = class {};
