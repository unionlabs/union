import type { encodeAbiParameters } from "viem"
import type { fungibleAssetOrderAbi, multiplexAbi } from "../evm/abi/index.js"
import { toHex } from "viem"

type Instruction = Multiplex | Batch | FungibleAssetOrder

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

const instr: Instruction = {
  opcode: 2,
  version: 0,
  operand: [
    {
      opcode: 3,
      version: 1,
      operand: [
        toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
        "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
        toHex("muno"),
        4n,
        "muno",
        "muno",
        18,
        0n,
        "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
        4n
      ]
    }
  ]
}

const instr2 = Batch([
  FungibleAssetOrder([
    toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
    "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
    toHex("muno"),
    4n,
    "muno",
    "muno",
    18,
    0n,
    "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
    4n
  ]),
  FungibleAssetOrder([
    toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"),
    "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
    toHex("muno"),
    4n,
    "muno",
    "muno",
    18,
    0n,
    "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
    4n
  ])
])
