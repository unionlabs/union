#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { fallback, http } from "viem"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { berachainTestnetbArtio } from "viem/chains"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

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
const HONEY_CONTRACT_ADDRESS = "0x0E4aaF1351de4c0264C5c7056Ef3777b41BD8e03"

try {
  const client = createUnionClient({
    account: berachainAccount,
    chainId: `${berachainTestnetbArtio.id}`,
    transport: fallback([
      http(
        "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
      ),
      http(berachainTestnetbArtio?.rpcUrls.default.http.at(0))
    ])
  })

  const transactionPayload = {
    amount: 1n,
    autoApprove: true,
    denomAddress: HONEY_CONTRACT_ADDRESS,
    destinationChainId: "stride-internal-1",
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
  } satisfies TransferAssetsParameters<"80084">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  if (gasEstimationResponse.isErr()) {
    consola.error(gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Union to Berachain gas cost:", gasEstimationResponse.value)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

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
