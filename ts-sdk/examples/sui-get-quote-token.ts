import { SuiClientOptions, getFullnodeUrl } from '@mysten/sui/client';
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import { SuiPublicClientDestination, createSuiPublicClient } from "../src/sui/client.js"
import { predictQuoteToken } from "../src/sui/quote-token.js"
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
    
    yield* Effect.log("Public client created:", publicClient)

    const token_address = "0x6d756e6f"

    const result = yield* predictQuoteToken(token_address).pipe(
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0x4429fafa71bf9730c9fda4a5ce6772aecb1ac6a1ef0522812949f71d241332b5",
        channelId: 2,
      }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
    )
    const expected_result = "0x204c18721deddf1ea8ec97b187aaf067c09111f350d956cb624b7b4002f0c5e246"
    assert(bytesToHex(result) === expected_result, `Expected wrapped token to be ${expected_result}, but got ${result}`)


    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
