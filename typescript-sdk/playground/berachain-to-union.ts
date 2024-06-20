#!/usr/bin/env bun
import { erc20Abi, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { berachainTestnetbArtio } from "viem/chains"
import { createUnionClient, offchainQuery } from "#mod.ts"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

/* `bun playground/berachain-to-union.ts --private-key "..."` */

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: { "private-key": { type: "string" } }
})
erc20Abi
const PRIVATE_KEY = values["private-key"]
if (!PRIVATE_KEY) throw new Error("Private key not found")

const berachainAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
  Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
  "union"
)

const WBTC_CONTRACT_ADDRESS = "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae"

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [beraInfo]
  } = await offchainQuery.chain({
    chainId: "80084",
    includeContracts: true
  })
  if (!beraInfo) raise("Berachain info not found")

  const ucsConfiguration = beraInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)
  if (!ucsConfiguration) raise("UCS configuration not found")

  const { channel_id, contract_address, source_chain, destination_chain } = ucsConfiguration

  const client = createUnionClient({
    evm: {
      account: berachainAccount,
      chain: berachainTestnetbArtio,
      transport: http(
        "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
      )
    },
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "muno" },
      transport: cosmosHttp("https://rpc.testnet.bonlulu.uno")
    }
  })

  const gasEstimationResponse = await client.simulateTransaction({
    amount: 1n,
    sourceChannel: channel_id,
    network: beraInfo.rpc_type,
    relayContractAddress: contract_address,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae", // wBTC
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  consola.info(`Gas cost: ${gasEstimationResponse.data}`)

  if (!gasEstimationResponse.success) {
    console.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset({
    amount: 1n,
    sourceChannel: channel_id,
    network: beraInfo.rpc_type,
    relayContractAddress: contract_address,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv",
    denomAddress: "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae", // wBTC
    path: [source_chain.chain_id, destination_chain.chain_id]
  })

  console.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  console.error(errorMessage)
} finally {
  process.exit(0)
}
