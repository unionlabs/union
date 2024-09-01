#!/usr/bin/env bun
import { fallback, getAddress, http } from "viem"
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import {
  createUnionClient,
  createPfmMemo,
  offchainQuery,
  type TransferAssetsParameters
} from "#mod.ts"

/* `bun playground/sepolia-to-berachain.ts --private-key "..."` */

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
  const {
    data: [sepoliaInfo]
  } = await offchainQuery.chains({
    includeContracts: true,
    includeEndpoints: true
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
      http("https://eth-sepolia.g.alchemy.com/v2/daqIOE3zftkyQP_TKtb8XchSMCtc1_6D"),
      http(sepolia?.rpcUrls.default.http.at(0))
    ])
  })

  const pfmMemo = createPfmMemo({
    channel: "channel-80",
    receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    port: "wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h"
  })

  if (pfmMemo.isErr()) {
    consola.error(pfmMemo.error)
    process.exit(1)
  }

  const transactionPayload = {
    amount: 1n,
    memo: pfmMemo.value,
    sourceChannel: channel_id,
    destinationChainId: destination_chain.chain_id,
    relayContractAddress: getAddress(contract_address),
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd", // "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789" // LINK
  } satisfies TransferAssetsParameters<"11155111">

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
