/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}
