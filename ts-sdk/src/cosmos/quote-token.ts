import { Context, Effect } from "effect"
import type { Hex } from "viem"
import { CosmosChannelDestination } from "./channel.js"
import { CosmWasmClientDestination } from "./client.js"
import { queryContract } from "./contract.js"

export const predictQuoteToken = (baseToken: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientDestination).client
    const config = yield* CosmosChannelDestination

    console.log({ config, client })

    const result = yield* queryContract<{ wrapped_token: Hex }>(client, config.ucs03address, {
      predict_wrapped_token: {
        path: "0",
        channel_id: config.channelId,
        token: baseToken,
      },
    })

    return result.wrapped_token
  })

export class CosmosDestinationConfig extends Context.Tag("CosmosDestinationConfig")<
  CosmosDestinationConfig,
  { readonly ucs03address: string; readonly channelId: number }
>() {}
