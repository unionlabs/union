import { initGraphQLTada } from "gql.tada"
import type { introspection } from "$generated/graphql-env"

export const graphql = initGraphQLTada<{
  introspection: introspection
  scalars: {
    jsonb: any
    timestamptz: string
  }
}>()

export type { FragmentOf, ResultOf, VariablesOf } from "gql.tada"
export { readFragment } from "gql.tada"
