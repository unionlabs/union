#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { http, getAddress } from "viem"
import { cosmosHttp } from "#transport.ts"
import { createUnionClient } from "#mod.ts"
import { GasPrice } from "@cosmjs/stargate"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import contracts from "~root/versions/contracts.json" with { type: "json" }

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

const CHANNEL = "channel-22"

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

const currentContracts = contracts.find(c => c.latest === true) as (typeof contracts)[0]
const relayContractAddress = getAddress(currentContracts?.sepolia.UCS01)

const client = createUnionClient({
  evm: {
    chain: sepolia,
    account: evmAccount,
    transport: http("https://rpc2.sepolia.org")
  },
  cosmos: {
    account: cosmosAccount,
    transport: cosmosHttp("https://rpc.testnet.bonlulu.uno"),
    gasPrice: GasPrice.fromString("0.0025muno")
  }
})

const transfer = await client.transferAsset({
  network: "evm",
  path: ["11155111", "union-testnet-8"],
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
  sourceChannel: CHANNEL,
  amount: 1n,
  denomAddress: USDC_CONTRACT_ADDRESS,
  relayContractAddress
})

console.info(transfer)
