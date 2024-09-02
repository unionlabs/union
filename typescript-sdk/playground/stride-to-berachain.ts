#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { createUnionClient } from "#mod.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { berachainTestnetbArtio } from "viem/chains"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import type { TransferAssetsParameters } from "#client/types.ts"

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
  const client = createUnionClient({
    account: cosmosAccount,
    chainId: "stride-internal-1",
    gasPrice: { amount: "0.0025", denom: "ustrd" },
    transport: http("https://stride-testnet-rpc.polkachu.com")
  })

  const transactionPayload = {
    amount: 1n,
    autoApprove: true,
    denomAddress: "ustrd",
    // or `client.evm.account.address` if you want to send to yourself
    recipient: berachainAccount.address,
    destinationChainId: `${berachainTestnetbArtio.id}`
  } satisfies TransferAssetsParameters<"stride-internal-1">

  consola.info(transactionPayload)

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
