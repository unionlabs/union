/**
 * @title ERC20 Predict Quote Token
 * @summary
 * Predict quote token on Sepolia.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
import { Evm } from "@unionlabs/sdk"
import { Effect } from "effect"
import { http, toHex } from "viem"
import { sepolia } from "viem/chains"

const client = Evm.PublicClientDestination.Live({
  chain: sepolia,
  transport: http(),
})

const destination = Evm.ChannelDestination.Live({
  ucs03address: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
  channelId: 1,
})

const program = Evm.predictQuoteToken(
  toHex("muno"),
).pipe(
  Effect.provide(client),
  Effect.provide(destination),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
