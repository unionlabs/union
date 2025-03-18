import { encodeAbiParameters, type Hex } from "viem"
import { batchAbi, fungibleAssetOrderAbi, instructionAbi } from "../abi/index.js"

export type FungibleAssetOrder = { opcode: 3; version: 1; operand: Hex }
/**
 * Creates a FungibleAssetOrder instruction
 * @param operand - The parameters for the fungible asset order as an array:
 * - [0] sender - The sender address as bytes
 * - [1] receiver - The receiver address as bytes
 * - [2] baseToken - The base token address as bytes
 * - [3] baseAmount - The amount of base token as uint256
 * - [4] baseTokenSymbol - The symbol of the base token
 * - [5] baseTokenName - The name of the base token
 * - [6] baseTokenDecimals - The number of decimals for the base token
 * - [7] baseTokenPath - The path identifier for the base token as uint256
 * - [8] quoteToken - The quote token address as bytes
 * - [9] quoteAmount - The amount of quote token as uint256
 * @returns A FungibleAssetOrder instruction object
 */
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
