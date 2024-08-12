#!/usr/bin/env bun
import { http } from "viem"
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { offchainQuery, createCosmosSdkClient } from "#mod.ts"

/**
 *
 *
 * W  I  P
 *
 *
 */

/* `bun playground/sepolia-to-osmosis.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

try {
  const {
    data: [sepoliaInfo]
  } = await offchainQuery.chain({
    chainId: "11155111",
    includeContracts: true
  })

  if (!sepoliaInfo) raise("Sepolia testnet info not found")

  const ucsConfiguration = sepoliaInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "osmo-test-5")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const client = createCosmosSdkClient({
    evm: {
      chain: sepolia,
      account: evmAccount,
      transport: http("https://rpc2.sepolia.org")
    }
  })

  const transfer = await client.transferAsset({
    amount: 1n,
    network: sepoliaInfo.rpc_type,
    denomAddress: USDC_CONTRACT_ADDRESS,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "osmo14qemq0vw6y3gc3u3e0aty2e764u4gs5l32ydm0",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
