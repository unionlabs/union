import { Effect } from "effect"
import type { Hex } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import type { Channel } from "$lib/schema/channel.ts"
import { getQuoteToken } from "./quote-token.ts"
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

    const wethAddress = yield* Effect.tryPromise({
      try: () =>
        publicClient.readContract({
          address: ucs03Address,
          abi: ucs03ZkgmAbi,
          functionName: "weth",
          args: []
        }) as Promise<Hex>,
      catch: error => {
        console.error("Failed to get WETH address:", error)
        return new Error("Failed to get WETH address from zkgm contract", { cause: error })
      }
    })

    return yield* getQuoteToken(sourceChain, wethAddress, channel, destinationChain).pipe(
      Effect.map(result => ({ wethQuoteToken: result.quote_token })),
      Effect.catchAll(error => {
        console.log("Error getting WETH quote token:", error)
        return Effect.succeed({ type: "NO_WETH_QUOTE" as const })
      })
    )
  })
