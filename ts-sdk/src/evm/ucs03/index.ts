import { encodeAbiParameters, type Hex } from "viem"
import { batchAbi, fungibleAssetOrderAbi, instructionAbi } from "../abi/index.js"

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
  instructions: Array<{ version: number; opcode: number; operand: Hex }>
): Batch => ({
  opcode: 2,
  version: 0,
  operand: encodeAbiParameters(batchAbi, [instructions])
})

export type Instruction = Hex
export const Instruction = (instruction: {
  opcode: number
  version: number
  operand: Hex
}): Instruction =>
  encodeAbiParameters(instructionAbi, [
    instruction.version,
    instruction.opcode,
    instruction.operand
  ])
