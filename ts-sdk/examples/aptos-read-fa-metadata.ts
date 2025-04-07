import { Effect } from "effect"
import { AptosPublicClient, createAptosPublicClient } from "../src/aptos/client.js"
import { readFaName, readFaDecimals, readFaSymbol, readFaTokenInfo } from "../src/aptos/fa.js"
import { Account, Ed25519PrivateKey } from "@aptos-labs/ts-sdk"
import { AptosConfig, Network } from "@aptos-labs/ts-sdk"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// Replace with your private key
const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create account from private key
    const privateKey = new Ed25519PrivateKey(PRIVATE_KEY)
    const account = Account.fromPrivateKey({ privateKey })

    const rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"

    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    const publicClient = yield* createAptosPublicClient(config)

    const real_token_address = "0x19842c145c835df3bb4daa4fb3914bb9bdbd4ff4bcf53b53a014eb54d39e875e"

    const result_name = yield* readFaName(real_token_address).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("Result Name:", result_name)

    const result_decimals = yield* readFaDecimals(real_token_address).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("Result Decimals:", result_decimals)

    const result_symbol = yield* readFaSymbol(real_token_address).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("Result Symbol:", result_symbol)

    const result_tokenIfo = yield* readFaTokenInfo(real_token_address).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient })
    )
    yield* Effect.log("Result Token Info:", result_tokenIfo)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
