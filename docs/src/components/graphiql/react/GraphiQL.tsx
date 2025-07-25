import { explorerPlugin } from "@graphiql/plugin-explorer"
import { createGraphiQLFetcher } from "@graphiql/toolkit"
import { GraphiQL } from "graphiql"
import { useCallback, useEffect, useMemo, useState } from "react"

import "graphiql/style.css"
import "@graphiql/plugin-explorer/style.css"

import { stringIsJSON } from "#/lib/utilities.ts"
import { dedent } from "ts-dedent"

const GRAPHQL_ENDPOINT = import.meta.env.PUBLIC_GRAPHQL_URL
  ?? "https://graphql.union.build/v1/graphql"

const DEFAULT_QUERY = dedent(/* GraphQL */ `
  query GetLatest10UserTransfers @cached(ttl: 1) {
    v2_transfers(
      args: {
        p_limit: 10
        p_addresses_canonical: [
          "0x3c5daaa3c96ab8fe4cfc2fb6d76193fe959a9f82"
        ]
      }
    ) {
      sender_canonical
      receiver_canonical
      base_amount
      base_token_meta {
        # denom
        representations { name symbol decimals }
      }
      source_universal_chain_id
      destination_universal_chain_id
    }
  }
`)

export default function UnionGraphiQL() {
  const [query, setQuery] = useState<string>(
    localStorage.getItem("graphiql:query") ?? DEFAULT_QUERY,
  )
  const [variables, setVariables] = useState<string>(
    localStorage.getItem("graphiql:variables") ?? "",
  )

  useEffect(() => localStorage.setItem("graphiql:query", query), [query])
  useEffect(
    () => localStorage.setItem("graphiql:variables", variables),
    [variables],
  )

  const fetcher = useMemo(
    () =>
      createGraphiQLFetcher({
        url: GRAPHQL_ENDPOINT,
        enableIncrementalDelivery: true,
      }),
    [],
  )

  const explorer = explorerPlugin({
    query,
    onEdit: setQuery,
    showAttribution: false,
    hideActions: true,
  })

  const handleShare = useCallback(() => {
    const url = new URL(window.location.href)
    url.searchParams.set("query", query)
    if (variables) {
      url.searchParams.set("variables", variables)
    }
    history.replaceState({}, "", url)
    navigator.clipboard.writeText(url.href)
  }, [query, variables])

  return (
    <GraphiQL
      fetcher={fetcher}
      defaultQuery={query}
      initialVariables={variables}
      onEditQuery={setQuery}
      onEditVariables={setVariables}
      defaultEditorToolsVisibility={stringIsJSON(variables)}
      isHeadersEditorEnabled
      plugins={[explorer]}
    >
      <GraphiQL.Toolbar>
        {({ copy, merge, prettify }) => (
          <>
            {prettify}
            {merge}
            {copy}
            <button
              key="share"
              type="button"
              className="graphiql-un-styled graphiql-toolbar-button"
              title="Copy sharable URL to clipboard"
              aria-label="Copy sharable URL to clipboard"
              onClick={handleShare}
            >
              <img
                src="https://api.iconify.design/ic:round-share.svg?color=%236b778a"
                alt="share"
                className="graphiql-toolbar-button-image size-6 hover:[#e5e7eb]"
              />
            </button>
          </>
        )}
      </GraphiQL.Toolbar>
      <GraphiQL.Logo>
        <span>Union GraphiQL</span>
      </GraphiQL.Logo>
    </GraphiQL>
  )
}
