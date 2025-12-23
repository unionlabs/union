// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}

import { getFullnodeUrl } from "@mysten/sui/client"
import { Effect, Logger } from "effect"

import { Sui } from "@unionlabs/sdk-sui"

const ADDRESS = process.env.ADDRESS
  ?? "0x03ff9dd9e093387bdd4432c6a3eb6a1bd5a8f39a530042ac7efe576f18d3232b"

const COIN_TYPE = "0x2::sui::SUI" as any

const program = Effect.gen(function*() {
  const { client } = yield* Sui.PublicClient
  yield* Effect.log("Sui public client initialized", client.network)

  const meta = yield* Sui.readCoinMetadata(COIN_TYPE)
  yield* Effect.log("SUI metadata", meta)

  const [name, symbol, decimals] = yield* Effect.all([
    Sui.getCoinName(COIN_TYPE),
    Sui.readCoinSymbol(COIN_TYPE),
    Sui.getCoinDecimals(COIN_TYPE),
  ])
  yield* Effect.log("SUI meta (granular)", { name, symbol, decimals })

  yield* Effect.log("Address", ADDRESS)
  const coins = yield* Sui.readCoinBalances(COIN_TYPE, ADDRESS as any)
  yield* Effect.log("SUI coins (objects)", coins)

  const total = yield* Sui.readTotalCoinBalance(COIN_TYPE, ADDRESS as any)
  yield* Effect.log("SUI total balance (mist as BigInt)", total.toString())

  const unique = yield* Sui.getAllCoinsUnique(ADDRESS as any)

  yield* Effect.log("All coins (unique, summed)", unique)
}).pipe(
  Effect.provide([
    Sui.PublicClient.Live({ url: getFullnodeUrl("testnet") }),
    Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault),
  ]),
)

Effect.runPromise(program).catch(console.error)
