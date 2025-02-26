import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { createUnionClient, bech32AddressToHex } from "#mod.ts"

import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"

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

// HOW TO RUN:
// npx tsx playground/movement-to-union.ts --private-key

const PRIVATE_KEY = cliArgs.values["private-key"]
const WRAPPED_MUNO_DENOM = "0x188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c"
const AMOUNT = 1n
const SOURCE_CHAIN_ID = "250"
const DESTINATION_CHAIN_ID = "union-testnet-9"

const RECEIVER = bech32AddressToHex({ address: "bbn14vcpe0yt8xdzaapm8yy6tm26sf45rdgu4u2ka9" })
// const RECEIVER =
//   "756E696F6E31786434787A356E346371657638643378657270666A656367706565706C34667834323263676A"
/* 
--------------------------------------------------------
-------------- HOW TO CALCULATE RECEIVER :-------------- 
--------------------------------------------------------

~/dev/union (movement_integration) ✗) $ printf "%s" union1xd4xz5n4cqev8d3xerpfjecgpeepl4fx422cgj  | xxd -p -u -c 10000
756E696F6E31786434787A356E346371657638643378657270666A656367706565706C34667834323263676A
*/

const channels = await getRecommendedChannels()

let channel_info = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel_info === null || true) {
  // Creating movement channel since its not found in hubble.
  // consola.info("no channel found")
  // process.exit(1)
  channel_info = {
    source_chain_id: SOURCE_CHAIN_ID,
    source_port_id: "80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84",
    source_channel_id: 2,
    source_connection_id: 1,
    destination_chain_id: DESTINATION_CHAIN_ID,
    destination_port_id:
      "756e696f6e3178326a7a65757037757766786a78787274666e61326b746375676c746e746775366b766330656561796b306438326c32343763717a3636396565",
    destination_channel_id: 27,
    destination_connection_id: 36
  }
}

consola.info("channel", channel_info)

let quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, WRAPPED_MUNO_DENOM, channel_info)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  consola.error(quoteToken.error)
  process.exit(1)
}

// manual quote token:
quoteToken = {
  type: "UNWRAPPED",
  value: {
    quote_token: `0x6d756e6f`
  }
}

if (quoteToken.value.type === "NO_QUOTE_AVAILABLE") {
  consola.error("No quote token available")
  process.exit(1)
}
consola.info("quote token", quoteToken.value)

if (!PRIVATE_KEY) {
  consola.error("no private key provided")
  process.exit(1)
}

const privateKey = new Ed25519PrivateKey(PRIVATE_KEY)
const account = Account.fromPrivateKey({ privateKey })

const unionClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: account,
  transport: http("https://aptos.testnet.bardock.movementlabs.xyz/v1")
})

const transfer = await unionClient.transferAsset({
  baseToken: WRAPPED_MUNO_DENOM,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel_info.source_channel_id,
  ucs03address: `0x${channel_info.source_port_id}`
})

if (transfer.isErr()) {
  consola.error("transfer submission failed:", transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
