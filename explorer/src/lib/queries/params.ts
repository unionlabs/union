import { Effect } from "effect"
import { CosmosClient } from "$lib/services/cosmos-client"

export const fetchStakingParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getStakingParams()
  })

export const fetchSlashingParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getSlashingParams()
  })

export const fetchDistributionParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getDistributionParams()
  })

export const fetchGovParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getGovParams()
  })

export const fetchMintParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient
    return yield* client.getMintParams()
  })

// Fetch all params at once
export const fetchAllParams = () =>
  Effect.gen(function* () {
    const client = yield* CosmosClient

    // Fetch all in parallel
    const [staking, slashing, distribution, gov, mint] = yield* Effect.all([
      Effect.either(client.getStakingParams()),
      Effect.either(client.getSlashingParams()),
      Effect.either(client.getDistributionParams()),
      Effect.either(client.getGovParams()),
      Effect.either(client.getMintParams()),
    ])

    return {
      staking: staking._tag === "Right" ? staking.right.params : null,
      slashing: slashing._tag === "Right" ? slashing.right.params : null,
      distribution: distribution._tag === "Right" ? distribution.right.params : null,
      gov: gov._tag === "Right" ? gov.right.params : null,
      mint: mint._tag === "Right" ? mint.right.params : null,
    }
  })
