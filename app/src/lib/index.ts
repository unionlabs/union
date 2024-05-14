export const BASE_URL = "https://app.union.build"

import { graphql } from "gql.tada"

const query = graphql(/* GraphQL */ `
query {
        v0_wasm_ibc_transfers(limit: 100, where: {
        _or: [{sender: {_eq: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"}}, {receiver: {_eq: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"}}]
      }) {
        sender
        receiver
        amount
        denom
        transaction_hash
        _contract_address
      }
}
`)

const response = await fetch("https://graphql.union.build/v1/graphql", {
  method: "POST",
  headers: {
    "Content-Type": "application-json",
    "X-Hasura-Role": "app"
  },
  body: JSON.stringify({
    query: `
  query {
        v0_wasm_ibc_transfers(limit: 100, where: {
        _or: [{sender: {_eq: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"}}, {receiver: {_eq: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"}}]
      }) {
        sender
        receiver
        amount
        denom
        transaction_hash
        _contract_address
      }
}
  `
  })
})

const data = await response.json()
console.log(JSON.stringify(data, undefined, 2))
