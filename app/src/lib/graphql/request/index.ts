import { URLS } from "$lib/constants/index.ts"
import { transfersTimestampFilterQueryDocument } from "$lib/graphql/documents/transfers"
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

graphqlRequest(transfersTimestampFilterQueryDocument, {
  timestamp: "2024-07-05T13:49:24+00:00"
}).then(_ => {
  console.info(_.bottom)
})
