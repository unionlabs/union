import { CosmosClient } from "$lib/services/cosmos-client"
import { Effect } from "effect"

export const fetchNodeInfo = () =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getNodeInfo()
  })

export const fetchTotalSupply = () =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getTotalSupply()
  })
