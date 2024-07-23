#!/usr/bin/env bun
import { parseArgs } from "node:util"
import { fallback, http } from "viem"
import { consola } from "scripts/logger"
import { cosmosHttp } from "#transport.ts"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { hexStringToUint8Array } from "#convert.ts"
import { berachainTestnetbArtio } from "viem/chains"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { createCosmosSdkClient, offchainQuery, type TransferAssetsParameters } from "#mod.ts"

/* `bun playground/berachain-to-stride.ts --private-key "..."` --estimate-gas */

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
if (!account) raise("Account not found")

const WBTC_CONTRACT_ADDRESS = "0x286F1C3f0323dB9c91D1E8f45c8DF2d065AB5fae"

try {
  /**
   * Calls Hubble, Union's indexer, to grab desired data that's always up-to-date.
   */
  const {
    data: [beraInfo]
  } = await offchainQuery.chain({
    chainId: "80084",
    includeContracts: true,
    includeEndpoints: true
  })

  if (!beraInfo) raise("Stride testnet info not found")

  const ucsConfiguration = beraInfo.ucs1_configurations
    ?.filter(config => config.destination_chain.chain_id === "union-testnet-8")
    .at(0)

  if (!ucsConfiguration) raise("UCS configuration not found")

  const forward = ucsConfiguration.forward.find(
    item => item.destination_chain.chain_id === "stride-internal-1"
  )

  if (!forward) raise("Forward configuration not found")

  const client = createCosmosSdkClient({
    evm: {
      account: berachainAccount,
      chain: berachainTestnetbArtio,
      transport: fallback([
        http(
          "https://autumn-solitary-bird.bera-bartio.quiknode.pro/3ddb9af57edab6bd075b456348a075f889eff5a7/"
        ),
        http(berachainTestnetbArtio?.rpcUrls.default.http.at(0))
      ])
    },
    cosmos: {
      account: cosmosAccount,
      gasPrice: { amount: "0.0025", denom: "strd" },
      transport: cosmosHttp("https://stride-testnet-rpc.polkachu.com")
    }
  })

  const pfmMemo = client.createPfmMemo({
    port: forward.port,
    channel: forward.channel_id,
    receiver: client.bech32AddressToHex({
      address: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3"
    })
  })

  const transactionPayload = {
    amount: 1n,
    memo: pfmMemo,
    approve: true,
    network: beraInfo.rpc_type,
    denomAddress: WBTC_CONTRACT_ADDRESS,
    sourceChannel: ucsConfiguration.channel_id,
    relayContractAddress: ucsConfiguration.contract_address,
    // or `client.cosmos.account.address` if you want to send to yourself
    recipient: "stride14qemq0vw6y3gc3u3e0aty2e764u4gs5l66hpe3",
    path: [ucsConfiguration.source_chain.chain_id, ucsConfiguration.destination_chain.chain_id]
  } satisfies TransferAssetsParameters

  // console.info(JSON.stringify(transactionPayload, undefined, 2))

  const gasEstimationResponse = await client.simulateTransaction(transactionPayload)

  consola.box("gas cost:", gasEstimationResponse)

  if (ONLY_ESTIMATE_GAS) process.exit(0)

  if (!gasEstimationResponse.success) {
    consola.info("Transaction simulation failed")
    process.exit(1)
  }

  const transfer = await client.transferAsset(transactionPayload)

  consola.info(transfer)
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : error
  consola.error(errorMessage)
} finally {
  process.exit(0)
}
