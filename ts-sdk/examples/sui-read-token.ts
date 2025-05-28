import { SuiClientOptions, getFullnodeUrl } from '@mysten/sui/client';
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import { SuiPublicClient, SuiPublicClientDestination, createSuiPublicClient } from "../src/sui/client.js"
import { readCoinBalances, readTotalCoinBalance } from "../src/sui/sui_coin.js"
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
    
    const token_address = "0xb152050bf26f6e49ad4367de0cc409d99408c4d92edf442d36bb005a08de32c8::fungible_token::FUNGIBLE_TOKEN";
    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779";
    const is_native = false;
    const result = yield* readCoinBalances(token_address, user_address, is_native).pipe(
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
    
    const token_address = "0xb152050bf26f6e49ad4367de0cc409d99408c4d92edf442d36bb005a08de32c8::fungible_token::FUNGIBLE_TOKEN";
    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779";
    const is_native = false;
    const result = yield* readTotalCoinBalance(token_address, user_address, is_native).pipe(
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
    
    const token_address = "0x2::sui::SUI";
    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779";
    const is_native = true;
    const result = yield* readTotalCoinBalance(token_address, user_address, is_native).pipe(
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
      Effect.catchAllCause(cause =>
        Effect.logError("Failed to read coin balance:", cause)
      ),
    )
    yield* Effect.log("Result:", result)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
