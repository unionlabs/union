import { Context, Effect } from "effect"
import type { Address, Hex } from "viem"
import { ucs03abi } from "./abi/ucs03.js"
import { readContract } from "./contract.js"
import { ViemPublicClientDestination } from "./client.js"

export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function* () {
    const client = (yield* ViemPublicClientDestination).client
    const config = yield* DestinationConfig

    const result = yield* readContract(client, {
      address: config.ucs03address,
      abi: ucs03abi,
      functionName: "predictWrappedToken",
      args: [0n, config.channelId, baseToken]
    })

    // Extract the address from the result tuple
    return result[0]
  })

export class DestinationConfig extends Context.Tag("DestinationConfig")<
  DestinationConfig,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}

export class SourceConfig extends Context.Tag("SourceConfig")<
  SourceConfig,
  { readonly ucs03address: Address; readonly channelId: number }
>() {}
