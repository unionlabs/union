import { toHex } from "viem"
import {
  Batch,
  encodeAbi,
  Forward,
  FungibleAssetOrder,
  Multiplex
} from "../src/ucs03/instruction.js"

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

export const exampleForwardInstruction = Forward(
  0n, // example path
  0n, // example timeoutHeight
  1234567890n, // example timeoutTimestamp
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
)

export const exampleTransferAndCall = Batch([
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
  Multiplex([
    "0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD",
    true,
    "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
    toHex("some smart contract data")
  ]),
  Forward(
    0n,
    10000000n,
    0n,
    Batch([
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
  )
])

console.log(exampleBatchInstruction, encodeAbi(exampleBatchInstruction))
console.log(exampleMultiplexInstruction, encodeAbi(exampleMultiplexInstruction))
console.log(exampleForwardInstruction, encodeAbi(exampleForwardInstruction))
console.log(exampleTransferAndCall, encodeAbi(exampleTransferAndCall))
