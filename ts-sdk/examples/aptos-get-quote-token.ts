import { Effect } from "effect"
import { AptosPublicClientDestination, createAptosPublicClient } from "../src/aptos/client.js"
import { AptosChannelDestination } from "../src/aptos/channel.js"
import { predictQuoteToken } from "../src/aptos/quote-token.js"
import { AptosConfig, Network } from "@aptos-labs/ts-sdk"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create account from private key

    const rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"

    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    const publicClient = yield* createAptosPublicClient(config)

    const token_address = "0x6d756e6f"

    const result2 = yield* predictQuoteToken(token_address).pipe(
      Effect.provideService(AptosPublicClientDestination, { client: publicClient }),
      Effect.provideService(AptosChannelDestination, {
        ucs03address: "0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84",
        channelId: 2
      }),
      Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause))
    )

    yield* Effect.log("Result:", result2)
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
