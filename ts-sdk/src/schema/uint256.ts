import * as S from "effect/Schema"

const MAX_UINT256 = 2n ** 256n

export class Uint256FromSelf extends S.BigIntFromSelf.pipe(
  S.betweenBigInt(0n, MAX_UINT256, {
    identifier: "Uint256",
    description: "a 256-bit unsigned integer",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT256),
  }),
) {}

export class Uint256 extends S.BigInt.pipe(
  S.betweenBigInt(0n, MAX_UINT256, {
    identifier: "Uint256",
    description: "a 256-bit unsigned integer, in a string",
    arbitrary: () => fc => fc.bigInt(0n, MAX_UINT256),
  }),
) {}

export const HexFromUint256 = S.Uint8ArrayFromHex
