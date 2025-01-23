import { fallback, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"
import { createUnionClient, hexToBytes } from "#mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"
import { evmApproveTransferAsset } from "#evm/transfer"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

const cliArgs = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = cliArgs.values["private-key"]
const LINK_CONTRACT_ADDRESS = "ustars"
const AMOUNT = 13n
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"
const SOURCE_CHAIN_ID = "elgafar-1"
const DESTINATION_CHAIN_ID = "17000"

const channels = await getRecommendedChannels()

const channel = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel === null) {
  consola.info("no channel found")
  process.exit(1)
}

consola.info("channel", channel)

const quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, LINK_CONTRACT_ADDRESS, channel)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  process.exit(1)
}

consola.info("quote token", quoteToken.value)

if (!PRIVATE_KEY) {
  consola.error("no private key provided")
  process.exit(1)
}

const stargazeClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: await DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "stars"),
  gasPrice: { amount: "0.025", denom: "ustars" },
  transport: http("https://rpc.elgafar-1.stargaze.chain.kitchen")
})

// no approval required
//

const transfer = await stargazeClient.transferAssetNew({
  baseToken: LINK_CONTRACT_ADDRESS,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel.source_channel_id,
  ucs03address: `0x${channel.source_port_id}`
})

if (transfer.isErr()) {
  console.error(transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
