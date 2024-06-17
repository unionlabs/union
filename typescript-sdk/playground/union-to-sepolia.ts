#!/usr/bin/env bun
import "#patch.ts"
import { http } from "viem"
import { parseArgs } from "node:util"
import { sepolia } from "viem/chains"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { consola, timestamp } from "../scripts/logger.ts"
import { createUnionClient, offchainQuery } from "#mod.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/union-to-sepolia.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "tx-count": { type: "string", default: "1" }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const TX_COUNT = Number(values["tx-count"])

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

consola.box(`Sending ${TX_COUNT} transactions from Union to Sepolia`)

const stamp = timestamp()

try {
  const {
    data: [unionTestnetInfo]
  } = await offchainQuery.chain({
    includeContracts: true,
    chainId: "union-testnet-8"
  })

  if (!unionTestnetInfo) raise("Union testnet info not found")

  const ucsConfiguration = unionTestnetInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "11155111")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

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

  const hash = await client.transferAsset({
    amount: 1n,
    denomAddress: "muno",
    network: unionTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  console.info(hash)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
