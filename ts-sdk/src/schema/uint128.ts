import * as S from "effect/Schema"

const MAX_UINT128 = 2n ** 128n

export class Uint128FromSelf extends S.BigIntFromSelf.pipe(
  S.betweenBigInt(0n, MAX_UINT128, {
    identifier: "Uint128",
    description: "a 128-bit unsigned integer",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT128)
  })
) {}

export class Uint128 extends S.BigInt.pipe(
  S.betweenBigInt(0n, MAX_UINT128, {
    identifier: "Uint128",
    description: "a 128-bit unsigned integer, in a string",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT128)
  })
) {}
