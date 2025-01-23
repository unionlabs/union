import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"
import { createUnionClient } from "#mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"
import { evmApproveTransferAsset } from "#evm/transfer"

const cliArgs = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = cliArgs.values["private-key"]
const LINK_CONTRACT_ADDRESS: `0x${string}` = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B"
const AMOUNT = 13n
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"
const SOURCE_CHAIN_ID = "17000"
const DESTINATION_CHAIN_ID = "11155111"

const channels = await getRecommendedChannels()

const channel = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel === null) {
  consola.error("no channel found")
  process.exit(1)
}

consola.info("channel", channel)

const quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, LINK_CONTRACT_ADDRESS, channel)

if (quoteToken.isErr()) {
  consola.error("could not get quote token")
  process.exit(1)
}

consola.info("quote token", quoteToken.value)

const holeskyClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: privateKeyToAccount(`0x${PRIVATE_KEY}`),
  transport: fallback([
    http("https://rpc.17000.holesky.chain.kitchen"),
    http(holesky?.rpcUrls.default.http.at(0))
  ])
})

const transferArgs = {
  baseToken: LINK_CONTRACT_ADDRESS,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel.source_channel_id,
  ucs03address: `0x${channel.source_port_id}`
} as const

const approveResponse = await holeskyClient.approveErc20(transferArgs)

if (approveResponse.isErr()) {
  consola.error(approveResponse.error)
  process.exit(1)
}

consola.info("approval tx hash", approveResponse.value)

const transfer = await holeskyClient.transferAsset(transferArgs)

if (transfer.isErr()) {
  consola.error(transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
