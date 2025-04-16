import { getAddress, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { bech32AddressToHex, createUnionClient } from "../src/mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "../src/query/offchain/ucs03-channels.ts"
import { privateKeyToAccount } from "viem/accounts"

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
const UBBN_DENOM = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B" // LINK<-Holesky
const AMOUNT = 1n
const RECEIVER = bech32AddressToHex({ address: "bbn14vcpe0yt8xdzaapm8yy6tm26sf45rdgu4u2ka9" })
const SOURCE_CHAIN_ID = "17000"
const DESTINATION_CHAIN_ID = "union-testnet-9"

const channels = await getRecommendedChannels()

const channel = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel === null) {
  consola.info("no channel found")
  process.exit(1)
}

consola.info("channel", channel)

const quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, UBBN_DENOM, channel)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  consola.error(quoteToken.error)
  process.exit(1)
}

if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
  consola.info("no quote token available")
  process.exit(1)
}

consola.info("quote token", quoteToken.value)

const transferArgs = {
  baseToken: getAddress(UBBN_DENOM),
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel.source_channel_id,
  ucs03address: getAddress(`0x${channel.source_port_id}`)
}

consola.info("transfer args", transferArgs)

if (!PRIVATE_KEY) {
  consola.error("no private key provided")
  process.exit(1)
}

const evmClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: privateKeyToAccount(`0x${PRIVATE_KEY}`),
  transport: http("https://rpc.17000.holesky.chain.kitchen")
})

const approveResponse = await evmClient.approveErc20(transferArgs)

if (approveResponse.isErr()) {
  consola.error(approveResponse.error)
  process.exit(1)
}

consola.info("approval tx hash", approveResponse.value)

const transfer = await evmClient.transferAsset(transferArgs)

if (transfer.isErr()) {
  consola.info("transfer submission failed")
  consola.error(transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
