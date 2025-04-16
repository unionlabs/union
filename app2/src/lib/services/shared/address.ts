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

    if (isHexMovement(trimmed, { strict: true })) {
      // TODO: Do we need more verification like getAddress and so on?
      return yield* Effect.try({
        try: () => trimmed,
        catch: err =>
          new AddressValidationError({
            input: trimmed,
            cause: err
          })
      })
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

export function isHexMovement(
  value: unknown,
  { strict = true }: { strict?: boolean } = {}
): boolean {
  if (!value) return false
  if (typeof value !== "string") return false
  // In strict mode, require a 0x prefix and exactly 64 hex characters after it.
  return strict ? /^0x[0-9a-fA-F]{64}$/.test(value) : value.startsWith("0x") && value.length === 66
}
