#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { arbitrumSepolia } from "viem/chains"
import { fallback, getAddress, http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, offchainQuery, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/arbitrum-to-union.ts --private-key "..."` */

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

const LINK_CONTRACT_ADDRESS = "0xb1d4538b4571d411f07960ef2838ce337fe1e80e"

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [arbitrumInfo]
  } = await offchainQuery.chain({
    chainId: `${arbitrumSepolia.id}`,
    includeEndpoints: true,
    includeContracts: true
  })
  if (!arbitrumInfo) raise("Berachain info not found")

  const ucsConfiguration = arbitrumInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const client = createUnionClient({
    account: evmAccount,
    chain: arbitrumSepolia,
    chainId: `${arbitrumSepolia.id}`,
    transport: fallback([http(arbitrumSepolia?.rpcUrls.default.http.at(0))])
  })

  const transactionPayload = {
    amount: 1n,
    approve: true,
    sourceChannel: ucsConfiguration.channel_id,
    denomAddress: LINK_CONTRACT_ADDRESS,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    destinationChainId: ucsConfiguration.destination_chain.chain_id,
    relayContractAddress: getAddress(ucsConfiguration.contract_address)
  } satisfies TransferAssetsParameters<"421614">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  if (gasEstimationResponse.isErr()) {
    consola.error(gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Union to Berachain gas cost:", gasEstimationResponse.value)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  const transfer = await client.transferAsset(transactionPayload)

  if (transfer.isErr()) {
    console.error(transfer.error)
    process.exit(1)
  }

  consola.info(transfer.value)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
