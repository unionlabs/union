#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { arbitrumSepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, offchainQuery, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/union-to-arbitrum.ts --private-key "..."` --estimate-gas */

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

const [account] = await cosmosAccount.getAccounts()

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
    ?.filter(config => config.destination_chain.chain_id === `${arbitrumSepolia.id}`)
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const { channel_id, contract_address, source_chain, destination_chain } = ucsConfiguration

  const client = createCosmosSdkClient({
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "muno" },
      transport: cosmosHttp("https://rpc.testnet-8.union.build")
    }
  })

  const transactionPayload = {
    amount: 1n,
    denomAddress: "muno",
    sourceChannel: channel_id,
    // or `client.evm.account.address` if you want to send to yourself
    recipient: evmAccount.address,
    network: unionTestnetInfo.rpc_type,
    relayContractAddress: contract_address,
    path: [source_chain.chain_id, destination_chain.chain_id]
  } satisfies TransferAssetsParameters

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  consola.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset(transactionPayload)

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
