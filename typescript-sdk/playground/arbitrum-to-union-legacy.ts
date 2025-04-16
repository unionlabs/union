#!/usr/bin/env bun
import "scripts/patch"
import { parseArgs } from "node:util"
import { fallback, http } from "viem"
import { consola } from "scripts/logger"
import { raise } from "../src/utilities/index.ts"
import { arbitrumSepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, type TransferAssetsParametersLegacy } from "../src/mod.ts"

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

const client = createUnionClient({
  account: evmAccount,
  chainId: `${arbitrumSepolia.id}`,
  transport: fallback([http(arbitrumSepolia?.rpcUrls.default.http.at(0))])
})

const transactionPayload = {
  amount: 1n,
  autoApprove: false,
  denomAddress: LINK_CONTRACT_ADDRESS,
  destinationChainId: "union-testnet-8",
  // or `client.cosmos.account.address` if you want to send to yourself
  receiver: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
} satisfies TransferAssetsParametersLegacy<"421614">

const approval = await client.approveTransactionLegacy(transactionPayload)

if (approval.isErr()) {
  consola.error(approval.error)
  process.exit(1)
}

consola.info(`Approved transaction hash: ${approval.value}`)

const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

if (gasEstimationResponse.isErr()) {
  consola.error(gasEstimationResponse.error)
  process.exit(1)
}

consola.success("Arbitrum to Union gas cost:", gasEstimationResponse.value)

if (ONLY_ESTIMATE_GAS) process.exit(0)

const transfer = await client.transferAssetLegacy(transactionPayload)

if (transfer.isErr()) {
  console.error(transfer.error)
  process.exit(1)
}

consola.info(`Transfer successful: ${transfer.value}`)
