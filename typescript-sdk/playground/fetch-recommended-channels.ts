import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { raise } from "#utilities/index.ts"
import { privateKeyToAccount } from "viem/accounts"
import { holesky, sepolia } from "viem/chains"
import { createUnionClient, type TransferAssetsParameters } from "#mod.ts"
import { getChannelInfo, getRecommendedChannels } from "#query/offchain/ucs03-channels"
import { evmApproveTransferAsset } from "#evm/transfer"

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]

const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)

const client = createUnionClient({
  chainId: "17000",
  account: evmAccount,
  transport: fallback([
    http("https://rpc.holesky.sepolia.chain.kitchen"),
    http(holesky?.rpcUrls.default.http.at(0))
  ])
})

let channels = await getRecommendedChannels()

const LINK_CONTRACT_ADDRESS = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B"

let channel = getChannelInfo("17000", "11155111", channels)

if (channel === null) {
  console.log("no channel found")
  process.exit(1)
}

const approveResponse = await evmApproveTransferAsset(client, {
  amount: 1n,
  denomAddress: LINK_CONTRACT_ADDRESS,
  receiver: `0x${channel.source_port_id}`
})

if (approveResponse.isErr()) {
  consola.error(approveResponse)
}

consola.info("approval", approveResponse.value)

const transfer = await client.transferAssetNew({
  baseToken: LINK_CONTRACT_ADDRESS,
  baseAmount: 1n,
  quoteToken: LINK_CONTRACT_ADDRESS, //wrong!
  quoteAmount: 1n,
  receiver: "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177",
  sourceChannelId: channel.source_channel_id,
  ucs03address: `0x${channel.source_port_id}`
})

if (transfer.isErr()) {
  console.error(transfer.error)
  process.exit(1)
}

consola.info(transfer.value)
