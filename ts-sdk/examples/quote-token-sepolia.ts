import { Effect } from "effect"
import { quoteToken } from "../src/evm/quote-token"
import { ViemPublicClientDestination } from "../src/evm/client"
import { createPublicClient, defineChain, http, toHex } from "viem"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const devnet = defineChain({
  id: 32382,
  name: "ethereum devnet",
  nativeCurrency: {
    decimals: 18,
    name: "Ether",
    symbol: "ETH"
  },
  rpcUrls: {
    default: {
      http: ["http://localhost:8545"]
    }
  },
  testnet: true
})

const client = createPublicClient({
  chain: devnet,
  transport: http()
})

Effect.runPromiseExit(
  quoteToken({
    baseToken: toHex("muno"),
    ucs03address: "0x05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5",
    destinationChannelId: 1
  }).pipe(Effect.provideService(ViemPublicClientDestination, { client }))
).then(exit => console.log(JSON.stringify(exit, null, 2)))
