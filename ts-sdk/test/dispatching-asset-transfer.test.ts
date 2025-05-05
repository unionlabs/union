import { describe, it } from "@effect/vitest"

import { createWalletClient, http } from "viem"
import { mnemonicToAccount } from "viem/accounts"
import { sepolia, holesky } from "viem/chains"
import { english, generateMnemonic } from "viem/accounts"

import { toHex, type Hex } from "viem"
import { ucs03abi } from "@unionlabs/sdk/evm/abi"
import * as Instruction from "@unionlabs/sdk/ucs03/instruction"

function generateSalt() {
  const rawSalt = new Uint8Array(32)
  crypto.getRandomValues(rawSalt)
  return toHex(rawSalt) as Hex
}

describe("Dispatching an Asset Transfer", () => {
  describe("Managing wallets", () => {
    it("example 1", () => {
      const mnemonic = generateMnemonic(english)
      const account = mnemonicToAccount(mnemonic)

      const sepoliaWallet = createWalletClient({
        account,
        chain: sepolia,
        transport: http()
      })

      const holeskyWallet = createWalletClient({
        account,
        chain: holesky,
        transport: http()
      })

      console.log(`Sepolia address: ${sepoliaWallet.account.address}`)
      console.log(`Holesky address: ${holeskyWallet.account.address}`)
    })
  })
  describe("Bridging", () => {
    // TODO: add mocking
    it.fails("create FAO", async () => {
      const MOCK_SOURCE_CHANNEL_ID = 0
      const WETH_ADDRESS = "0x123"

      const mnemonic = generateMnemonic(english)
      const account = mnemonicToAccount(mnemonic)

      const sepoliaWallet = createWalletClient({
        account,
        chain: sepolia,
        transport: http()
      })

      const holeskyWallet = createWalletClient({
        account,
        chain: holesky,
        transport: http()
      })

      // We're actually enqueuing two transfers, the main transfer, and fee.
      const instruction = new Instruction.Batch({
        operand: [
          // Our main transfer.
          new Instruction.FungibleAssetOrder({
            operand: [
              sepoliaWallet.account.address,
              holeskyWallet.account.address,
              WETH_ADDRESS,
              4n,
              // symbol
              "WETH",
              // name
              "Wrapped Ether",
              // decimals
              18,
              // path
              0n,
              // quote token
              "0x685a6d912eced4bdd441e58f7c84732ceccbd1e4",
              // quote amount
              4n
            ]
          }),
          // Our fee transfer.
          new Instruction.FungibleAssetOrder({
            operand: [
              sepoliaWallet.account.address,
              holeskyWallet.account.address,
              WETH_ADDRESS,
              1n,
              // symbol
              "WETH",
              // name
              "Wrapped Ether",
              // decimals
              18,
              // path
              0n,
              // quote token
              "0x685a6d912eced4bdd441e58f7c84732ceccbd1e4",
              // quote amount
              0n
            ]
          })
        ]
      })

      const transferHash = await sepoliaWallet.writeContract({
        account: sepoliaWallet.account.address,
        abi: ucs03abi,
        chain: sepolia,
        functionName: "send",
        address: holeskyWallet.account.address,
        args: [
          // obtained from the graphql Channels query
          MOCK_SOURCE_CHANNEL_ID,
          // this transfer is timeout out by timestamp, so we set height to 0.
          0n,
          // The actual timeout. It is current time + 2 hours.
          BigInt(Math.floor(Date.now() / 1000) + 7200),
          generateSalt(),
          {
            opcode: instruction.opcode,
            version: instruction.version,
            operand: Instruction.encodeAbi(instruction)
          }
        ]
      })

      console.log("transferHash", transferHash)
    })
  })
})
