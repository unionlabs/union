import { URLS } from "$lib/constants/index.ts"
import type {
  Variables,
  RequestDocument,
  TypedDocumentNode,
  VariablesAndRequestHeadersArgs
} from "./types.ts"

export async function graphqlRequest<T, V extends Variables = object>(
  document: RequestDocument | TypedDocumentNode<T, V>,
  ...variablesAndRequestHeaders: VariablesAndRequestHeadersArgs<V>
): Promise<T> {
  const response = await fetch(URLS.GRAPHQL, {
    method: "POST",
    headers: {
      "X-Hasura-Role": "anon",
      Accept: "application/json",
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      query: typeof document === "string" ? document : document.loc?.source.body,
      variables: variablesAndRequestHeaders.at(0)
    })
  })

  const data = (await response.json()) as T

  return data
}
