import type { encodeAbiParameters } from "viem"
import type { forwardAbi, fungibleAssetOrderAbi, multiplexAbi } from "../evm/abi/index.js"

type Instruction = Forward | Multiplex | Batch | FungibleAssetOrder

type Forward = {
  opcode: 0
  version: 0
  operand: [bigint, bigint, bigint, Instruction]
}
export const Forward = (
  path: bigint,
  timeoutHeight: bigint,
  timeoutTimestamp: bigint,
  instruction: Instruction
): Forward => ({
  opcode: 0,
  version: 0,
  operand: [path, timeoutHeight, timeoutTimestamp, instruction]
})

type Multiplex = {
  opcode: 1
  version: 0
  operand: Parameters<typeof encodeAbiParameters<typeof multiplexAbi>>[1]
}
export const Multiplex = (
  operand: Parameters<typeof encodeAbiParameters<typeof multiplexAbi>>[1]
): Multiplex => ({
  opcode: 1,
  version: 0,
  operand
})

type Batch = {
  opcode: 2
  version: 0
  operand: Array<Instruction>
}
export const Batch = (instructions: Array<Instruction>): Batch => ({
  opcode: 2,
  version: 0,
  operand: instructions
})

type FungibleAssetOrder = {
  opcode: 3
  version: 1
  operand: Parameters<typeof encodeAbiParameters<typeof fungibleAssetOrderAbi>>[1]
}
export const FungibleAssetOrder = (
  operand: Parameters<typeof encodeAbiParameters<typeof fungibleAssetOrderAbi>>[1]
): FungibleAssetOrder => ({
  opcode: 3,
  version: 1,
  operand
})
