#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "../src/utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexToBytes } from "../src/convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, type TransferAssetsParametersLegacy } from "../src/mod.ts"
import { berachainTestnetbArtio } from "viem/chains"

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
  Uint8Array.from(hexToBytes(PRIVATE_KEY)),
  "union"
)

const [account] = await cosmosAccount.getAccounts()
console.info(account?.address)

try {
  const client = createUnionClient({
    account: cosmosAccount,
    chainId: "union-testnet-8",
    gasPrice: { amount: "0.0025", denom: "muno" },
    transport: http("https://rpc.testnet-8.union.build")
  })

  const transactionPayload = {
    amount: 1n,
    denomAddress: "muno",
    // or `client.evm.account.address` if you want to send to yourself
    receiver: berachainAccount.address,
    destinationChainId: `${berachainTestnetbArtio.id}`
  } satisfies TransferAssetsParametersLegacy<"union-testnet-8">

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

  const transfer = await client.transferAssetLegacy(transactionPayload)

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
