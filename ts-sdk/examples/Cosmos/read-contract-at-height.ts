/**
 * @title Read Contract at Height
 * @description Example read contract call for token balance.
 * @badge WIP:caution
 */
/// <reference types="effect" />
/// <reference types="@cosmjs/stargate" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
import { Cosmos } from "@unionlabs/sdk"
import { Effect } from "effect"

const contractAddr = "bbn1zsrv23akkgxdnwul72sftgv2xjt5khsnt3wwjhp0ffh683hzp5aq5a0h6n"
const minter = "bbn1sakazthycqgzer50nqgr5ta4vy3gwz8wxla3s8rd8pql4ctmz5qssg39sf"
const ucs03Addr = "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h"

const client = Cosmos.Client.Live("https://rpc.bbn-test-5.babylon.chain.kitchen")
const destinationClient = Cosmos.Client.Live("https://rpc.bbn-test-5.babylon.chain.kitchen")
const destinationChannel = Cosmos.ChannelDestination.Live({
  ucs03address: ucs03Addr,
  channelId: 3,
})

// Example CW20 token balance query
const babylon = Effect.gen(function*() {
  // Query a CW20 token contract for a balance
  const balanceNow = yield* Cosmos.channelBalance(0n, "ubbn")
  yield* Effect.log(`Balance now: ${balanceNow}`)

  const rest = "https://rest.bbn-test-5.babylon.chain.kitchen"

  const balanceAtBlock123 = yield* Cosmos.channelBalanceAtHeight(rest, 0n, "ubbn", 912421)

  console.info("Balance at 233799:", balanceAtBlock123)

  const cw20balance = yield* Cosmos.readCw20Balance(contractAddr, minter)

  console.info("cw20balance", cw20balance)

  const cw20BalanceAtHeight = yield* Cosmos.readCw20BalanceAtHeight(
    rest,
    contractAddr,
    minter,
    912421,
  )
  console.info("cw20BalanceAtHeight", cw20BalanceAtHeight)

  const cw20TotalSupply = yield* Cosmos.readCw20TotalSupply(contractAddr)

  console.info("cw20TotalSupply", cw20TotalSupply)

  const cw20TotalSupplyAtHeight = yield* Cosmos.readCw20TotalSupplyAtHeight(
    rest,
    contractAddr,
    912421,
  )

  console.info("cw20TotalSupplyAtHeight", cw20TotalSupplyAtHeight)

  return balanceAtBlock123
}).pipe(
  Effect.provide(client),
  Effect.provide(destinationChannel),
  Effect.provide(destinationClient),
)

Effect.runPromise(babylon)
  .then(console.log)
  .catch(console.error)

/*

Effect.runPromiseExit(
  Effect.gen(function*() {
    // Create a CosmWasm client
    const rpc = "https://rpc.xion-testnet-2.xion.chain.cooking"
    const client = yield* createCosmWasmClient(rpc)
    const tokenDenom = "xion100jj57u4rna4wcdnn8pxvnacxvm0fx6zaazj5xqq555syvvae2wsqsum0y"
    const latest = yield* getChainHeight(client)

    console.info("height: ", latest)

    const totalSupplyNow = yield* readCw20TotalSupply(tokenDenom).pipe(
      Effect.provideService(CosmWasmClientContext, { client }),
      Effect.tapError(e => Effect.logError("Error fetching channel balance:", e)),
    )
    console.info("totalSupplyNow", totalSupplyNow)

    const rest = "https://rest.xion-testnet-2.xion.chain.cooking"

    const totalSupplyAtHeight = yield* readCw20TotalSupplyAtHeight(rest, tokenDenom, Number(latest))
      .pipe(
        Effect.tapError(e => Effect.logError("height-query failed:", e)),
      )

    console.info("totalSupplyAtHeight:", totalSupplyAtHeight)

    const minter = "xion1egp7k30mskfxmhy2awk677tnqdl6lfkfxhrwsv"
    const { amount } = yield* getBalanceNow(client, minter, "uxion")

    console.info("client.getBalance: ", amount)

    const amountAtHeight2 = yield* getBalanceAtHeight(rest, minter, "uxion", 3250475).pipe(
      Effect.provide(FetchHttpClient.layer),
      Effect.tapErrorCause((cause) =>
        Effect.logError("Error fetching channel balance at height:", cause)
      ),
    )
    console.info("Balance at height2: ", amountAtHeight2)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
*/
