import { Effect } from 'effect'
import { AddressValidationError } from './errors'
import {getAddress} from "viem";
import {bech32AddressToHex} from "@unionlabs/client";

export const deriveReceiverEffect = (input: string) =>
  Effect.gen(function* () {
    const trimmed = input.trim()

    // Handle empty input
    if (!trimmed) {
      return yield* Effect.fail(new AddressValidationError({
        input,
        cause: undefined
      }))
    }

    if (trimmed.toLowerCase().startsWith("0x")) {
      // For hex addresses
      return yield* Effect.try({
        try: () => getAddress(trimmed),
        catch: err => new AddressValidationError({
          input: trimmed,
          cause: err
        })
      })
    } else {
      // For bech32 addresses
      return yield* Effect.try({
        try: () => {
          const hexAddress = bech32AddressToHex({ address: trimmed })
          return getAddress(hexAddress)
        },
        catch: err => new AddressValidationError({
          input: trimmed,
          cause: err
        })
      })
    }
  })

/**
 * Helper function that returns null for invalid inputs (for use during typing)
 */
export const getDerivedReceiverSafe = (input: string): string | null => {
  const result = Effect.runSync(
    Effect.either(deriveReceiverEffect(input))
  )

  return result._tag === 'Right' ? result.right : null
}