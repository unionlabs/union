/**
 * @title Get Channel Balance
 * @badge WIP:caution
 */
/// <reference types="effect" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
import { Sui } from "@unionlabs/sdk"
import { Effect } from "effect"

const publicClient = Sui.PublicClientDestination.FromNode("testnet")
const destinationChannel = Sui.ChannelDestination.Live({
  ucs03address: "0x0b04d5df6eaa283cc3f4ca62860221329d7b0c76e27828b48694b62c81a19a22",
  channelId: 2,
})

const program = Effect.gen(function*() {
  const relay_store = "0x87c6df16da471be8a9abe4228b2b20673897e75e164419fec03cf45422f23646"
  const token = "0xacc51178ffc547cdfa36a8ab4a6ae3823edaa8f07ff9177d9d520aad080b28fd"
  const path = 0

  return yield* Sui.channelBalance(path, token, relay_store)
}).pipe(
  Effect.provide(publicClient),
  Effect.provide(destinationChannel),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
