import { toHex } from "viem"
import { Instruction } from "../src/ucs03.js"

export const exampleBatchInstruction = new Instruction.Batch({
  operand: [
    new Instruction.FungibleAssetOrder({
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
    }),
    new Instruction.FungibleAssetOrder({
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
    })
  ]
})

export const exampleMultiplexInstruction = new Instruction.Multiplex({
  operand: [
    "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
    true,
    "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
    toHex("some smart contract data")
  ]
})

export const exampleForwardInstruction = new Instruction.Forward({
  operand: [
    0n, // example path
    0n, // example timeoutHeight
    1234567890n, // example timeoutTimestamp
    new Instruction.FungibleAssetOrder({
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
    })
  ]
})

export const exampleTransferAndCall = new Instruction.Batch({
  operand: [
    new Instruction.FungibleAssetOrder({
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
    }),
    new Instruction.Multiplex({
      operand: [
        "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
        true,
        "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
        toHex("some smart contract data")
      ]
    }),
    new Instruction.Forward({
      operand: [
        0n,
        10000000n,
        0n,
        new Instruction.Batch({
          operand: [
            new Instruction.FungibleAssetOrder({
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
            })
          ]
        })
      ]
    })
  ]
})

console.log(exampleBatchInstruction, Instruction.encodeAbi(exampleBatchInstruction))
console.log(exampleMultiplexInstruction, Instruction.encodeAbi(exampleMultiplexInstruction))
console.log(exampleForwardInstruction, Instruction.encodeAbi(exampleForwardInstruction))
console.log(exampleTransferAndCall, Instruction.encodeAbi(exampleTransferAndCall))
