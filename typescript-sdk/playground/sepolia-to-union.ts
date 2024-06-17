#!/usr/bin/env bun
import { http } from "viem"
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/sepolia-to-union.ts --private-key "..."` */

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

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const relayContractAddress = "0xD0081080Ae8493cf7340458Eaf4412030df5FEEb"

const client = createUnionClient({
  evm: {
    chain: sepolia,
    account: evmAccount,
    transport: http("https://rpc2.sepolia.org")
  },
  cosmos: {
    account: cosmosAccount,
    gasPrice: { amount: "0.0025", denom: "muno" },
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
  }
})

try {
  const transfer = await client.transferAsset({
    amount: 1n,
    network: "evm",
    relayContractAddress,
    sourceChannel: "channel-22",
    denomAddress: USDC_CONTRACT_ADDRESS,
    path: ["11155111", "union-testnet-8"],
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
