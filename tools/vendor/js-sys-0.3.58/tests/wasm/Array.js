// Used for `Array.rs` tests
exports.populate_array =  function(arr, start, len) {
  for (i = 0; i < len; i++) {
    arr[i] = start + i;
  }
};
