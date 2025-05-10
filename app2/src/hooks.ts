if (typeof BigInt.prototype.toJSON !== "function") {
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}

export const init: () => Promise<void> = async () => {
  await import("$lib/runtime")
  await import("$lib/logging/datadog").then(x => x.init())
}
