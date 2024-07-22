#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { fallback, http } from "viem"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { berachainTestnetbArtio } from "viem/chains"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, offchainQuery } from "#mod.ts"

/* `bun playground/union-to-berachain.ts --private-key "..."` --estimate-gas */

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

const berachainAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [unionTestnetInfo]
  } = await offchainQuery.chain({
    chainId: "union-testnet-8",
    includeEndpoints: true,
    includeContracts: true
  })

  if (!unionTestnetInfo) raise("Union testnet info not found")

  const ucsConfiguration = unionTestnetInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "80084")
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const { channel_id, contract_address, source_chain, destination_chain } = ucsConfiguration

  const client = createCosmosSdkClient({
    evm: {
      account: berachainAccount,
      chain: berachainTestnetbArtio,
      transport: fallback([
        http(
          "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
        ),
        http(berachainTestnetbArtio?.rpcUrls.default.http.at(0))
      ])
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
    sourceChannel: channel_id,
    network: unionTestnetInfo.rpc_type,
    // or `client.evm.account.address` if you want to send to yourself
    recipient: berachainAccount.address,
    relayContractAddress: contract_address,
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  consola.box("Union to Berachain gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    amount: 1n,
    denomAddress: "muno",
    sourceChannel: channel_id,
    network: unionTestnetInfo.rpc_type,
    // or `client.evm.account.address` if you want to send to yourself
    recipient: berachainAccount.address,
    relayContractAddress: contract_address,
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
