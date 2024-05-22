import {
  Client,
  type Exchange,
  cacheExchange,
  fetchExchange,
  debugExchange,
  subscriptionExchange
} from "@urql/svelte"
import { URLS } from "$lib/constants"
import { devtoolsExchange } from "@urql/devtools"
import { retryExchange } from "@urql/exchange-retry"
import { refocusExchange } from "@urql/exchange-refocus"
import type { TadaPersistedDocumentNode } from "gql.tada"
import { persistedExchange } from "@urql/exchange-persisted"
import { requestPolicyExchange } from "@urql/exchange-request-policy"
import { createClient as createWSClient, type SubscribePayload } from "graphql-ws"

const isDevelopment = import.meta.env.DEV

/**
 * @see https://commerce.nearform.com/open-source/urql/docs/
 */

const headers = {
  "X-Hasura-Role": "app"
} satisfies HeadersInit

const wsClient = createWSClient({
  url: URLS.GRAPHQL_WSS,
  shouldRetry: () => true,
  connectionParams: async () => ({ headers })
})

const productionOnlyExchanges = (
  isDevelopment
    ? []
    : [
        cacheExchange,
        persistedExchange({
          generateHash: async (_, document) => (document as TadaPersistedDocumentNode).documentId,
          preferGetForPersistedQueries: true,
          enforcePersistedQueries: true,
          enableForMutation: true,
          enableForSubscriptions: true
        })
      ]
) satisfies Array<Exchange>

export const graphqlClient = new Client({
  url: URLS.GRAPHQL_WSS,
  // don't cache at all in development
  requestPolicy: import.meta.env.DEV ? "network-only" : "cache-and-network",
  exchanges: [
    devtoolsExchange,
    refocusExchange(),
    requestPolicyExchange({
      ttl: 60 * 1_000, // 1 minute
      shouldUpgrade: operation => operation.context.requestPolicy !== "cache-first"
    }),
    cacheExchange,
    persistedExchange({
      enableForMutation: true,
      enableForSubscriptions: true,
      enforcePersistedQueries: true,
      preferGetForPersistedQueries: true,
      generateHash: async (_, document) => (document as TadaPersistedDocumentNode).documentId
    }),
    fetchExchange,
    subscriptionExchange({
      forwardSubscription: operation => ({
        subscribe: sink => ({
          unsubscribe: wsClient.subscribe(operation as SubscribePayload, sink)
        })
      })
    }),
    retryExchange({
      randomDelay: true,
      maxDelayMs: 15_000,
      maxNumberAttempts: 2,
      initialDelayMs: 1_000,
      retryIf: error => !!error?.networkError
    }),
    debugExchange
  ],
  fetchOptions: () => ({ headers })
})
