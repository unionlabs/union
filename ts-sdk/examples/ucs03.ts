import { toHex } from "viem"
import { Batch, FungibleAssetOrder, Multiplex } from "../src/ucs03/index.js"

export const exampleInstruction = {
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

export const exampleBatchInstruction = Batch([
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

export const exampleMultiplexInstruction = Multiplex([
  "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
  true,
  "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
  toHex("some smart contract data")
])

console.log(exampleInstruction)
console.log(exampleBatchInstruction)
console.log(exampleMultiplexInstruction)
