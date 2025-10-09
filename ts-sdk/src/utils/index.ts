import crc32 from "crc/crc32"
import { Data, Effect, String as Str } from "effect"
import { fromBytes, fromHex, isHex, toHex } from "viem"

import type { SchemaAST } from "effect"
import { Schema } from "effect"

/**
 * @see https://effect.website/docs/schema/basic-usage/#simplifying-tagged-structs-with-taggedstruct
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

const CHKSUM_LEN = 4

type RpcType = "evm" | "cosmos" | "aptos"

export class CryptoError extends Data.TaggedError("CryptoError")<{
  cause?: unknown
}> {}

export const generateSalt = (rpcType: RpcType) =>
  Effect.gen(function*() {
    const len = (rpcType === "aptos" ? 14 : 32) - CHKSUM_LEN
    const saltBytes = new Uint8Array(len)
    if (globalThis.crypto instanceof Crypto) {
      yield* Effect.try({
        try: () => globalThis.crypto.getRandomValues(saltBytes),
        catch: (cause) => new CryptoError({ cause }),
      })
    } else {
      return yield* new CryptoError({ cause: new Error("Crypto API not supported.") })
    }
    const crc = crc32(saltBytes.buffer).toString(16)
    const paddedCrc = Str.padStart(7, "0")(crc)
    const crcBytes = fromHex(`0x${paddedCrc}`, "bytes")
    const concatenated = new Uint8Array([...saltBytes, ...crcBytes])
    const result = toHex(concatenated)
    return yield* Effect.succeed(result)
  })

export const verifySalt = (hex: `0x${string}`): Effect.Effect<boolean> =>
  Effect.sync(() => {
    const decoded = fromHex(hex, "bytes") as unknown as any
    const delim = decoded.length - CHKSUM_LEN
    const salt = decoded.subarray(0, delim)
    const crc = decoded.subarray(delim, decoded.length)
    const computed = crc32(salt)
    return computed === fromBytes(crc, "number")
  })

export const ensureHex = <T extends string>(s: T) => (isHex(s) ? s : toHex(s))

export { extractErrorDetails } from "./extract-error-details.js"
export { operationNamesFromDocumentNode } from "./gql.js"
