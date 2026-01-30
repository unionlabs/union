// GraphQL client for Union's indexer API

const GRAPHQL_ENDPOINT = "https://graphql.union.build/v1/graphql"

export async function graphqlFetch<T>(query: string, variables?: Record<string, unknown>): Promise<T> {
  const response = await fetch(GRAPHQL_ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query, variables }),
  })

  const json = await response.json()

  if (json.errors) {
    throw new Error(json.errors[0]?.message ?? "GraphQL error")
  }

  return json.data as T
}
