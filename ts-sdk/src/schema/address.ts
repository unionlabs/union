import { bech32, bytes } from "@scure/base"
import { Effect, ParseResult, pipe, Schema as S } from "effect"
import { Address, checksumAddress, isAddress } from "viem"
import * as Ucs05 from "../Ucs05.js"
import { Hex, HexChecksum, HexFromString } from "./hex.js"

/**
 * @category models
 * @since 2.0.0
 */
export const Bech32 = S.NonEmptyString.pipe(
  S.brand("Bech32"),
  S.filter((a, _, ast) => {
    try {
      bech32.decode(a as any)
      return true
    } catch (e) {
      return new ParseResult.Type(ast, a, (e as Error).message)
    }
  }),
) as unknown as S.filter<S.brand<S.TemplateLiteral<`${string}1${string}`>, "Bech32">>
/**
 * @category models
 * @since 2.0.0
 */
export type Bech32 = typeof Bech32.Type

/**
 * @remarks
 * For Reference, see: https://docs.union.build/ucs/05.
 * We always store bytes arrays as hex-encoded strings.
 *
 * @category models
 * @since 2.0.0
 */
export const AddressCanonicalBytes = Hex.pipe(S.brand("CanonicalBytes"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressCanonicalBytes = typeof AddressCanonicalBytes.Type

// Cosmos Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const AddressCosmosCanonical = AddressCanonicalBytes.pipe(S.brand("AddressCosmosCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressCosmosCanonical = typeof AddressCosmosCanonical.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AddressCosmosDisplay = Bech32.pipe(S.brand("AddressCosmosDisplay"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressCosmosDisplay = typeof AddressCosmosDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AddressCosmosZkgm = Hex.pipe(S.brand("AddressCosmosZkgm")) // TODO: Hex<Bech32<Hrp, Cosmos.Canonical>>
/**
 * @category models
 * @since 2.0.0
 */
export type AddressCosmosZkgm = typeof AddressCosmosZkgm.Type

// export const AddressCosmosDisplayFromCanonical = flow(
//   Bech32FromAddressCanonicalBytesWithPrefix,
//   S.compose(AddressCosmosDisplay)
// )

/**
 * @category utils
 * @since 2.0.0
 */
export const AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix = (prefix: string) =>
  pipe(
    Bech32FromAddressCanonicalBytesWithPrefix(prefix),
    S.compose(HexFromString),
    S.compose(AddressCosmosZkgm),
  )

// Evm Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const AddressEvmCanonical = AddressCanonicalBytes.pipe(S.brand("AddressEvmCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressEvmCanonical = typeof AddressEvmCanonical.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AddressEvmDisplay = HexChecksum.pipe(S.brand("AddressEvmDisplay"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressEvmDisplay = typeof AddressEvmDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AddressEvmZkgm = AddressEvmCanonical.pipe(S.brand("AddressEvmZkgm"))
/**
 * @category models
 * @since 2.0.0
 */
export type AddressEvmZkgm = typeof AddressEvmZkgm.Type

// Aptos Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const AddressAptosCanonical = AddressCanonicalBytes.pipe(S.brand("AddressAptosCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export const AddressAptosDisplay = AddressAptosCanonical
/**
 * @category models
 * @since 2.0.0
 */
export const AddressAptosZkgm = AddressAptosCanonical

/**
 * @category models
 * @since 2.0.0
 */
export const ERC55 = S.NonEmptyString.pipe(
  S.filter(a => isAddress(a, { strict: true }) && checksumAddress(a as Address) === a, {
    description: "a string matching ERC-55 in checksum format",
  }),
)
/**
 * @category models
 * @since 2.0.0
 */
export type ERC55 = typeof ERC55.Type

// TODO: rename me
export const ValidAddress = S.Union(ERC55, Bech32)
export type ValidAddress = typeof ValidAddress.Type

/**
 * @category errors
 * @since 2.0.0
 */
export class Bech32EncodeError extends S.TaggedClass<Bech32EncodeError>()("Bech32EncodeError", {
  message: S.String,
}) {}

/**
 * @category errors
 * @since 2.0.0
 */
export class Bech32DecodeError extends S.TaggedClass<Bech32DecodeError>()("Bech32DecodeError", {
  message: S.String,
}) {}

/**
 * @category models
 * @since 2.0.0
 */
export const Bech32FromAddressCanonicalBytesWithPrefix = (
  prefix: Ucs05.HRP,
  options: {
    validateHrp: boolean
  } = {
    validateHrp: true,
  },
) =>
  S.transformOrFail(AddressCanonicalBytes, Bech32, {
    strict: true,
    decode: (fromA, _options, ast, _fromI) =>
      Effect.try({
        try: () => {
          const words = bech32.toWords(bytes("hex", fromA.slice(2)))
          return bech32.encode(prefix, words) as `${string}1${string}`
        },
        catch: cause => new Bech32EncodeError({ message: (cause as Error).message }),
      }).pipe(
        Effect.mapBoth({
          onSuccess: a => Bech32.make(a as any),
          onFailure: e => new ParseResult.Type(ast, fromA, e.message),
        }),
      ),
    encode: (toI, _options, ast, _toA) => {
      const a = Effect.try({
        try: () => bech32.decode(toI),
        catch: cause => new Bech32DecodeError({ message: (cause as Error).message }),
      }).pipe(
        // TODO: convert to try; pull out fn
        Effect.flatMap(decoded => {
          if (options?.validateHrp) {
            if (decoded.prefix !== prefix) {
              return Effect.fail(
                new Bech32DecodeError({
                  message:
                    `Given prefix "${decoded.prefix}" does not match requirement "${prefix}"`,
                }),
              )
            }
          }
          const canonicalAddress = bech32.fromWords(decoded.words)
          const bytes = new Uint8Array(canonicalAddress)
          const bytesToCanonicalHex = (bytes: Uint8Array): Hex =>
            pipe(
              Array.from(bytes),
              arr => arr.reduce((str, byte) => str + byte.toString(16).padStart(2, "0"), "0x"),
              hex => hex as Hex,
            )
          return Effect.succeed(AddressCanonicalBytes.make(bytesToCanonicalHex(bytes)))
        }),
        Effect.mapError(e => new ParseResult.Type(ast, toI, e.message)),
      )
      return a
      // return void 0 as unknown as Effect.Effect<Hex>
    },
  })
