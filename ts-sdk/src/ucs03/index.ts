import { encodeAbiParameters, type Hex } from "viem"
import { batchAbi, forwardAbi, fungibleAssetOrderAbi, multiplexAbi } from "../evm/abi/index.js"

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

export const encodeAbi = (instruction: Instruction): Hex => {
  switch (instruction.opcode) {
    case 0: {
      // Forward
      return encodeAbiParameters(forwardAbi, [
        instruction.operand[0],
        instruction.operand[1],
        instruction.operand[2],
        {
          opcode: instruction.operand[3].opcode,
          version: instruction.operand[3].version,
          operand: encodeAbi(instruction.operand[3])
        }
      ])
    }
    case 1: {
      // Multiplex
      return encodeAbiParameters(multiplexAbi, instruction.operand)
    }
    case 2: {
      // Batch - recursively encode each instruction
      return encodeAbiParameters(batchAbi, [
        instruction.operand.map(instr => ({
          version: instr.version,
          opcode: instr.opcode,
          operand: encodeAbi(instr)
        }))
      ])
    }
    case 3: {
      // FungibleAssetOrder
      return encodeAbiParameters(fungibleAssetOrderAbi, instruction.operand)
    }
    default: {
      throw new Error(`impossible`)
    }
  }
}
