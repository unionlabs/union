import { getFullnodeUrl, SuiClientOptions } from "@mysten/sui/client"
import { assert } from "console"
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import { createSuiPublicClient, SuiPublicClientDestination } from "../src/sui/client.js"
import { predictQuoteToken } from "../src/sui/quote-token.js"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }

    const publicClient = yield* createSuiPublicClient(config)

    yield* Effect.log("Public client created:", publicClient)

    const token_address = "0x6d756e6f"

    const result = yield* predictQuoteToken(token_address).pipe(
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0xbd3eb037fda14d910c5574b8c950222cc5c4211675b9c2265c07a66cef3a7691",
        channelId: 2,
      }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
    )
    const expected_result = "0x4c18721deddf1ea8ec97b187aaf067c09111f350d956cb624b7b4002f0c5e246"
    assert!(
      result === expected_result,
      `Expected wrapped token to be ${expected_result}, but got ${result}`,
    )
    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
