import { Effect } from "effect"
import { quoteToken } from "../src/evm/quote-token.js"
import { ViemPublicClientDestination } from "../src/evm/client.js"
import { createPublicClient, http, toHex } from "viem"
import { sepolia } from "viem/chains"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}
const client = createPublicClient({
  chain: sepolia,
  transport: http()
})

quoteToken({
  baseToken: toHex("muno"),
  ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
  destinationChannelId: 9
})
  .pipe(Effect.provideService(ViemPublicClientDestination, { client }), Effect.runPromiseExit)
  .then(e => console.log(JSON.stringify(e, null, 2)))
