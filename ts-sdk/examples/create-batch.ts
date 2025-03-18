import { toHex } from "viem"
import { Batch, FungibleAssetOrder, Instruction } from "../src/evm/ucs03/index.js"

const fungibleAssetOrder = FungibleAssetOrder([
  "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
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

const batch = Batch([fungibleAssetOrder])

const instruction = Instruction(batch)

console.log(instruction)
