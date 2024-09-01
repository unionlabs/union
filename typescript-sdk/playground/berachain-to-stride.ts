#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { fallback, getAddress, http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { berachainTestnetbArtio } from "viem/chains"
import {
  bech32AddressToHex,
  createPfmMemo,
  createUnionClient,
  offchainQuery,
  type TransferAssetsParameters
} from "#mod.ts"

/* `bun playground/berachain-to-stride.ts --private-key "..."` --estimate-gas */

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

const WBTC_CONTRACT_ADDRESS = "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae"

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [beraInfo]
  } = await offchainQuery.chain({
    chainId: "80084",
    includeContracts: true,
    includeEndpoints: true
  })

  if (!beraInfo) raise("Stride testnet info not found")

  const ucsConfiguration = beraInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const forward = ucsConfiguration.forward.find(
    item => item.destination_chain.chain_id === "stride-internal-1"
  )

  if (!forward) raise("Forward configuration not found")

  const client = createUnionClient({
    chainId: "80084",
    account: berachainAccount,
    chain: berachainTestnetbArtio,
    transport: fallback([
      http(
        "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
      ),
      http(berachainTestnetbArtio?.rpcUrls.default.http.at(0))
    ])
  })

  const pfmMemo = createPfmMemo({
    port: forward.port,
    channel: forward.channel_id,
    receiver: bech32AddressToHex({
      address: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
    })
  })

  if (pfmMemo.isErr()) {
    consola.error(pfmMemo.error)
    process.exit(1)
  }

  const transactionPayload = {
    amount: 1n,
    approve: false,
    memo: pfmMemo.value,
    denomAddress: WBTC_CONTRACT_ADDRESS,
    sourceChannel: ucsConfiguration.channel_id,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3",
    destinationChainId: ucsConfiguration.destination_chain.chain_id,
    relayContractAddress: getAddress(ucsConfiguration.contract_address)
  } satisfies TransferAssetsParameters<"80084">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  if (gasEstimationResponse.isErr()) {
    consola.error(gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Union to Berachain gas cost:", gasEstimationResponse.value)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    console.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  const approval = await client.approveTransaction({
    account: berachainAccount,
    amount: transactionPayload.amount,
    denomAddress: getAddress(transactionPayload.denomAddress),
    relayContractAddress: getAddress(transactionPayload.relayContractAddress)
  })

  if (approval.isErr()) {
    consola.error("Approval failed", approval.error)
    process.exit(1)
  }

  consola.info("Approval successful", approval.value)

  const transfer = await client.transferAsset(transactionPayload)

  if (transfer.isErr()) {
    consola.error("Transfer failed", transfer.error)
    process.exit(1)
  }

  consola.info("Transfer successful", transfer.value)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
