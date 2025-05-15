// Polyfill for serializing BigInt as string in JSON
if (typeof BigInt.prototype.toJSON !== "function") {
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
