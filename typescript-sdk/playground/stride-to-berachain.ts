#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, offchainQuery, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/stride-to-berachain.ts --private-key "..."` --estimate-gas */

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
  "stride"
)

const [account] = await cosmosAccount.getAccounts()

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [strideTestnetInfo]
  } = await offchainQuery.chain({
    includeContracts: true,
    chainId: "stride-internal-1"
  })

  if (!strideTestnetInfo) raise("Stride testnet info not found")

  const ucsConfiguration = strideTestnetInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const forward = ucsConfiguration.forward.find(item => item.destination_chain.chain_id === "80084")

  if (!forward) raise("Forward configuration not found")

  const client = createUnionClient({
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "ustrd" },
      transport: cosmosHttp("https://stride-testnet-rpc.polkachu.com")
    }
  })

  const pfmMemo = client.createPfmMemo({
    port: forward.port,
    channel: forward.channel_id,
    receiver: berachainAccount.address
  })

  const transactionPayload = {
    amount: 1n,
    memo: pfmMemo,
    approve: true,
    denomAddress: "ustrd",
    network: strideTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    // or `client.evm.account.address` if you want to send to yourself
    recipient: berachainAccount.address,
    relayContractAddress: ucsConfiguration.contract_address,
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  } satisfies TransferAssetsParameters

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  consola.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    consola.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset(transactionPayload)

  consola.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
