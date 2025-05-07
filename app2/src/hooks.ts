if (typeof BigInt.prototype.toJSON !== "function") {
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
