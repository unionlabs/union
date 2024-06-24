#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { raise } from "#utilities/index.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, cosmosHttp, offchainQuery } from "#mod.ts"

/* `bun playground/osmosis-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "osmo"
)

try {
  const {
    data: [osmosisTestnetInfo]
  } = await offchainQuery.chain({
    includeContracts: true,
    chainId: "osmo-test-5"
  })

  if (!osmosisTestnetInfo) raise("Osmosis testnet info not found")

  const ucsConfiguration = osmosisTestnetInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const client = createCosmosSdkClient({
    // @ts-expect-error
    evm: {},
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "uosmo" },
      transport: cosmosHttp(
        osmosisTestnetInfo.rpcs?.at(0)?.url ?? "https://rpc.osmo.test.yieldpay.finance"
      )
    }
  })

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    denomAddress: "uosmo",
    network: osmosisTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  console.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    amount: 1n,
    denomAddress: "uosmo",
    network: osmosisTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
