import { Effect } from "effect"
import { CosmWasmClientContext, createCosmWasmClient } from "../src/cosmos/client"
import { readCw20TokenInfoAndBalance } from "../src/cosmos/cw20"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a CosmWasm client
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")

    // Read CW20 token info and balance
    const tokenData = yield* readCw20TokenInfoAndBalance(
      "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr", // WETH on stargaze
      "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4" // The address to check balance for
    ).pipe(Effect.provideService(CosmWasmClientContext, { client }))

    return tokenData
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
