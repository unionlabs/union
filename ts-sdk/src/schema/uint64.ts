import * as S from "effect/Schema"

const MAX_UINT64 = 18_446_744_073_709_551_615n

export class Uint64 extends S.BigIntFromSelf.pipe(
  S.betweenBigInt(0n, MAX_UINT64, {
    identifier: "Uint64",
    description: "a 64-bit unsigned integer",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT64)
  })
) {}

export class Uint64FromString extends S.BigInt.pipe(
  S.betweenBigInt(0n, MAX_UINT64, {
    identifier: "Uint64FromString",
    description: "a 64-bit unsigned integer, in a string",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT64)
  })
) {}
