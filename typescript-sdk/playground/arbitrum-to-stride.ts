#!/usr/bin/env bun
import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { arbitrumSepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/arbitrum-to-stride.ts --private-key "..."` --estimate-gas */

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

const explorerURL = "https://scope.sh/421614/tx"

try {
  const client = createUnionClient({
    account: evmAccount,
    chainId: `${arbitrumSepolia.id}`,
    transport: http(arbitrumSepolia?.rpcUrls.default.http.at(0))
  })

  const transactionPayload = {
    amount: 1n,
    autoApprove: false,
    denomAddress: LINK_CONTRACT_ADDRESS,
    destinationChainId: "stride-internal-1",
    // or `client.cosmos.account.address` if you want to send to yourself
    receiver: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
  } satisfies TransferAssetsParameters<"80084">

  // const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  // if (gasEstimationResponse.isErr()) {
  //   consola.error(gasEstimationResponse.error)
  //   process.exit(1)
  // }

  // consola.success("Arbitrum to Stride gas estimation:", gasEstimationResponse.value)
  // if (ONLY_ESTIMATE_GAS) process.exit(0)

  const approval = await client.approveTransaction(transactionPayload)
  if (approval.isErr()) {
    consola.error("Approval failed", approval.error)
    process.exit(1)
  }

  consola.info("approve:", `${explorerURL}/${approval.value}`)

  const transfer = await client.transferAsset(transactionPayload)

  if (transfer.isErr()) {
    consola.error("Transfer failed", transfer.error)
    process.exit(1)
  }

  consola.info("transfer:", `${explorerURL}/${transfer.value}`)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
