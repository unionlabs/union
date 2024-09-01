#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import type { TransferAssetsParameters } from "#client/types.ts"
import { createUnionClient, createPfmMemo, offchainQuery } from "#mod.ts"

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
    account: cosmosAccount,
    chainId: "stride-internal-1",
    gasPrice: { amount: "0.0025", denom: "ustrd" },
    transport: http("https://stride-testnet-rpc.polkachu.com")
  })

  const pfmMemo = createPfmMemo({
    port: forward.port,
    channel: forward.channel_id,
    receiver: berachainAccount.address
  })

  if (pfmMemo.isErr()) {
    consola.error(pfmMemo.error)
    process.exit(1)
  }

  const transactionPayload = {
    amount: 1n,
    approve: true,
    memo: pfmMemo.value,
    denomAddress: "ustrd",
    // or `client.evm.account.address` if you want to send to yourself
    recipient: berachainAccount.address,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    destinationChainId: ucsConfiguration.destination_chain.chain_id
  } satisfies TransferAssetsParameters<"stride-internal-1">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  if (gasEstimationResponse.isErr()) {
    consola.error(gasEstimationResponse.error)
    process.exit(1)
  }

  consola.info(`Gas cost: ${gasEstimationResponse.value}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  const transfer = await client.transferAsset(transactionPayload)

  if (transfer.isErr()) {
    consola.error(transfer.error)
    process.exit(1)
  }

  consola.info(transfer.value)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
