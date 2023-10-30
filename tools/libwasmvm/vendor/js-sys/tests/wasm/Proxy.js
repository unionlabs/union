exports.proxy_target = function() {
  return { a: 100 };
};

exports.proxy_handler = function() {
  return {
    get: function(obj, prop) {
      return prop in obj ? obj[prop] : 37;
    }
  };
};
