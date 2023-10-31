exports.import_me = function() {};

global.webkitMySpecialApi = class {
  foo() { return 123; }
};
global.MySpecialApi2 = class {
  foo() { return 124; }
};
global.bMySpecialApi3 = class {
  foo() { return 125; }
};
