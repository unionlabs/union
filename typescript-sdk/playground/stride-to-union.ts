#!/usr/bin/env bun
import {
  cosmosHttp,
  offchainQuery,
  createCosmosSdkClient,
  type TransferAssetsParameters
} from "#mod.ts"
import { parseArgs } from "node:util"
import { raise } from "#utilities/index.ts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/stride-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")
const ONLY_ESTIMATE_GAS = values["estimate-gas"] ?? false

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "stride"
)
const [account] = await cosmosAccount.getAccounts()
console.info(account?.address)

try {
  const {
    data: [strideTestnetInfo]
  } = await offchainQuery.chain({
    includeContracts: true,
    chainId: "stride-internal-1"
  })

  if (!strideTestnetInfo) raise("Stride testnet info not found")

  const ucsConfiguration = strideTestnetInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const client = createCosmosSdkClient({
    // @ts-expect-error
    evm: {},
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "strd" },
      transport: cosmosHttp(
        //
        // "https://stride.testnet-1.stridenet.co/"
        "https://stride-testnet-rpc.polkachu.com/"
      )
    }
  })

  const transactionPayload = {
    amount: 1n,
    denomAddress: "strd",
    network: strideTestnetInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  } satisfies TransferAssetsParameters

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  console.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset(transactionPayload)

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
