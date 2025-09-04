/**
 * This module contains generic utilities.
 *
 * @since 2.0.0
 */
import crc32 from "crc/crc32"
import { BigDecimal, Data, Effect, Schema, SchemaAST, String as Str } from "effect"
import { dual, LazyArg, pipe } from "effect/Function"
import * as M from "effect/Match"
import * as O from "effect/Option"
import { Unify } from "effect/Unify"
import { fromBytes, fromHex, isHex, toHex } from "viem"

const CHKSUM_LEN = 4

type RpcType = "evm" | "cosmos" | "aptos"

/**
 * @category errors
 * @since 2.0.0
 */
export class CryptoError extends Data.TaggedError("CryptoError")<{
  cause?: unknown
}> {}

/**
 * @category utils
 * @since 2.0.0
 */
export const generateSalt = (rpcType: RpcType) =>
  Effect.gen(function*() {
    const len = (rpcType === "aptos" ? 14 : 32) - CHKSUM_LEN
    const saltBytes = new Uint8Array(len)
    if (globalThis.crypto instanceof Crypto) {
      try {
        globalThis.crypto.getRandomValues(saltBytes)
      } catch (cause) {
        return yield* new CryptoError({ cause })
      }
    } else {
      return yield* new CryptoError({ cause: new Error("Crypto API not supported.") })
    }
    const crc = crc32(saltBytes).toString(16)
    const paddedCrc = Str.padStart(7, "0")(crc)
    const crcBytes = fromHex(`0x${paddedCrc}`, "bytes")
    const concatenated = new Uint8Array([...saltBytes, ...crcBytes])
    const result = toHex(concatenated)
    return yield* Effect.succeed(result)
  })

/**
 * @category utils
 * @since 2.0.0
 */
export const verifySalt = (hex: `0x${string}`): Effect.Effect<boolean> =>
  Effect.sync(() => {
    const decoded: Uint8Array<ArrayBuffer> = fromHex(hex, "bytes")
    const delim = decoded.length - CHKSUM_LEN
    const salt = decoded.subarray(0, delim)
    const crc = decoded.subarray(delim, decoded.length)
    const computed = crc32(salt)
    return computed === fromBytes(crc, "number")
  })

/**
 * Ensure input is hex encoded (e.g. `0x${string}`), otherwise encode as hex.
 *
 * @category utils
 * @since 2.0.0
 */
export const ensureHex = <T extends string>(s: T) => (isHex(s) ? s : toHex(s))

/**
 * Determine timeout timestamp for a fungible asset order.
 *
 * @remarks Actually three days, not 1 day
 *
,* @category utils
 * @since 2.0.0
 */
export function getTimeoutInNanoseconds24HoursFromNow(): bigint {
  const millisecondsNow = Date.now() // current time in ms
  const millisecondsIn24Hours = 24 * 60 * 60 * 1000 * 3 // 24 hours in ms * 3
  const totalMilliseconds = millisecondsNow + millisecondsIn24Hours
  return BigInt(totalMilliseconds) * BigInt(1_000_000) // convert ms to ns
}

/**
 * @category utils
 * @since 2.0.0
 */
export function extractErrorDetails<T extends Error>(
  error: T,
  withOwnProperties: boolean | undefined = true,
):
  & {
    [K in keyof T]: T[K]
  }
  & {
    message: string
    name: string
    stack?: string
    cause?: unknown
  }
{
  const extractedError = {} as
    & {
      [K in keyof T]: T[K]
    }
    & {
      message: string
      name: string
      stack?: string
      cause?: unknown
    }

  // Extract all own properties, including non-enumerable ones
  if (withOwnProperties) {
    Object.getOwnPropertyNames(error).forEach(key => {
      extractedError[key as keyof T] = error[key as keyof T]
    })
  }

  // Explicitly copy inherited properties
  extractedError.message = error.message
  extractedError.name = error.name
  if (error.stack) {
    extractedError.stack = error.stack
  }
  if ("cause" in error) {
    extractedError.cause = error.cause
  }

  return extractedError
}

/**
 * @category utils
 * @since 2.0.0
 */
export const matchOptionBool: {
  <A, B, C>(options: {
    readonly onNone: LazyArg<A>
    readonly onSomeTrue: (a: true) => B
    readonly onSomeFalse: (a: false) => C
  }): (self: O.Option<boolean>) => Unify<A | Unify<B | C>>
  <A, B, C>(self: O.Option<boolean>, options: {
    readonly onNone: LazyArg<A>
    readonly onSomeTrue: (a: true) => B
    readonly onSomeFalse: (a: false) => C
  }): Unify<A | Unify<B | C>>
} = dual(
  2,
  <A, B, C>(self: O.Option<boolean>, { onNone, onSomeTrue, onSomeFalse }: {
    readonly onNone: LazyArg<A>
    readonly onSomeTrue: (a: true) => B
    readonly onSomeFalse: (a: false) => C
  }): Unify<A | Unify<B | C>> =>
    pipe(
      M.value(self),
      M.tagsExhaustive({
        Some: ({ value }) =>
          M.value(value).pipe(
            M.when(true, onSomeTrue),
            M.when(false, onSomeFalse),
            M.exhaustive,
          ),
        None: onNone,
      }),
    ),
)

/**
 * @see https://effect.website/docs/schema/basic-usage/#simplifying-tagged-structs-with-taggedstruct
 * @category utils
 * @since 2.0.0
 */
export const TaggedStruct = <
  Tag extends SchemaAST.LiteralValue,
  Fields extends Schema.Struct.Fields,
>(
  tag: Tag,
  fields: Fields,
) =>
  Schema.Struct({
    _tag: Schema.Literal(tag).pipe(
      Schema.optional,
      Schema.withDefaults({
        constructor: () => tag, // Apply _tag during instance construction
        decoding: () => tag, // Apply _tag during decoding
      }),
    ),
    ...fields,
  })

/**
 * @category utils
 * @since 2.0.0
 */
export const formatBigDecimal = (n: BigDecimal.BigDecimal): string => {
  const normalized = BigDecimal.normalize(n)

  const negative = normalized.value < BigInt(0)
  const absolute = negative ? `${normalized.value}`.substring(1) : `${normalized.value}`

  let before: string
  let after: string

  if (normalized.scale >= absolute.length) {
    before = "0"
    after = "0".repeat(normalized.scale - absolute.length) + absolute
  } else {
    const location = absolute.length - normalized.scale
    if (location > absolute.length) {
      const zeros = location - absolute.length
      before = `${absolute}${"0".repeat(zeros)}`
      after = ""
    } else {
      after = absolute.slice(location)
      before = absolute.slice(0, location)
    }
  }

  const complete = after === "" ? before : `${before}.${after}`
  return negative ? `-${complete}` : complete
}
