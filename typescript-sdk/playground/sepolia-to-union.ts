#!/usr/bin/env bun
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { fallback, getAddress, http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { createUnionClient, offchainQuery } from "#mod.ts"
import type { ApproveTransferAssetFromEvmParams } from "#transfer/evm.js"

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
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [sepoliaInfo]
  } = await offchainQuery.chain({
    chainId: "11155111",
    includeEndpoints: true,
    includeContracts: true
  })
  if (!sepoliaInfo) raise("Sepolia info not found")

  const ucsConfiguration = sepoliaInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const { channel_id, contract_address, source_chain, destination_chain } = ucsConfiguration

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

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    sourceChannel: channel_id,
    account: evmAccount.address,
    denomAddress: LINK_CONTRACT_ADDRESS,
    destinationChainId: destination_chain.chain_id,
    relayContractAddress: getAddress(contract_address),
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
  })

  consola.box("Sepolia to Union gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (gasEstimationResponse.isErr()) {
    consola.info("Transaction simulation failed", gasEstimationResponse.error)
    process.exit(1)
  }

  const approvalParams = {
    amount: 1n,
    simulate: true,
    account: evmAccount,
    denomAddress: LINK_CONTRACT_ADDRESS,
    relayContractAddress: getAddress(contract_address)
  } satisfies ApproveTransferAssetFromEvmParams

  const approvalTransfer = await client.approveTransaction(approvalParams)

  consola.success("Approval transaction:", approvalTransfer)

  if (approvalTransfer.isErr()) {
    consola.info("Approval transaction failed", approvalTransfer.error)
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    approve: false,
    sourceChannel: channel_id,
    destinationChainId: destination_chain.chain_id,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    ...approvalParams
  })

  if (transfer.isErr()) {
    consola.info("Transfer failed", transfer.error)
    process.exit(1)
  }

  consola.info("Transfer successful", transfer.value)
  process.exit(0)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
