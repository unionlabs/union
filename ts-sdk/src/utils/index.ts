import { isHex, toHex, type Hex } from "viem"

export { extractErrorDetails } from "./extract-error-details.js"

type RpcType = "evm" | "cosmos" | "aptos"

export const generateSalt = (rpcType: RpcType): Hex => {
  const saltLength = rpcType === "aptos" ? 14 : 32
  const rawSalt = new Uint8Array(saltLength)

  for (let i = 0; i < rawSalt.length; i++) {
    rawSalt[i] = Math.floor(Math.random() * 256)
  }

  return toHex(rawSalt) as Hex
}

export const ensureHex = <T extends string>(s: T) =>
  isHex(s) ? s : toHex(s)