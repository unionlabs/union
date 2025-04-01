import { Effect } from "effect"
import { CosmWasmClientContext, createCosmWasmClient } from "../src/cosmos/client.ts"
import { readCw20TokenInfo, readCw20Balance, readCw20Allowance } from "../src/cosmos/cw20.ts"

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a CosmWasm client
    const client = yield* createCosmWasmClient("https://rpc.elgafar-1.stargaze-apis.com")

    // Contract address and wallet address
    const contractAddress = "stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr" // WETH on stargaze
    const walletAddress = "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4" // The address to check balance for

    // Provide the client to the Effect context
    const withClient = Effect.provideService(CosmWasmClientContext, { client })

    // Read CW20 token info
    const tokenInfo = yield* readCw20TokenInfo(contractAddress).pipe(withClient)

    // Read CW20 token balance
    const balance = yield* readCw20Balance(contractAddress, walletAddress).pipe(withClient)

    const spender = "stars14qemq0vw6y3gc3u3e0aty2e764u4gs5lddqqxv"
    // Read CW20 token allowance
    const allowance = yield* readCw20Allowance(contractAddress, walletAddress, spender).pipe(
      withClient
    )

    // Combine the results
    return {
      ...tokenInfo,
      balance,
      allowance
    }
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
