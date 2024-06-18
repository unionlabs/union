#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { sepolia } from "viem/chains"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/union-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const client = createUnionClient({
  evm: {
    chain: sepolia,
    account: evmAccount,
    transport: http("https://rpc2.sepolia.org")
  },
  cosmos: {
    account: cosmosAccount,
    gasPrice: { amount: "0.025", denom: "muno" },
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
  }
})

try {
  const unionToUnionGasCost = await client.simulateTransaction({
    amount: 1n,
    network: "cosmos",
    denomAddress: "muno",
    cosmosSigner: cosmosAccount,
    path: ["union-testnet-8", "union-testnet-8"],
    gasPrice: { amount: "0.025", denom: "muno" },
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  })

  console.info("Union to Union gas cost:", unionToUnionGasCost)

  const unionToSepoliaGasCost = await client.simulateTransaction({
    amount: 1n,
    network: "cosmos",
    denomAddress: "muno",
    sourceChannel: "channel-28",
    path: ["union-testnet-8", "11155111"],
    gasPrice: { amount: "0.025", denom: "muno" },
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    relayContractAddress: "0xD0081080Ae8493cf7340458Eaf4412030df5FEEb"
  })

  console.info("Union to Sepolia gas cost:", unionToSepoliaGasCost)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
