import * as React from "react"
import { dedent } from "ts-dedent"
import { stringIsJSON } from "#/lib/utilities.ts"
import { GraphiQL, type GraphiQLProps } from "graphiql"
import { createGraphiQLFetcher } from "@graphiql/toolkit"
import { explorerPlugin } from "@graphiql/plugin-explorer"

const GRAPHQL_ENDPOINT =
  import.meta.env.PUBLIC_GRAPHQL_URL ?? "https://graphql.union.build/v1/graphql"

type LocalStorageParsedQuery = {
  query: string
  headers: string
  variables: string
  operationName: string
}

const fetcher = createGraphiQLFetcher({
  url: GRAPHQL_ENDPOINT,
  enableIncrementalDelivery: true
})

let query = dedent(/* GraphQL */ `
  query UserTransfers {
    v1_transfers(
      limit: 3,
      where: {
        sender: { 
          _eq: "union17ttpfu2xsmfxu6shl756mmxyqu33l5ljs5j6md"
        }
      }
    ) {
      sender
      receiver
      source_transaction_hash
      destination_transaction_hash
    }
  }
`)

const graphiqlProps = {
  fetcher,
  plugins: [
    explorerPlugin({
      hideActions: true,
      title: "union.build",
      explorerIsOpen: true,
      showAttribution: false
    })
  ],
  disableTabs: true,
  onEditVariables: variables => {
    localStorage.setItem("graphiql:variables", variables)
  },
  isHeadersEditorEnabled: true,
  defaultEditorToolsVisibility: stringIsJSON(localStorage.getItem("graphiql:variables") ?? ""), 
  toolbar: {
    additionalContent: [
      React.createElement(
        "button",
        {
          key: "share",
          type: "button",
          className: "graphiql-un-styled graphiql-toolbar-button",
          title: "Copy sharable URL to clipboard",
          "aria-label": "Copy sharable URL to clipboard",
          onClick: (event: React.SyntheticEvent) => {
            event.preventDefault()

            const query = localStorage.getItem("graphiql:query")
            if (!query) return

            const encoded = encodeURIComponent(query)
            let urlPath = `?query=${encoded}`

            const variables = localStorage.getItem("graphiql:variables")
            if (variables) urlPath += `&variables=${encodeURIComponent(variables)}`

            window.history.pushState({}, "", urlPath)
            // copy url to clipboard
            navigator.clipboard.writeText(window.location.href)
          }
        },
        React.createElement("img", {
          alt: "share",
          className: "graphiql-toolbar-button-image size-6 hover:[#e5e7eb]",
          src: "https://api.iconify.design/ic:round-share.svg?color=%236b778a"
        })
      )
    ]
  },
  defaultQuery: localStorage.getItem("graphiql:query") || query
} satisfies GraphiQLProps

export const graphiqlElement = React.createElement(
  GraphiQL,
  graphiqlProps,
  React.createElement(GraphiQL.Logo, {}, React.createElement("span", {}, "Union GraphiQL"))
)
