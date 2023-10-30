// Used for `Array.rs` tests
exports.populate_array = function(arr, start, len) {
  var isBigInt = typeof(arr[0]) === "bigint";
  for (i = 0; i < len; i++) {
    arr[i] = isBigInt ? BigInt(start + i) : start + i;
  }
};
