#!/usr/bin/env bun
import "#patch.ts"
import { http } from "viem"
import { parseArgs } from "node:util"
import { sepolia } from "viem/chains"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { consola, timestamp } from "../scripts/logger.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, offchainQuery } from "#mod.ts"

/* `bun playground/union-to-sepolia.ts --private-key "..."` --estimate-gas */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) raise("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

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

  const client = createCosmosSdkClient({
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

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    denomAddress: "muno",
    network: unionTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  consola.box("Union to Sepolia gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    amount: 1n,
    denomAddress: "muno",
    network: unionTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
