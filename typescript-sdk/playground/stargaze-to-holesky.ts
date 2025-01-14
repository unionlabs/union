#!/usr/bin/env bun
import "scripts/patch.ts"
import { http } from "viem"
import { holesky } from "viem/chains"
import { parseArgs } from "node:util"
import { raise } from "#utilities/index.ts"
import { consola } from "../scripts/logger.ts"
import { hexToBytes } from "#convert.ts"
import { privateKeyToAccount } from "viem/accounts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/union-to-holesky.ts --private-key "..."` --estimate-gas */

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
  Uint8Array.from(hexToBytes(PRIVATE_KEY)),
  "stars"
)

try {
  const client = createUnionClient({
    account: cosmosAccount,
    chainId: "elgafar-1",
    gasPrice: { amount: "0.0025", denom: "ustars" },
    transport: http("https://rpc.elgafar-1.stargaze.chain.kitchen")
  })

  const transferPayload = {
    amount: 1n,
    denomAddress: "ustars",
    destinationChainId: `${holesky.id}`,
    receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"
  } satisfies TransferAssetsParameters<"elgafar-1">

  // const gasEstimationResponse = await client.simulateTransaction(transferPayload)

  // consola.box("Union to holesky gas cost:", gasEstimationResponse)

  // if (ONLY_ESTIMATE_GAS) process.exit(0)

  // if (!gasEstimationResponse.isOk()) {
  //   console.info("Transaction simulation failed")
  //   process.exit(1)
  // }

  const transfer = await client.transferAsset(transferPayload)

  if (transfer.isErr()) {
    console.error(transfer.error)
    process.exit(1)
  }

  consola.info(transfer.value)
  process.exit(0)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
