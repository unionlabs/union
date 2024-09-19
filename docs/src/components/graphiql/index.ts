import * as React from "react"
import { dedent } from "ts-dedent"
import { GraphiQL, type GraphiQLProps } from "graphiql"
import { createGraphiQLFetcher } from "@graphiql/toolkit"
import { explorerPlugin } from "@graphiql/plugin-explorer"

const GRAPHQL_ENDPOINT = "https://graphql.union.build/v1/graphql"
const fetcher = createGraphiQLFetcher({
  url: GRAPHQL_ENDPOINT,
  enableIncrementalDelivery: true
})

const graphiqlProps = {
  fetcher,
  plugins: [
    explorerPlugin({
      hideActions: true,
      title: "union.build",
      explorerIsOpen: true,
      showAttribution: false,
      styles: {
        buttonStyle: { color: "white", backgroundColor: "transparent" }
      }
    })
  ],
  disableTabs: true,
  isHeadersEditorEnabled: false,
  defaultEditorToolsVisibility: false,
  query: dedent(/* GraphQL */ `
      query UserTransfers {
        v0_transfers(where: { sender: { _eq: "union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md" } }) {
          source_transaction_hash
          destination_transaction_hash
        }
      }
    `),
  defaultQuery: dedent(/* GraphQL */ `
      query UserTransfers {
        v0_transfers(where: { sender: { _eq: "union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md" } }) {
          source_transaction_hash
          destination_transaction_hash
        }
      }
    `)
} satisfies GraphiQLProps

export const graphiqlElement = React.createElement(GraphiQL, graphiqlProps)
