import { fromHex, http, toHex } from "viem"
import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import { createUnionClient, hexToBytes } from "#mod.ts"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { queryContractState } from "#query/on-chain"

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
const WRASPPED_MUNO_DENOM_CW20 = "bbn1e9ycc775kxv7klq5eh9vznjslps3tqt3f2ttku8ptky9qqt6ecjqn570rp"
const AMOUNT = 12n
const RECEIVER = toHex("union1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwpzq6pr4")
const SOURCE_CHAIN_ID = "bbn-test-5"
const DESTINATION_CHAIN_ID = "union-testnet-9"

const baseToken = toHex(WRASPPED_MUNO_DENOM_CW20)

const channels = await getRecommendedChannels()

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
  account: await DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "bbn"),
  gasPrice: { amount: "0.025", denom: "ubbn" },
  transport: http("https://rpc.bbn-test-5.babylon.chain.kitchen")
})

const CW20_TOKEN_MINTER = "bbn143365ksyxj0zxj26djqsjltscty75qdlpwry6yxhr8ckzhq92xas8pz8sn"

const allowanceParams = {
  contractAddress: WRASPPED_MUNO_DENOM_CW20,
  amount: AMOUNT,
  spender: CW20_TOKEN_MINTER
}
consola.info("allowance params", allowanceParams)

const approveResponse = await unionClient.cw20IncreaseAllowance(allowanceParams)
consola.info("approval", approveResponse)

let contractSTate = await queryContractState({
  restUrl: "https://rest.bbn-test-5.babylon.chain.kitchen",
  contractAddress: WRASPPED_MUNO_DENOM_CW20
})
consola.log("contract state", contractSTate)

if (approveResponse.isErr()) {
  consola.error(approveResponse.error)
  process.exit(1)
}

consola.info("approval tx hash", approveResponse.value)

const transfer = await unionClient.transferAsset({
  baseToken: WRASPPED_MUNO_DENOM_CW20,
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
