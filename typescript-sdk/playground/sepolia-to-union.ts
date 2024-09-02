#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/sepolia-to-union.ts --private-key "..."` --estimate-gas */

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

const LINK_CONTRACT_ADDRESS = "0x779877A7B0D9E8603169DdbD7836e478b4624789"
const wOSMO_CONTRACT_ADDRESS = "0x3C148Ec863404e48d88757E88e456963A14238ef"
const USDC_CONTRACT_ADDRESS = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"

try {
  const client = createUnionClient({
    chainId: "11155111",
    account: evmAccount,
    transport: fallback([
      http("https://sepolia.infura.io/v3/238b407ca9d049829b99b15b3fd99246"),
      http(
        "https://special-summer-film.ethereum-sepolia.quiknode.pro/3e6a917b56620f854de771c23f8f7a8ed973cf7e"
      ),
      http("https://eth-sepolia.g.alchemy.com/v2/daqIOE3zftkyQP_TKtb8XchSMCtc1_6D"),
      http(sepolia?.rpcUrls.default.http.at(0))
    ])
  })

  const transactionPayload = {
    amount: 1n,
    autoApprove: true,
    denomAddress: LINK_CONTRACT_ADDRESS,
    destinationChainId: "union-testnet-8",
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  } satisfies TransferAssetsParameters<"11155111">

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  consola.box("Sepolia to Union gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    consola.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  consola.success("Sepolia to Union gas cost:", gasEstimationResponse)

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
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
