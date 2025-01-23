import { parseArgs } from "node:util"
import { consola } from "scripts/logger"
import {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels
} from "#query/offchain/ucs03-channels"

const { values } = parseArgs({
  args: process.argv.slice(2),
  options: {
    "private-key": { type: "string" },
    "estimate-gas": { type: "boolean", default: false }
  }
})

const PRIVATE_KEY = values["private-key"]

// const evmAccount = privateKeyToAccount(`0x${PRIVATE_KEY}`)
// const sepoliaClient = createUnionClient({
//   chainId: "11155111",
//   account: evmAccount,
//   transport: fallback([
//     http("https://rpc.11155111.sepolia.chain.kitchen"),
//     http(holesky?.rpcUrls.default.http.at(0))
//   ])
// })

let channels = await getRecommendedChannels()

const LINK_CONTRACT_ADDRESS = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B"
const LINK_CONTRACT_ADDRESS_SEPOLIA = "0x44799296211290262fd6b22a07a0fa13414caddc"

let channel = getChannelInfo("11155111", "17000", channels)
if (channel === null) {
  consola.info("no channel found")
  process.exit(1)
}
consola.info({ channel })
let quoteToken = await getQuoteToken("11155111", LINK_CONTRACT_ADDRESS_SEPOLIA, channel)
if (quoteToken.isErr()) {
  consola.info("could not get quote token")
  process.exit(1)
}

console.log(JSON.stringify(quoteToken.value))

let channel2 = getChannelInfo("17000", "11155111", channels)
if (channel2 === null) {
  consola.info("no channel found")
  process.exit(1)
}
consola.info({ channel })
let quoteToken2 = await getQuoteToken("17000", LINK_CONTRACT_ADDRESS, channel2)
if (quoteToken2.isErr()) {
  consola.info("could not get quote token")
  process.exit(1)
}

console.log(JSON.stringify(quoteToken2.value))

/**
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
*/
