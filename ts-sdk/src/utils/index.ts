import { fromBytes, fromHex, isHex, toHex, type Hex } from "viem"
import crc32 from "crc/crc32"
import { Effect } from "effect"

export { extractErrorDetails } from "./extract-error-details.js"

type RpcType = "evm" | "cosmos" | "aptos"

export const generateSalt = (rpcType: RpcType): Effect.Effect<Hex> =>
  Effect.sync(() => {
    const len = (rpcType === "aptos" ? 14 : 32) - 4
    const saltBytes = new Uint8Array(len)
    if (globalThis.crypto instanceof Crypto) {
      globalThis.crypto.getRandomValues(saltBytes)
    } else {
      throw new Error("Crypto API not supported.")
    }
    const crc = crc32(saltBytes).toString(16)
    const crcBytes = fromHex(`0x${crc}`, "bytes")
    const concatenated = new Uint8Array([...saltBytes, ...crcBytes])
    const result = toHex(concatenated)
    return result
  })

export const verifySalt = (hex: `0x${string}`): Effect.Effect<boolean> =>
  Effect.sync(() => {
    const decoded: Uint8Array<ArrayBuffer> = fromHex(hex, "bytes")
    const delim = decoded.length - 4
    const salt = decoded.subarray(0, delim)
    const crc = decoded.subarray(delim, decoded.length)
    const computed = crc32(salt)
    return computed === fromBytes(crc, "number")
  })

export const ensureHex = <T extends string>(s: T) => (isHex(s) ? s : toHex(s))
