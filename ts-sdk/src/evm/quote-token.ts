import { Effect } from "effect"
import type { Address, Hex } from "viem"
import { ucs03abi } from "./abi/ucs03.js"
import { readContract } from "./contract.js"
import { PublicDestinationViemClient } from "./client.js"

export const quoteToken = (baseToken: Hex, ucs03address: Address, destinationChannelId: number) =>
  Effect.gen(function* () {
    const client = (yield* PublicDestinationViemClient).client
    
    const result = yield* readContract(
      client,
      {
        address: ucs03address,
        abi: ucs03abi,
        functionName: "predictWrappedToken",
        args: [0n, destinationChannelId, baseToken]
      }
    )

    // Extract the address from the result tuple
    return result[0]
  })
