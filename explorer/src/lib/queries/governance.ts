import { CosmosClient } from "$lib/services/cosmos-client"
import { Effect } from "effect"

export const fetchProposals = (status?: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getProposals(status)
  })

export const fetchProposal = (id: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getProposal(id)
  })

export const fetchProposalVotes = (id: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getProposalVotes(id)
  })

export const fetchProposalTally = (id: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getProposalTally(id)
  })
