import { Context, Effect } from "effect"
import { CosmWasmClientDestination } from "./client.js"
import { queryContract } from "./contract.js"
import type { Address } from "viem"

export const predictQuoteToken = (baseToken: string) =>
  Effect.gen(function* () {
    const client = (yield* CosmWasmClientDestination).client
    const config = yield* CosmosDestinationConfig

    const result = yield* queryContract<{ wrapped_token: Address }>(client, config.ucs03address, {
      predict_wrapped_token: {
        path: "0",
        channel: config.channelId,
        token: baseToken
      }
    })

    return result.wrapped_token
  })

export class CosmosDestinationConfig extends Context.Tag("CosmosDestinationConfig")<
  CosmosDestinationConfig,
  { readonly ucs03address: string; readonly channelId: number }
>() {}
