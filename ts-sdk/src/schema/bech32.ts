import { bech32, bytes } from "@scure/base"
import { Effect, ParseResult, pipe, Schema as S } from "effect"
import { Hex } from "./hex.js"
import { AddressCanonicalBytes } from "./address.js"
import { HRP } from "./chain.js"

export const Bech32 = S.NonEmptyString.pipe(
  S.brand("Bech32"),
  S.filter(
    (a, _, ast) => {
      try {
        bech32.decode(a as any)
        return true
      } catch (e) {
        return new ParseResult.Type(ast, a, (e as Error).message)
      }
    }
  )
) as unknown as S.filter<S.brand<S.TemplateLiteral<`${string}1${string}`>, "Bech32">>
export type Bech32 = typeof Bech32.Type

export class Bech32EncodeError extends S.TaggedClass<Bech32EncodeError>()(
  "Bech32EncodeError",
  {
    message: S.String,
  }
) {}

export class Bech32DecodeError extends S.TaggedClass<Bech32DecodeError>()(
  "Bech32DecodeError",
  {
    message: S.String,
  }
) {}

export const Bech32FromAddressCanonicalBytesWithPrefix = (
  // TODO(ehegnes): also validate HRP
  prefix: HRP
) => S.transformOrFail(
  AddressCanonicalBytes,
  Bech32,
  {
    strict: true,
    decode: (fromA, options, ast, fromI) =>
      Effect.try({
        try: () => {
          const words = bech32.toWords(bytes("hex", fromA.slice(2)))
          return bech32.encode(prefix, words) as `${string}1${string}`
        },
        catch: (cause) => new Bech32EncodeError({ message: (cause as Error).message })
      }).pipe(
        Effect.mapBoth({
          onSuccess: (a) => Bech32.make(a as any),
          onFailure: (e) => new ParseResult.Type(ast, fromA, e.message)
        })
      ),
    encode: (toI, options, ast, toA) => {
      const a = Effect.try({
        try: () => bech32.decode(toI),
        catch: (cause) => new Bech32DecodeError({ message: (cause as Error).message })
      }).pipe(
        // TODO: convert to try; pull out fn
        Effect.map((decoded) => {
          const canonicalAddress = bech32.fromWords(decoded.words)
          const bytes = new Uint8Array(canonicalAddress)
          const bytesToCanonicalHex = (bytes: Uint8Array): Hex =>
            pipe(
              Array.from(bytes),
              arr => arr.reduce((str, byte) => str + byte.toString(16).padStart(2, "0"), "0x"),
              hex => hex as Hex
            )
          return AddressCanonicalBytes.make(bytesToCanonicalHex(bytes))
        }),
        Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message))
      )
      return a
      // return void 0 as unknown as Effect.Effect<Hex>
    },
  }
)