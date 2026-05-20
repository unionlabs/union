import * as S from "effect/Schema"

const MAX_UINT8 = (2n ** 8n) - 1n

export class Uint8FromSelf extends S.BigIntFromSelf.pipe(
  S.betweenBigInt(0n, MAX_UINT8, {
    identifier: "Uint8",
    description: "a 8-bit unsigned integer",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT8),
  }),
) {}

export class Uint8 extends S.BigInt.pipe(
  S.betweenBigInt(0n, MAX_UINT8, {
    identifier: "Uint8",
    description: "a 8-bit unsigned integer, represented as a decimal string",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT8),
  }),
) {}
