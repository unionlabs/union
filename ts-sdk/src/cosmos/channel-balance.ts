import { Effect } from "effect"
import { CosmosChannelDestination } from "./channel.js"
import { CosmWasmClientDestination } from "./client.js"
import { queryContract } from "./contract.js"

export const channelBalance = (path: bigint, token: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientDestination).client
    const config = yield* CosmosChannelDestination

    const result = yield* queryContract(client, config.ucs03address, {
      get_channel_balance: {
        channel_id: config.channelId,
        path: path,
        denom: token,
      },
    })
    return result
  })
