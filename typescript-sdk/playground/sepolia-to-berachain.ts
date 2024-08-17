#!/usr/bin/env bun
import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { berachainTestnetbArtio, sepolia } from "viem/chains"
import { createUnionClient, offchainQuery, type TransferAssetsParameters } from "#mod.ts"

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
  } = await offchainQuery.chain({
    chainId: "11155111",
    includeContracts: true,
    includeEndpoints: true
  })
  if (!sepoliaInfo) raise("Sepolia info not found")

  const ucsConfiguration = sepoliaInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const forward = ucsConfiguration.forward.find(
    item => item.destination_chain.chain_id === `${berachainTestnetbArtio.id}`
  )

  if (!forward) raise("Forward configuration not found")

  const client = createUnionClient({
    evm: {
      chain: sepolia,
      account: evmAccount,
      transport: fallback([http(sepolia?.rpcUrls.default.http.at(0))], {
        rank: true,
        retryCount: 3
      })
    }
  })

  const pfmMemo = client.createPfmMemo({
    port: forward.port,
    channel: forward.channel_id,
    receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd"
  })

  const transferAssetsParameters = {
    amount: 1n,
    memo: pfmMemo,
    approve: true,
    evmSigner: evmAccount.address,
    network: sepoliaInfo.rpc_type,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    recipient: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd", // "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789", // LINK
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  } satisfies TransferAssetsParameters

  consola.info(`Payload ${JSON.stringify(transferAssetsParameters, undefined, 2)}`)

  const gasEstimationResponse = await client.simulateTransaction(transferAssetsParameters)

  consola.box("Sepolia to Berachain gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset(transferAssetsParameters)

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
