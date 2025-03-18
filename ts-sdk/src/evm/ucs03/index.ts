import { encodeAbiParameters, type Hex } from "viem"
import { batchAbi, fungibleAssetOrderAbi } from "../abi/index.js"

export type FungibleAssetOrder = { opcode: 3; version: 1; operand: Hex }
export const FungibleAssetOrder = (
  operand: Parameters<typeof encodeAbiParameters<typeof fungibleAssetOrderAbi>>[1]
): FungibleAssetOrder => ({
  opcode: 3,
  version: 1,
  operand: encodeAbiParameters(fungibleAssetOrderAbi, operand)
})

export type Batch = { opcode: 2; version: 0; operand: Hex }
export const Batch = (
  instructions: Array<{ version: number; opcode: number; operand: `0x${string}` }>
): Batch => ({
  opcode: 2,
  version: 0,
  operand: encodeAbiParameters(batchAbi, [instructions])
})
