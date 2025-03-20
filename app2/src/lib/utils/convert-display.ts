import { bech32 } from "@scure/base"
import { Effect, Option, pipe, Schema } from "effect"
import {
  AddressAptosDisplay,
  AddressCosmosDisplay,
  AddressEvmDisplay,
  type AddressCanonicalBytes
} from "$lib/schema/address"

// Helper to convert Uint8Array to AddressCanonicalBytes
const bytesToCanonicalHex = (bytes: Uint8Array): AddressCanonicalBytes =>
  pipe(
    Array.from(bytes),
    arr => arr.reduce((str, byte) => str + byte.toString(16).padStart(2, "0"), "0x"),
    hex => hex as AddressCanonicalBytes
  )

/**
 * Convert a bech32 display address to canonical bytes
 */
export const cosmosDisplayToCanonical = (
  displayAddress: typeof AddressCosmosDisplay.Type
): Effect.Effect<Option.Option<AddressCanonicalBytes>, never, never> =>
  pipe(
    Effect.try({
      try: () => bech32.decode(displayAddress as `${string}1${string}`),
      catch: () => null // Return null on failure
    }),
    Effect.map(result =>
      pipe(
        Option.fromNullable(result), // Convert null to Option.none, success to Option.some
        Option.map(decoded => {
          const canonicalAddress = bech32.fromWords(decoded.words)
          const bytes = new Uint8Array(canonicalAddress)
          return bytesToCanonicalHex(bytes)
        })
      )
    ),
    Effect.catchAll(() => Effect.succeed(Option.none<AddressCanonicalBytes>()))
  )

/**
 * Convert an Evm display address (hex) to canonical bytes
 */
export const evmDisplayToCanonical = (
  displayAddress: typeof AddressEvmDisplay.Type
): Effect.Effect<Option.Option<AddressCanonicalBytes>, never, never> =>
  Effect.sync(() => {
    const hexWithoutPrefix = displayAddress.slice(2)
    const bytes = new Uint8Array(20)
    for (let i = 0; i < 40; i += 2) {
      bytes[i / 2] = Number.parseInt(hexWithoutPrefix.substring(i, i + 2), 16)
    }
    return Option.some(bytesToCanonicalHex(bytes))
  })

/**
 * Convert an Aptos display address (hex) to canonical bytes
 */
export const aptosDisplayToCanonical = (
  displayAddress: typeof AddressAptosDisplay.Type
): Effect.Effect<Option.Option<AddressCanonicalBytes>, never, never> =>
  Effect.sync(() => {
    const hexWithoutPrefix = displayAddress.slice(2)
    const bytes = new Uint8Array(32)
    for (let i = 0; i < 64; i += 2) {
      bytes[i / 2] = Number.parseInt(hexWithoutPrefix.substring(i, i + 2), 16)
    }
    return Option.some(bytesToCanonicalHex(bytes))
  })

/**
 * Validate if a display address converts to a valid canonical form for a given chain
 */
export const isValidCanonicalForChain = (
  displayAddress: string,
  destinationRpcType: string
): Effect.Effect<boolean, never, never> =>
  Effect.sync(() => {
    if (!displayAddress || displayAddress.length === 0) {
      return false
    }

    const validateEvmAddress = (): boolean => {
      return pipe(
        Effect.try({
          try: () => Schema.decodeSync(AddressEvmDisplay)(displayAddress, { errors: "all" }),
          catch: () => null
        }),
        Effect.map(result =>
          pipe(
            Option.fromNullable(result),
            Option.map(decoded => Effect.runSync(evmDisplayToCanonical(decoded))),
            Option.map(canonicalResult =>
              pipe(
                canonicalResult,
                Option.match({
                  onNone: () => false,
                  onSome: value => value.length === 42
                })
              )
            ),
            Option.getOrElse(() => false)
          )
        ),
        Effect.catchAll(() => Effect.succeed(false)),
        Effect.runSync
      )
    }

    const validateCosmosAddress = (): boolean => {
      return pipe(
        Effect.try({
          try: () => Schema.decodeSync(AddressCosmosDisplay)(displayAddress, { errors: "all" }),
          catch: () => null
        }),
        Effect.map(result =>
          pipe(
            Option.fromNullable(result),
            Option.map(decoded => Effect.runSync(cosmosDisplayToCanonical(decoded))),
            Option.map(canonicalResult =>
              pipe(
                canonicalResult,
                Option.match({
                  onNone: () => false,
                  onSome: value => [42, 66].includes(value.length)
                })
              )
            ),
            Option.getOrElse(() => false)
          )
        ),
        Effect.catchAll(() => Effect.succeed(false)),
        Effect.runSync
      )
    }

    const validateAptosAddress = (): boolean => {
      return pipe(
        Effect.try({
          try: () => Schema.decodeSync(AddressAptosDisplay)(displayAddress, { errors: "all" }),
          catch: () => null
        }),
        Effect.map(result =>
          pipe(
            Option.fromNullable(result),
            Option.map(decoded => Effect.runSync(aptosDisplayToCanonical(decoded))),
            Option.map(canonicalResult =>
              pipe(
                canonicalResult,
                Option.match({
                  onNone: () => false,
                  onSome: value => value.length === 66
                })
              )
            ),
            Option.getOrElse(() => false)
          )
        ),
        Effect.catchAll(() => Effect.succeed(false)),
        Effect.runSync
      )
    }

    switch (destinationRpcType) {
      case "evm":
        return validateEvmAddress()
      case "cosmos":
        return validateCosmosAddress()
      case "aptos":
        return validateAptosAddress()
      default:
        return false
    }
  })

// Example
export const runExample = async () => {
  const evmAddr = "0x3C5daAa3c96AB8fe4cFC2fB6d76193fe959A9f82" as typeof AddressEvmDisplay.Type
  const evmResult = await Effect.runPromise(evmDisplayToCanonical(evmAddr))
  console.log(
    "Evm:",
    Option.getOrElse(evmResult, () => "Invalid EVM address")
  )

  const cosmosAddr =
    "union10z7xxj2m8q3f7j58uxmff38ws9u8m0vmne2key" as typeof AddressCosmosDisplay.Type
  const cosmosResult = await Effect.runPromise(cosmosDisplayToCanonical(cosmosAddr))
  console.log(
    "Cosmos:",
    Option.getOrElse(cosmosResult, () => "Invalid Cosmos address")
  )

  const aptosAddr =
    "0x55f8fa1cd0ba02c4ec77b9ee8e6c4c3c040d717af46873be759cd5e2d4205059" as typeof AddressAptosDisplay.Type
  const aptosResult = await Effect.runPromise(aptosDisplayToCanonical(aptosAddr))
  console.log(
    "Aptos:",
    Option.getOrElse(aptosResult, () => "Invalid Aptos address")
  )

  const isValidEvm = await Effect.runPromise(
    isValidCanonicalForChain("0x1234567890123456789012345678901234567890", "evm")
  )
  console.log("Is valid Evm:", isValidEvm)
}
