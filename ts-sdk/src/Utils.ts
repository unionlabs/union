/**
 * This module contains generic utilities.
 *
 * @since 2.0.0
 */
import crc32 from "crc/crc32"
import { Data, Effect, String as Str } from "effect"
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
