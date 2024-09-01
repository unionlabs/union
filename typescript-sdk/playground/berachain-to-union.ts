#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { fallback, getAddress, http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { berachainTestnetbArtio } from "viem/chains"
import { offchainQuery, createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/berachain-to-union.ts --private-key "..."` */

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

console.info(berachainAccount.address)

const WBTC_CONTRACT_ADDRESS = "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae"
const DAI_CONTRACT_ADDRESS = "0x806Ef538b228844c73E8E692ADCFa8Eb2fCF729c"

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [beraInfo]
  } = await offchainQuery.chain({
    chainId: "80084",
    includeEndpoints: true,
    includeContracts: true
  })
  if (!beraInfo) raise("Berachain info not found")

  const ucsConfiguration = beraInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const client = createUnionClient({
    chainId: "80084",
    account: berachainAccount,
    transport: fallback([
      http(
        "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
      ),
      http(berachainTestnetbArtio?.rpcUrls.default.http.at(0))
    ])
  })

  const transactionPayload = {
    amount: 1n,
    approve: true,
    sourceChannel: ucsConfiguration.channel_id,
    denomAddress: DAI_CONTRACT_ADDRESS,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
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
