import { Effect } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { CosmosChannelDestination } from "./channel.js"
import { CosmWasmClientDestination } from "./client.js"
import { ExtendedCosmWasmClientContext } from "./client.js"
import { queryContract } from "./contract.js"
import { QueryContractError } from "./contract.js"
import { queryContractSmartAtHeight } from "./query.js"

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

export const channelBalanceAtHeight = (rest: string, path: bigint, token: string, height: number) =>
  Effect.gen(function*() {
    const config = yield* CosmosChannelDestination
    const resp = yield* queryContractSmartAtHeight(rest, config.ucs03address, {
      get_channel_balance: {
        channel_id: config.channelId,
        path,
        denom: token,
      },
    }, height)
    return resp
  })
