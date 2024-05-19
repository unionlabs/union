import { graphql } from "gql.tada"
import { URLS } from "$lib/constants"
import { request } from "graphql-request"
import { fetcher } from "$lib/utilities/index.ts"
import { createQueries, createQuery } from "@tanstack/svelte-query"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"

/**
 * TODO:
 * - [x] Add IBC transfers query
 * - [x] Add Union transfers query
 * - [❗] Add Sepolia transfers query: no need
 */

export function ibcTransfersQuery({ address, limit = 100 }: { address: string; limit?: number }) {
  const query = graphql(/* GraphQL */ `
    query userTransfers($address: String!, $limit: Int!) {
      v0_cosmos_wasm_ibc_transfer(
        order_by: { height: desc },
        limit: $limit,
        where: {
        _or: [{ sender: { _eq: $address } }, { receiver: { _eq: $address }}]
      }) {
        height
        time
        json
        memo
        chain_id
        transaction_hash
      }
    }
  `)
  return createQuery({
    queryKey: ["ibc-transfers", address],
    queryFn: async () => request(URLS.GRAPHQL, query, { address, limit }),
    enabled: !!address
  })
}

export interface CrossChainTransfer {}

export function unionTransfersQuery({
  address,
  include = ["SENT", "RECEIVED"],
  refetchInterval = 10_000,
  enabled = true
}: {
  address: string
  include?: Array<"SENT" | "RECEIVED">
  refetchInterval?: number
  enabled?: boolean
}) {
  const query = graphql(/* GraphQL */ `
query CosmosSDKUnionTransfers($address: String!, $limit: Int!) {
v0_cosmos_transfer(limit:  $limit, where: {_or: [{sender: {_eq: $address}}, {recipient: {_eq: $address}}]}) {
  sender
  recipient
  amount
  denom
  height
  chain_id
  transaction_hash
}
}
  `)
  const baseUrl = `${URLS.UNION.REST}/cosmos/tx/v1beta1/txs`
  if (!isValidCosmosAddress(address)) return null
  return createQueries({
    queries: [
      {
        queryKey: ["union-transfers-sent", address],
        queryFn: async () =>
          await fetcher<UnionTransfersQuery>(`${baseUrl}?query=transfer.sender='${address}'`),
        refetchInterval,
        // TODO: add Union address validation
        enabled: include.includes("SENT") && isValidCosmosAddress(address) && enabled
      },
      {
        queryKey: ["union-transfers-received", address],
        queryFn: async () =>
          await fetcher<UnionTransfersQuery>(`${baseUrl}?query=transfer.recipient='${address}'`),
        refetchInterval,
        // TODO: add Union address validation
        enabled: include.includes("RECEIVED") && isValidCosmosAddress(address) && enabled
      }
    ],
    combine: resultArray => ({
      error: resultArray.map(result => result.error),
      status: resultArray.flatMap(result => result.status).at(-1),
      data: resultArray.flatMap(result => result.data?.tx_responses).filter(Boolean)
    })
  })
}

export interface UnionTransfersQuery {
  total: string
  pagination: any
  tx_responses: Array<UnionTransfer>
}

export interface UnionTransfer {
  code: number
  data: string
  info: string
  height: string
  txhash: string
  raw_log: string
  logs: Array<any>
  gas_used: string
  codespace: string
  timestamp: string
  gas_wanted: string
  tx: {
    body: {
      messages: Array<{
        "@type": string
        sender: string
        contract: string
        msg: {
          transfer: {
            channel: string
            receiver: string
            timeout: any
            memo: string
          }
        }
        funds: Array<{ denom: string; amount: string }>
      }>
      memo: string
      timeout_height: string
      extension_options: Array<any>
      non_critical_extension_options: Array<any>
    }
    "@type": string
    signatures: Array<string>
    auth_info: {
      signer_infos: Array<{
        sequence: string
        mode_info: { single: { mode: string } }
        public_key: { "@type": string; key: string }
      }>
      fee: {
        payer: string
        granter: string
        gas_limit: string
        amount: Array<{ denom: string; amount: string }>
      }
      tip: any
    }
  }
  events: Array<{
    type: string
    attributes: Array<{
      key: string
      value: string
      index: boolean
    }>
  }>
}
