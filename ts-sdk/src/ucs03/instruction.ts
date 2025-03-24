import { encodeAbiParameters, type Hex } from "viem"
import { batchAbi, forwardAbi, fungibleAssetOrderAbi, multiplexAbi } from "../evm/abi/index.js"
import * as M from "effect/Match"

export type Instruction = Forward | Multiplex | Batch | FungibleAssetOrder

export type Forward = {
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

export type Multiplex = {
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

export type Batch = {
  opcode: 2
  version: 0
  operand: Array<Instruction>
}
export const Batch = (instructions: Array<Instruction>): Batch => ({
  opcode: 2,
  version: 0,
  operand: instructions
})

export type FungibleAssetOrder = {
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

// type InstructionToHex = (_: Instruction) => Hex
export const encodeAbi: (_: Instruction) => Hex = M.type<Instruction>().pipe(
  M.when(
    i => i.opcode === 0,
    i =>
      encodeAbiParameters(forwardAbi, [
        i.operand[0],
        i.operand[1],
        i.operand[2],
        {
          opcode: i.operand[3].opcode,
          version: i.operand[3].version,
          operand: encodeAbi(i.operand[3])
        }
      ])
  ),
  M.when(
    i => i.opcode === 1,
    i => encodeAbiParameters(multiplexAbi, i.operand)
  ),
  M.when(
    i => i.opcode === 2,
    i =>
      encodeAbiParameters(batchAbi, [
        i.operand.map(instr => ({
          version: instr.version,
          opcode: instr.opcode,
          operand: encodeAbi(instr)
        }))
      ])
  ),
  M.when(
    i => i.opcode === 3, // FungibleAssetOrder
    i => encodeAbiParameters(fungibleAssetOrderAbi, i.operand)
  ),
  // M.exhautive
  M.orElseAbsurd // check why not exhaustive
)
