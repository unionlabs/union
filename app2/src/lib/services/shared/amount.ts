import { Effect, Option } from "effect"
import { parseUnits } from "viem"
import { AmountParsingError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import type { Token } from "$lib/schema/token.ts"

export const parseAmountEffect = (amount: string, token: Token) =>
  Effect.gen(function* () {
    if (!token) {
      return "0"
    }

    const decimals = token.representations[0]?.decimals ?? 0

    if (!amount || amount.trim() === "") {
      return "0"
    }

    // Convert parseUnits(...) to string
    return yield* Effect.try({
      try: () => parseUnits(amount, decimals).toString(),
      catch: err =>
        new AmountParsingError({
          input: amount,
          decimals,
          cause: err
        })
    })
  })

// Updated to return Option.Option<string> (the string representation)
export const getParsedAmountSafe = (amount: string, token: Token): Option.Option<string> => {
  const result = Effect.runSync(Effect.either(parseAmountEffect(amount, token)))

  return result._tag === "Right"
    ? Option.some(result.right) // Already a string
    : Option.none()
}
