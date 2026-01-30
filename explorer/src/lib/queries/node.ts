import { Effect } from "effect"
import { CosmosClient } from "$lib/services/cosmos-client"

export const fetchNodeInfo = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getNodeInfo()
  })

export const fetchTotalSupply = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getTotalSupply()
  })
