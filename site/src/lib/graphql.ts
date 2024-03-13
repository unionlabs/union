import {
  gql,
  Client,
  queryStore,
  cacheExchange,
  fetchExchange,
  getContextClient,
  type AnyVariables,
  type DocumentInput
} from "@urql/svelte"

export const client = new Client({
  url: "https://graphql.union.build/v1/graphql",
  exchanges: [cacheExchange, fetchExchange]
})

export function getQueryStore<Data = any>(query: DocumentInput<Data, AnyVariables> | string) {
  return queryStore({
    client: getContextClient(),
    query: typeof query === "string" ? gql(query) : query
  })
}
