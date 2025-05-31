import { getFullnodeUrl, SuiClientOptions } from "@mysten/sui/client"
import { assert } from "console"
import { Effect } from "effect"
import { SuiChannelDestination } from "../src/sui/channel.js"
import {
  createSuiPublicClient,
  SuiPublicClient,
  SuiPublicClientDestination,
} from "../src/sui/client.js"
import {
  getAllCoins,
  getAllCoinsUnique,
  readCoinBalances,
  readCoinMetadata,
  readTotalCoinBalance,
} from "../src/sui/sui_coin.js"

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

    const user_address = "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779"
    const uniqueCoins = yield* getAllCoinsUnique(user_address).pipe(
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)),
      Effect.catchAllCause(cause => Effect.logError("Failed to read coin balance:", cause)),
    )
    for (const { coinType, balance } of uniqueCoins) {
      console.info(`Coin Type: ${coinType}, Balance: ${balance}`)
      const metadata = yield* readCoinMetadata(coinType).pipe(
        Effect.provideService(SuiPublicClient, { client: publicClient }),
        Effect.catchAllCause(cause => Effect.logError("Failed to read coin balance:", cause)),
      )
      yield* Effect.log(`Coin: ${metadata.name} (${metadata.symbol})`)
      yield* Effect.log(`Decimals: ${metadata.decimals}`)
      yield* Effect.log(`Balance: ${balance}`)
      yield* Effect.log("------------------------")
    }
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
