import { Effect, Schedule } from "effect"
import type { Hex } from "viem"
import type { Channel } from "$lib/schema/channel.ts"
import { getPublicClient } from "$lib/services/evm/clients.ts"
import type { Chain } from "$lib/schema/chain.ts"

export const getWethQuoteToken = (
  sourceChain: Chain,
  ucs03Address: Hex,
  channel: Channel,
  destinationChain: Chain
) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(sourceChain)
    // const wethAddress = yield* Effect.tryPromise({
    //   try: () => new Promise(() => {Promise.resolve("0x000")}),

    //   // publicClient.readContract({
    //   //   address: ucs03Address,
    //   //   abi: ucs03ZkgmAbi,
    //   //   functionName: "weth",
    //   //   args: []
    //   // }) as Promise<Hex>,
    //   catch: error => {
    //     console.error("Failed to get WETH address:", error)
    //     return new GetWethQuoteError({
    //       cause: `Failed to get WETH address from zkgm contract: ${error}`
    //     })
    //   }
    // })
    // const wethAddress = "0x000"
    return { wethQuoteToken: "0x00" }

    // return yield* getQuoteToken(sourceChain, wethAddress, channel, destinationChain).pipe(
    //   Effect.map(result => ({ wethQuoteToken: result.quote_token })),
    //   Effect.mapError(
    //     error =>
    //       new GetWethQuoteError({
    //         cause: `Failed to get WETH quote token: ${error instanceof Error ? error.message : String(error)}`
    //       })
    //   )
    // )
  })
