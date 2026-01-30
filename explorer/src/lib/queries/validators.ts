import { CosmosClient } from "$lib/services/cosmos-client"
import { Effect } from "effect"

export const fetchValidators = (status?: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getValidators(status)
  })

export const fetchValidator = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getValidator(address)
  })

export const fetchValidatorDelegations = (address: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getValidatorDelegations(address)
  })

export const fetchStakingPool = () =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getStakingPool()
  })

export const fetchStakingParams = () =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getStakingParams()
  })
