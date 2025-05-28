import { SuiClientOptions, getFullnodeUrl } from '@mysten/sui/client';
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import { SuiPublicClient, SuiPublicClientDestination, createSuiPublicClient } from "../src/sui/client.js"
import { getAllCoins, getAllCoinsUnique } from "../src/sui/sui_coin.js"
import { assert } from 'console';

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet")
    }

    const publicClient = yield* createSuiPublicClient(config)
    
    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779";
    const result = yield* getAllCoins(user_address).pipe(
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
      Effect.catchAllCause(cause =>
        Effect.logError("Failed to read coin balance:", cause)
      ),
    )
    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))




Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet")
    }

    const publicClient = yield* createSuiPublicClient(config)
    
    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779";
    const result = yield* getAllCoinsUnique(user_address).pipe(
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
      Effect.catchAllCause(cause =>
        Effect.logError("Failed to read coin balance:", cause)
      ),
    )
    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))

