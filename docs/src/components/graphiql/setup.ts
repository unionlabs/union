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

let query = dedent(/* GraphQL */ `
      query UserTransfers {
        v0_transfers(
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
  onCopyQuery: query => {
    console.info(query)
  },
  isHeadersEditorEnabled: true,
  defaultEditorToolsVisibility: false,
  toolbar: {
    additionalContent: [
      React.createElement(
        "button",
        // biome-ignore lint/a11y/useButtonType: <explanation>
        {
          key: "share",
          type: "button",
          className: "graphiql-un-styled graphiql-toolbar-button",
          title: "Copy sharable URL to clipboard",
          "aria-label": "Copy sharable URL to clipboard",
          onClick: (event: React.SyntheticEvent) => {
            event.preventDefault()
            if (!localStorage?.getItem("graphiql:query")) return
            const query = localStorage.getItem("graphiql:query")
            if (!query) return
            const encoded = encodeURIComponent(query)
            console.info(encoded)
            console.info(window.btoa(encoded))
            window.history.pushState({}, "", `?query=${encoded}`)
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
