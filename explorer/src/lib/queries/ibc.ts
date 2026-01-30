import { Effect } from "effect"
import { CosmosClient } from "$lib/services/cosmos-client"

export const fetchIBCChannels = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getIBCChannels()
  })

export const fetchIBCConnections = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getIBCConnections()
  })

export const fetchIBCClientStates = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getIBCClientStates()
  })
