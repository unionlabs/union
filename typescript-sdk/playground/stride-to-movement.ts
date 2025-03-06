import { fromHex, http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { createUnionClient, hexToBytes } from "#mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels //,
  // Channel
} from "#query/offchain/ucs03-channels"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"

type Channel = {
  source_chain_id: string
  source_port_id: string
  source_channel_id: number
  source_connection_id: number
  destination_chain_id: string
  destination_port_id: string
  destination_channel_id: number
  destination_connection_id: number
}

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
const MUNO_DENOM = "ustrd"
const MUNO_HEX = "0x7573747264"
const AMOUNT = 1n
const RECEIVER = "0x4f597b9ac27cc279a66f4963c7955861955604eab8b38dcffee4cbcb7756e4d8"
const SOURCE_CHAIN_ID = "stride-internal-1"
const DESTINATION_CHAIN_ID = "250"

const channels = await getRecommendedChannels()

let channel_info = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)

if (channel_info === null) {
  consola.info("no channel found")
  channel_info = {
    source_chain_id: SOURCE_CHAIN_ID,
    source_port_id: "7374726964653178326a7a65757037757766786a78787274666e61326b746375676c746e746775366b766330656561796b306438326c3234376371333570727573",
    source_channel_id: 5,
    source_connection_id: 333333333,
    destination_chain_id: DESTINATION_CHAIN_ID,
    destination_port_id:
      "80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84",
    destination_channel_id: 4,
    destination_connection_id: 333333333
  }
}
consola.info("channel", channel_info)

let quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, MUNO_HEX, channel_info

)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  consola.error(quoteToken.error)
  process.exit(1)
}
console.info("quote token", quoteToken)

if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
  consola.error("No quote token available")
  process.exit(1)
}
consola.info("quote token", quoteToken.value)

if (!PRIVATE_KEY) {
  consola.error("no private key provided")
  process.exit(1)
}


const unionClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: await DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "stride"),
  gasPrice: { amount: "0.025", denom: "ustrd" },
  transport: http("https://rpc.stride-internal-1.stride.chain.cooking")
})

const transfer = await unionClient.transferAsset({
  baseToken: MUNO_DENOM,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel_info.source_channel_id,
  ucs03address: fromHex(`0x${channel_info.source_port_id}`, "string")
})

if (transfer.isErr()) {
  consola.error("transfer submission failed:", transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
