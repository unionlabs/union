/**
 * @title Read Token Balance
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

const program1 = Effect.gen(function*() {
  const token_address =
    "0xb152050bf26f6e49ad4367de0cc409d99408c4d92edf442d36bb005a08de32c8::fungible_token::FUNGIBLE_TOKEN"
  const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"
  const result = yield* Sui.readCoinBalances(token_address, user_address).pipe(
    Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
  )
  yield* Effect.log("Result:", result)
}).pipe(
  Effect.provide(publicClient),
)

Effect.runPromise(program1)
  .then(console.log)
  .catch(console.error)

const program2 = Effect.gen(function*() {
  const token_address =
    "0xb152050bf26f6e49ad4367de0cc409d99408c4d92edf442d36bb005a08de32c8::fungible_token::FUNGIBLE_TOKEN"
  const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"
  const result = yield* Sui.readTotalCoinBalance(token_address, user_address).pipe(
    Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
  )
  yield* Effect.log("Result:", result)
}).pipe(
  Effect.provide(publicClient),
)

Effect.runPromise(program2)
  .then(console.log)
  .catch(console.error)

const program3 = Effect.gen(function*() {
  const token_address = "0x2::sui::SUI" // native
  const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"
  const result = yield* Sui.readTotalCoinBalance(token_address, user_address).pipe(
    Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
  )
  yield* Effect.log("Result:", result)
}).pipe(
  Effect.provide(publicClient),
)

Effect.runPromise(program3)
  .then(console.log)
  .catch(console.error)
