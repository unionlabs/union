/**
 * @title Get All Coins
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

const publicClient = Sui.PublicClient.FromNode("testnet")

const program = Sui.getAllCoins(
  "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
).pipe(
  Effect.provide(publicClient),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
