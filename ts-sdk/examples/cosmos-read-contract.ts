import { Effect } from "effect"
import { CosmWasmClientContext, createCosmWasmClient } from "../src/cosmos/client"
import { queryContract } from "../src/cosmos/contract"

// Example CW20 token balance query
Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a CosmWasm client
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")

    // Query a CW20 token contract for a balance
    const balance = yield* queryContract<{ balance: string }>(
      client,
      "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr", // Example CW20 contract address
      {
        balance: {
          address: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4" // The address to check balance for
        }
      }
    )

    return balance
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
