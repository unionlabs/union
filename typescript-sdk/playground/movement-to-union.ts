import { http } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { createUnionClient } from "#mod.ts"

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
const WRAPPED_MUNO_DENOM = "0xe464e3224064bab061d6dc1602ba74a1f15768d1fb40e4051b968a8f071641dd"
const AMOUNT = 1n
const SOURCE_CHAIN_ID = "177"
const DESTINATION_CHAIN_ID = "union-testnet-9"

const RECEIVER =
  "756E696F6E31336B6B73687A74716C79396D63326D6C366D66676D716B6566353676366A656B6D7A37777934"
/* 
--------------------------------------------------------
-------------- HOW TO CALCULATE RECEIVER :-------------- 
--------------------------------------------------------

~/dev/union (movement_integration) ✗) $ printf "%s" union13kkshztqly9mc2ml6mfgmqkef56v6jekmz7wy4  | xxd -p -u -c 10000
756E696F6E31336B6B73687A74716C79396D63326D6C366D66676D716B6566353676366A656B6D7A37777934
*/

const channels = await getRecommendedChannels()

let channel_info = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel_info === null) {
  // Creating movement channel since its not found in hubble.
  consola.info("no channel found")
  // process.exit(1)
  channel_info = {
    source_chain_id: SOURCE_CHAIN_ID,
    source_port_id: "cebf41f47bdde649131694af50980b70ecd328d88a9b6c2993dae6405b0c88ed",
    source_channel_id: 5,
    source_connection_id: 15,
    destination_chain_id: DESTINATION_CHAIN_ID,
    destination_port_id:
      "756e696f6e3178326a7a65757037757766786a78787274666e61326b746375676c746e746775366b766330656561796b306438326c32343763717a3636396565",
    destination_channel_id: 7,
    destination_connection_id: 11
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
  transport: http("https://aptos.testnet.porto.movementlabs.xyz/v1")
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
