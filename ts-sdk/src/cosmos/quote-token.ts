import { Context, Effect } from "effect"
import { CosmWasmClientContext } from "./client.js"
import { queryContract } from "./contract.js"

export const predictQuoteToken = (baseToken: string) =>
  Effect.gen(function* () {
    const client = (yield* CosmWasmClientContext).client
    const config = yield* CosmosDestinationConfig

    const result = yield* queryContract<{ token_address: string }>(
      client,
      config.ucs03address,
      {
        predict_wrapped_token: {
          channel_id: config.channelId,
          base_token: baseToken
        }
      }
    )

    return result.token_address
  })

export class CosmosDestinationConfig extends Context.Tag("CosmosDestinationConfig")<
  CosmosDestinationConfig,
  { readonly ucs03address: string; readonly channelId: number }
>() {}
