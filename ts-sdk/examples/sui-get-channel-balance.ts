import { SuiClientOptions, getFullnodeUrl } from '@mysten/sui/client';
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import { SuiPublicClientDestination, createSuiPublicClient } from "../src/sui/client.js"
import { channelBalance } from "../src/sui/channel-balance.js"
import { assert } from 'console';

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

function bytesToHex(bs: number[]): `0x${string}` {
  return "0x" + bs.map(b => b.toString(16).padStart(2, "0")).join("");
}

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet")
    }

    const publicClient = yield* createSuiPublicClient(config)
    
    const token_address = "0x6d756e6f"
    const relay_store = "0x87c6df16da471be8a9abe4228b2b20673897e75e164419fec03cf45422f23646"
    const token = "0xacc51178ffc547cdfa36a8ab4a6ae3823edaa8f07ff9177d9d520aad080b28fd"
    const path = 0;

    const result = yield* channelBalance(path, token, relay_store).pipe(
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0x0b04d5df6eaa283cc3f4ca62860221329d7b0c76e27828b48694b62c81a19a22",
        channelId: 2,
      }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
    )

    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
