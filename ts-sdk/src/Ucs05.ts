/**
 * This module handles interaction with the [UCS05](https://docs.union.build/ucs/05/) standard.
 *
 * @since 2.0.0
 */
import { isValidSuiAddress, normalizeSuiAddress } from "@mysten/sui/utils"
import { bech32, bytes } from "@scure/base"
import {
  absurd,
  Effect,
  flow,
  identity,
  Match,
  ParseResult,
  pipe,
  Schema as S,
  Struct,
} from "effect"
import { isAddress, toHex } from "viem"
import { AddressCanonicalBytes } from "./schema/address.js"
import { Hex, HexFromString } from "./schema/hex.js"

// const AddressFromChain = (chain: Chain) =>

/**
 * @category models
 * @since 2.0.0
 */
export const SuiAddress = S.NonEmptyString.pipe(
  S.filter((a) => isValidSuiAddress(a), {
    description: "Sui address (32-byte hex). Accepts with/without 0x; even length; hex only.",
  }),
)
/**
 * @category models
 * @since 2.0.0
 */
export type SuiAddress = typeof SuiAddress.Type

/**
 * @category models
 * @since 2.0.0
 */
export const SuiDisplay = S.Struct({
  _tag: S.tag("SuiDisplay"),
  address: SuiAddress,
})
/**
 * @category models
 * @since 2.0.0
 */
export type SuiDisplay = typeof SuiDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const HRP = S.String.pipe(
  S.length(
    {
      min: 1,
      max: 83,
    },
    {
      description: "HRP must be between 1 to 83 US-ASCII characters, inclusive",
    },
  ),
  S.pattern(/^[\x21-\x7E]+$/, {
    description: "HRP characters must be within the range [33-126], inclusive",
  }),
)
/**
 * @category models
 * @since 2.0.0
 */
export type HRP = typeof HRP.Type

/**
 * @category models
 * @since 2.0.0
 */
export const Bech32 = S.NonEmptyString.pipe(
  S.filter((a, _, ast) => {
    try {
      bech32.decode(a as any)
      return true
    } catch (e) {
      return new ParseResult.Type(ast, a, (e as Error).message)
    }
  }),
) as unknown as S.filter<S.TemplateLiteral<`${string}1${string}`>>
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
export const CanonicalBytes = Hex.pipe(S.brand("CanonicalBytes"))
/**
 * @category models
 * @since 2.0.0
 */
export type CanonicalBytes = typeof CanonicalBytes.Type

// Cosmos Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const CosmosCanonical = CanonicalBytes.pipe(S.brand("CosmosCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosCanonical = typeof CosmosCanonical.Type

/**
 * @category models
 * @since 2.0.0
 */
export const CosmosZkgm = Hex.pipe(S.brand("CosmosZkgm")) // TODO: Hex<Bech32<Hrp, Cosmos.Canonical>>
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosZkgm = typeof CosmosZkgm.Type

// export const AddressCosmosDisplayFromCanonical = flow(
//   Bech32FromAddressCanonicalBytesWithPrefix,
//   S.compose(AddressCosmosDisplay)
// )

/**
 * @category utils
 * @since 2.0.0
 */
export const CosmosZkgmFromCanonicalBytesWithPrefix = (prefix: string) =>
  pipe(
    Bech32FromCanonicalBytesWithPrefix(prefix),
    S.compose(HexFromString),
    S.compose(CosmosZkgm),
  )

// Evm Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const EvmCanonical = CanonicalBytes.pipe(S.brand("EvmCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export type EvmCanonical = typeof EvmCanonical.Type

/**
 * @category models
 * @since 2.0.0
 */
export const EvmZkgm = EvmCanonical.pipe(S.brand("EvmZkgm"))
/**
 * @category models
 * @since 2.0.0
 */
export type EvmZkgm = typeof EvmZkgm.Type

// Aptos Address Types
/**
 * @category models
 * @since 2.0.0
 */
export const AptosCanonical = CanonicalBytes.pipe(S.brand("AptosCanonical"))
/**
 * @category models
 * @since 2.0.0
 */
export const AptosDisplay = AptosCanonical
/**
 * @category models
 * @since 2.0.0
 */
export const AptosZkgm = AptosCanonical

/**
 * @category models
 * @since 2.0.0
 */
export const ERC55 = S.NonEmptyString.pipe(
  S.filter(a => isAddress(a, { strict: true }), {
    description: "a string matching ERC-55 in checksum format",
  }),
)
/**
 * @category models
 * @since 2.0.0
 */
export type ERC55 = typeof ERC55.Type

/**
 * @category models
 * @since 2.0.0
 */
export const EvmDisplay = S.Struct({
  _tag: S.tag("EvmDisplay"),
  address: ERC55,
})
/**
 * @category models
 * @since 2.0.0
 */
export type EvmDisplay = typeof EvmDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const CosmosDisplay = S.Struct({
  _tag: S.tag("CosmosDisplay"),
  address: Bech32,
})
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosDisplay = typeof CosmosDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AnyDisplay = S.Union(
  CosmosDisplay,
  EvmDisplay,
  SuiDisplay,
)
/**
 * @category models
 * @since 2.0.0
 */
export type AnyDisplay = typeof AnyDisplay.Type

/**
 * @category models
 * @since 2.0.0
 */
export const AnyDisplayFromString = S.transformOrFail(
  S.String,
  AnyDisplay,
  {
    decode: (address) =>
      pipe(
        Effect.raceAll([
          S.decodeUnknownEither(EvmDisplay)({ _tag: "EvmDisplay", address }),
          S.decodeUnknownEither(CosmosDisplay)({ _tag: "CosmosDisplay", address }),
          S.decodeUnknownEither(SuiDisplay)({ _tag: "SuiDisplay", address }),
        ]),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: flow(Struct.get("address"), Effect.succeed),
  },
)

/**
 * @category models
 * @since 2.0.0
 */
export type AnyDisplayFromString = typeof AnyDisplayFromString.Type

/**
 * @category models
 * @since 2.0.0
 */
export const Zkgm = Hex.pipe(S.brand("Zkgm"))
/**
 * @category models
 * @since 2.0.0
 */
export type Zkgm = typeof Zkgm.Type

/**
 * @category transformations
 * @since 2.0.0
 */
export const ZkgmFromAnyDisplay = S.transform(
  AnyDisplay,
  Zkgm,
  {
    decode: (fromA) =>
      Match.value(fromA).pipe(
        Match.tagsExhaustive({
          CosmosDisplay: ({ address }) => toHex(address),
          EvmDisplay: ({ address }) => identity<Hex>(address),
          SuiDisplay: ({ address }) => identity<Hex>(normalizeSuiAddress(address) as Hex),
        }),
      ),
    encode: (_) => absurd<AnyDisplay>(void 0 as never),
  },
)

/**
 * @category utils
 * @since 2.0.0
 */
export const anyDisplayToZkgm = Match.type<AnyDisplay>().pipe(
  Match.tagsExhaustive({
    CosmosDisplay: ({ address }) => S.decode(HexFromString)(address),
    EvmDisplay: ({ address }) => Effect.succeed<Hex>(address),
    SuiDisplay: ({ address }) => S.decode(HexFromString)(normalizeSuiAddress(address)),
  }),
)

/**
 * @category transformations
 * @since 2.0.0
 */
export const anyDisplayToCanonical = Match.type<AnyDisplay>().pipe(
  Match.tagsExhaustive({
    // XXX: THIS IS WRONG
    CosmosDisplay: ({ address }) => {
      // AddressCanonicalBytes.make(toHex(address)),
      const { bytes } = bech32.decodeToBytes(address)
      const result = AddressCanonicalBytes.make(toHex(bytes))
      return result
      console.log("bytes", { result })
    },
    EvmDisplay: ({ address }) => AddressCanonicalBytes.make(address),
    SuiDisplay: ({ address }) => AddressCanonicalBytes.make(normalizeSuiAddress(address) as Hex),
  }),
)
/**
 * Union of possible valid address schemas.
 *
 * @category models
 * @since 2.0.0
 */
export const ValidAddress = S.Union(ERC55, Bech32, SuiAddress)
/**
 * @category models
 * @since 2.0.0
 */
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
export const Bech32FromCanonicalBytesWithPrefix = (
  prefix: HRP,
  options: {
    validateHrp: boolean
  } = {
    validateHrp: true,
  },
) =>
  S.transformOrFail(CanonicalBytes, Bech32, {
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
          return Effect.succeed(CanonicalBytes.make(bytesToCanonicalHex(bytes)))
        }),
        Effect.mapError(e => new ParseResult.Type(ast, toI, e.message)),
      )
      return a
      // return void 0 as unknown as Effect.Effect<Hex>
    },
  })
