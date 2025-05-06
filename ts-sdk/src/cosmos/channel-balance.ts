import { Effect } from "effect"
import { extractErrorDetails } from "../utils/extract-error-details.js"
import { CosmosChannelDestination } from "./channel.js"
import { CosmWasmClientDestination } from "./client.js"
import { ExtendedCosmWasmClientContext } from "./client.js"
import { queryContract } from "./contract.js"
import { QueryContractError } from "./contract.js"

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

export const channelBalanceAtHeight = (path: bigint, token: string, height: number) =>
  Effect.gen(function*() {
    const client = (yield* ExtendedCosmWasmClientContext).client
    const config = yield* CosmosChannelDestination
    const resp = yield* Effect.tryPromise({
      try: () =>
        client.queryContractSmartAtHeight(
          config.ucs03address,
          {
            get_channel_balance: {
              channel_id: config.channelId,
              path,
              denom: token,
            },
          },
          height,
        ),
      catch: error => new QueryContractError({ cause: extractErrorDetails(error as Error) }),
    }).pipe(
      Effect.timeout("10 seconds"),
      Effect.retry({ times: 5 }),
      Effect.catchAllCause(err =>
        Effect.sync(() => {
          console.error("Error in channelBalanceAtHeight:", err)
        })
      ),
    )
    return resp.data
  })
