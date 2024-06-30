#!/usr/bin/env bun
import { fallback, http } from "viem"
import { sepolia } from "viem/chains"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, offchainQuery } from "#mod.ts"

/**
 * WIP
 */

/* `bun playground/sepolia-to-berachain.ts --private-key "..."` */

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

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

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

  const client = createCosmosSdkClient({
    evm: {
      chain: sepolia,
      account: evmAccount,
      transport: fallback([
        http("https://eth-sepolia.g.alchemy.com/v2/daqIOE3zftkyQP_TKtb8XchSMCtc1_6D"),
        http(sepolia?.rpcUrls.default.http.at(0))
      ])
    },
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "muno" },
      transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
    }
  })

  const pfmMemo = client.createPfmMemo({
    channel: "channel-80",
    receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
    port: "wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h"
  })

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    memo: pfmMemo,
    sourceChannel: channel_id,
    evmSigner: evmAccount.address,
    network: sepoliaInfo.rpc_type,
    relayContractAddress: contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd", // "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789", // LINK
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  consola.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    amount: 1n,
    memo: pfmMemo,
    sourceChannel: channel_id,
    network: sepoliaInfo.rpc_type,
    relayContractAddress: contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd", //"union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789", // LINK
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
