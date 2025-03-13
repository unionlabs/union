import { Effect, Option } from "effect"
import { getAddress, isHex } from "viem"
import { bech32AddressToHex } from "@unionlabs/client"
import { AddressValidationError } from "$lib/services/shared"

export const deriveReceiverEffect = (input: string) =>
  Effect.gen(function* () {
    const trimmed = input.trim()

    if (!trimmed) {
      return yield* Effect.fail(
        new AddressValidationError({
          input,
          cause: undefined
        })
      )
    }

    if (isHex(trimmed, { strict: true })) {
      return yield* Effect.try({
        try: () => getAddress(trimmed),
        catch: err =>
          new AddressValidationError({
            input: trimmed,
            cause: err
          })
      })
    }

    return yield* Effect.try({
      try: () => {
        const hexAddress = bech32AddressToHex({ address: trimmed })
        return getAddress(hexAddress)
      },
      catch: err =>
        new AddressValidationError({
          input: trimmed,
          cause: err
        })
    })
  })

export const getDerivedReceiverSafe = (input: string): Option.Option<string> => {
  const result = Effect.runSync(Effect.either(deriveReceiverEffect(input)))
  return result._tag === "Right" ? Option.some(result.right) : Option.none()
}
