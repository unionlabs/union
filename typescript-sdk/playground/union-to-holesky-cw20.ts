import { fromHex, http, toHex } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { createUnionClient, hexToBytes } from "../src/mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "../src/query/offchain/ucs03-channels.ts"
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
const CW20_DENOM = "union10y75w84ecnqwx4v8xdn00tppgxckxeu80n3nhy8qdt66slhrtevs789d4k"
const AMOUNT = 12n
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"
const SOURCE_CHAIN_ID = "union-testnet-9"
const DESTINATION_CHAIN_ID = "17000"

const baseToken = toHex(CW20_DENOM)

const channels = await getRecommendedChannels()

consola.log(channels)

const channel = getChannelInfo(SOURCE_CHAIN_ID, DESTINATION_CHAIN_ID, channels)
if (channel === null) {
  consola.info("no channel found")
  process.exit(1)
}

consola.info("channel", channel)

consola.info("base token", baseToken)

const quoteToken = await getQuoteToken(SOURCE_CHAIN_ID, baseToken, channel)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  consola.error(quoteToken.error)
  process.exit(1)
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

const unionClient = createUnionClient({
  chainId: SOURCE_CHAIN_ID,
  account: await DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "union"),
  gasPrice: { amount: "0.025", denom: "muno" },
  transport: http("https://rpc.testnet-9.union.build")
})

const allowanceParams = {
  contractAddress: CW20_DENOM,
  amount: AMOUNT,
  spender: fromHex(`0x${channel.source_port_id}`, "string") // found using read-contract-state.ts
}
consola.info("allowance params", allowanceParams)

const approveResponse = await unionClient.cw20IncreaseAllowance(allowanceParams)
consola.info("approval", approveResponse)

// let contractSTate = await queryContractState({
//   restUrl: "https://rest.bbn-test-5.babylon.chain.kitchen",
//   contractAddress: WRASPPED_MUNO_DENOM_CW20
// })
// consola.log("contract state", contractSTate)

if (approveResponse.isErr()) {
  consola.error(approveResponse.error)
  process.exit(1)
}

const transfer = await unionClient.transferAsset({
  baseToken: CW20_DENOM,
  baseAmount: AMOUNT,
  quoteToken: quoteToken.value.quote_token,
  quoteAmount: AMOUNT,
  receiver: RECEIVER,
  sourceChannelId: channel.source_channel_id,
  ucs03address: fromHex(`0x${channel.source_port_id}`, "string")
})

if (transfer.isErr()) {
  consola.error("transfer submission failed:", transfer.error)
  process.exit(1)
}

consola.info("transfer tx hash", transfer.value)
