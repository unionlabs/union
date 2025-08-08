/**
 * @title Get Quote Token
 * @badge WIP:caution
 */
/// <reference types="effect" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
import * as Sui from "@unionlabs/sdk/Sui"
import { Effect, pipe } from "effect"
import { assert } from "effect/Console"

const TOKEN_ADDRESS = "0x6d756e6f"
const EXPECTED_RESULT = "0x4c18721deddf1ea8ec97b187aaf067c09111f350d956cb624b7b4002f0c5e246"

const PublicClientDestination = Sui.PublicClientDestination.FromNode("testnet")
const ChannelDestination = Sui.ChannelDestination.of({
  ucs03address: "0xbd3eb037fda14d910c5574b8c950222cc5c4211675b9c2265c07a66cef3a7691",
  channelId: 2,
})

Effect.runPromiseExit(
  Effect.gen(function*() {
    const quoteToken = pipe(
      Sui.predictQuoteToken(TOKEN_ADDRESS),
      Effect.provide(PublicClientDestination),
      Effect.provideService(Sui.ChannelDestination, ChannelDestination),
    )

    const result = yield* quoteToken

    yield* assert!(
      result === EXPECTED_RESULT,
      `Expected wrapped token to be ${EXPECTED_RESULT}, but got ${result}`,
    )
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
