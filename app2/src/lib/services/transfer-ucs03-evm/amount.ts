import {Effect, Option} from "effect";
import {parseUnits} from "viem";
import {AmountParsingError} from "$lib/services/transfer-ucs03-evm/errors.ts";
import type {Token} from "$lib/schema/token.ts";

export const parseAmountEffect = (amount: string, token: Token) =>
  Effect.gen(function* () {
    if (!token) {
      return BigInt(0).toString()
    }

    const decimals = token.representations[0]?.decimals ?? 0

    if (!amount || amount.trim() === '') {
      return BigInt(0).toString()
    }

    return yield* Effect.try({
      try: () => parseUnits(amount.toString(), decimals),
      catch: err => new AmountParsingError({
        input: amount,
        decimals,
        cause: err
      })
    })
  })

// Updated to return Option<bigint>
export const getParsedAmountSafe = (amount: string, token: Token): Option.Option<bigint> => {
  const result = Effect.runSync(
    Effect.either(parseAmountEffect(amount, token))
  )

  return result._tag === 'Right'
    ? Option.some(BigInt(result.right))
    : Option.none();
}