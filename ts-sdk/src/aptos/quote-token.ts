import { Effect } from "effect"
import type { Hex } from "viem"
import { ucs03abi } from "./abi/ucs03.js"
import { readContract } from "./contract.js"
import { AptosPublicClientDestination } from "./client.js"
import { AptosChannelDestination } from "./channel.js"

export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClientDestination).client
    const config = yield* AptosChannelDestination

    const result = yield* readContract(client, {
      address: config.ucs03address,
      abi: ucs03abi,
      functionName: "predictWrappedToken",
      args: [0n, config.channelId, baseToken]
    })

    // Extract the address from the result tuple
    return result[0]
  })
