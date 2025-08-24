/**
 * @title Read Coin Metadata
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

const program = Effect.gen(function*() {
  const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"
  const uniqueCoins = yield* Sui.getAllCoinsUnique(user_address).pipe(
    Effect.provide(publicClient),
    Effect.tapErrorCause((cause) => Effect.logError("Predict failed", cause)),
  )
  for (const { coinType, balance } of uniqueCoins) {
    console.info(`Coin Type: ${coinType}, Balance: ${balance}`)
    const metadata = yield* Sui.readCoinMetadata(coinType).pipe(
      Effect.catchAllCause(cause => Effect.logError("Failed to read coin balance:", cause)),
    )
    yield* Effect.log(`Coin: ${metadata!.name} (${metadata!.symbol})`)
    yield* Effect.log(`Decimals: ${metadata!.decimals}`)
    yield* Effect.log(`Balance: ${balance}`)
    yield* Effect.log("------------------------")
  }
}).pipe(
  Effect.provide(publicClient),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
