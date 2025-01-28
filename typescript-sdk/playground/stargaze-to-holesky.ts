import { fromHex, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { createUnionClient, hexToBytes } from "#mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

// hack to encode bigints to json
declare global {
  interface BigInt {
    toJSON: () => string
  }
}

if (!BigInt.prototype.toJSON) {
  Object.defineProperty(BigInt.prototype, "toJSON", {
    value: function () {
      return this.toString()
    },
    writable: true,
    configurable: true
  })
}
// end hack

const cliArgs = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = cliArgs.values["private-key"]
const STARS_DENOM = "ustars"
const AMOUNT = 420n
const RECEIVER = "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
const SOURCE_CHAIN_ID = "elgafar-1"
const DESTINATION_CHAIN_ID = "17000"

const channels = await getRecommendedChannels()

const channel = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel === null) {
  consola.info("no channel found")
  process.exit(1)
}

consola.info("channel", channel)

const quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, STARS_DENOM, channel)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  consola.error(quoteToken.error)
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

const transfer = await stargazeClient.transferAsset({
  baseToken: STARS_DENOM,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel.source_channel_id,
  ucs03address: fromHex(`0x${channel.source_port_id}`, "string")
})

if (transfer.isErr()) {
  consola.info("transfer submission failed")
  consola.error(transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
