import { Effect, ParseResult } from "effect"
import { pipe } from "effect/Function"
import * as S from "effect/Schema"
import { fromHex, toHex } from "viem"

/**
 * Describes `0x${string}` from non-empty string.
 *
 * Note: To be used ineroperably with `viem`.
 */
export const Hex = S.NonEmptyString.pipe( // TODO: add `Bytes` brand separately
  S.pattern(/^0x[0-9a-fA-F]+$/), // TODO: remove uppercase
  S.minLength(3),
  S.annotations({
    title: "hex",
    description: "hex string",
  }),
) as unknown as S.TemplateLiteral<`0x${string}`>
export type Hex = typeof Hex.Type

// TODO: validate ERC55 checksum
// TODO: see `Hex` for type hacking to avoid `TemplateLiteral` incongruency
export const HexChecksum = S.NonEmptyString.pipe(
  S.pattern(/^0x[0-9a-fA-F]+$/),
  S.minLength(3),
) as unknown as S.TemplateLiteral<`0x${string}`>
export type HexChecksum = typeof HexChecksum.Type

export const HexFromString = S.transform(S.String, Hex, {
  strict: true,
  decode: s => toHex(s),
  encode: hex => fromHex(hex, "string"),
})
export type HexFromString = typeof HexFromString.Type

export const StringFromHex = S.transform(Hex, S.String, {
  strict: true,
  decode: hex => fromHex(hex, "string"),
  encode: s => toHex(s),
})
export type StringFromHex = typeof HexFromString.Type

/**
 * TODO: handle signed hex?
 */
export const NumberFromHexString = S.transformOrFail(
  Hex,
  S.Positive,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Effect.try({
          try: () => Number.parseInt(fromA, 16),
          catch: (e) => new Error(String(e)),
        }),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
      ),
    encode: (toI) => Effect.succeed(`0x${toI.toString(16)}` as const),
    strict: true,
  },
)
export type NumberFromHexString = typeof NumberFromHexString.Type

export const HexFromJson = S.transformOrFail(
  S.Unknown,
  Hex,
  {
    encode: (toI, options, ast, toA) =>
      pipe(
        toI,
        S.decode(StringFromHex),
        Effect.flatMap(S.decode(S.parseJson())),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    decode: (fromA, options, ast, fromI) =>
      pipe(
        fromA,
        S.encode(S.parseJson()),
        Effect.flatMap(S.encode(StringFromHex)),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
  },
)

// Sui type tag: 0x<hex>::<Module>::<Name>
export const SuiTypeTag = S.String.pipe(
  S.pattern(/^0x[0-9a-fA-F]+::[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*$/),
  S.annotations({
    title: "sui-type-tag",
    description: "Sui coin type tag like 0x2::sui::SUI",
    examples: ["0x2::sui::SUI"],
  }),
)
export type SuiTypeTag = typeof SuiTypeTag.Type

/**
 * Hex <-> Sui type tag ("0x2::sui::SUI") bridge.
 * Decodes a Hex that encodes a Sui type tag (UTF-8) into the type tag string, and vice versa.
 */
export const StringFromSuiHex = S.transformOrFail(
  Hex,                     // encoded side (0x...)
  SuiTypeTag,              // decoded side ("0x...::mod::Name")
  {
    // Hex -> "0x...::module::Name"
    decode: (hex, _opts, ast) => {
      const s = fromHex(hex, "string")
      return /^0x[0-9a-fA-F]+::[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*$/.test(s)
        ? Effect.succeed(s as typeof SuiTypeTag.Type)
        : Effect.fail(new ParseResult.Type(ast, hex, `Expected hex-encoded Sui type tag, got "${s}"`))
    },
    // "0x...::module::Name" -> Hex
    encode: (tag) => Effect.succeed(toHex(tag)),
    strict: true,
  },
)
export type StringFromSuiHex = typeof StringFromSuiHex.Type

/**
 * If you ever need the opposite direction explicitly typed:
 * Sui type tag string -> Hex, and back.
 */
export const SuiHexFromString = S.transformOrFail(
  SuiTypeTag,
  Hex,
  {
    decode: (tag) => Effect.succeed(toHex(tag)),
    encode: (hex, _opts, ast) => {
      const s = fromHex(hex, "string")
      return /^0x[0-9a-fA-F]+::[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*$/.test(s)
        ? Effect.succeed(hex)
        : Effect.fail(new ParseResult.Type(ast, hex, `Expected hex of a Sui type tag, got "${s}"`))
    },
    strict: true,
  },
)
export type SuiHexFromString = typeof SuiHexFromString.Type
