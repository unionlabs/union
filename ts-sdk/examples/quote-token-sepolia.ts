import { Effect } from "effect"
import { DestinationConfig, predictQuoteToken } from "../src/evm/quote-token"
import { ViemPublicClientDestination } from "../src/evm/client"
import { createPublicClient, http, toHex } from "viem"
import { sepolia } from "viem/chains"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

Effect.runPromiseExit(
  predictQuoteToken(toHex("muno")).pipe(
    Effect.provideService(DestinationConfig, {
      ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
      channelId: 1
    }),
    Effect.provideService(ViemPublicClientDestination, {
      client: createPublicClient({
        chain: sepolia,
        transport: http()
      })
    })
  )
).then(exit => console.log(JSON.stringify(exit, null, 2)))
