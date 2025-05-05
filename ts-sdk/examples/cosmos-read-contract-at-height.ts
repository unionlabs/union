import { Effect } from "effect"
import {
  createCosmWasmClient,
  createExtendedCosmWasmClient,
  ExtendedCosmWasmClientContext,
  CosmWasmClientContext
} from "../src/cosmos/client.js"
import { channelBalance, channelBalanceAtHeight } from "../src/cosmos/channel-balance.js"
import { CosmWasmClientDestination } from "../src/cosmos/client.js"
import { CosmosChannelDestination } from "../src/cosmos/channel.js"
import {
  readCw20BalanceAtHeight,
  readCw20Balance,
  readCw20TotalSupply,
  readCw20TotalSupplyAtHeight
} from "../src/cosmos/cw20.js"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}
// Example CW20 token balance query
Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create a CosmWasm client
    const rpc = "https://rpc.bbn-test-5.babylon.chain.kitchen"
    const client = yield* createCosmWasmClient(rpc)
    const contractAddr = "bbn1zsrv23akkgxdnwul72sftgv2xjt5khsnt3wwjhp0ffh683hzp5aq5a0h6n"
    const minter = "bbn1sakazthycqgzer50nqgr5ta4vy3gwz8wxla3s8rd8pql4ctmz5qssg39sf"
    const ucs03Addr = "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h"
    // Query a CW20 token contract for a balance

    const balanceNow = yield* channelBalance(0n, "ubbn").pipe(
      Effect.provideService(CosmWasmClientDestination, { client }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: ucs03Addr,
        channelId: 3
      }),
      Effect.tapError(e => Effect.logError("Error fetching channel balance:", e))
    )
    console.info("balance", balanceNow)

    const rest = "https://rest.bbn-test-5.babylon.chain.kitchen"
    const extClient = yield* createExtendedCosmWasmClient(rpc, rest)

    const balanceAtBlock123 = yield* channelBalanceAtHeight(0n, "ubbn", 912421).pipe(
      Effect.provideService(ExtendedCosmWasmClientContext, { client: extClient }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: ucs03Addr,
        channelId: 3
      }),
      Effect.tapError(e => Effect.logError("height-query failed:", e))
    )

    console.info("Balance at 233799:", balanceAtBlock123)

    const withClient = Effect.provideService(CosmWasmClientContext, { client })

    const cw20balance = yield* readCw20Balance(contractAddr, minter).pipe(withClient)

    console.info("cw20balance", cw20balance)

    const withExtClient = Effect.provideService(ExtendedCosmWasmClientContext, {
      client: extClient
    })
    const cw20BalanceAtHeight = yield* readCw20BalanceAtHeight(contractAddr, minter, 912421).pipe(
      withExtClient
    )
    console.info("cw20BalanceAtHeight", cw20BalanceAtHeight)

    const cw20TotalSupply = yield* readCw20TotalSupply(contractAddr).pipe(withClient)

    console.info("cw20TotalSupply", cw20TotalSupply)

    const cw20TotalSupplyAtHeight = yield* readCw20TotalSupplyAtHeight(contractAddr, 912421).pipe(
      withExtClient
    )

    console.info("cw20TotalSupplyAtHeight", cw20TotalSupplyAtHeight)

    return balanceAtBlock123
  })
).then(exit => console.log(JSON.stringify(exit, null, 2)))
