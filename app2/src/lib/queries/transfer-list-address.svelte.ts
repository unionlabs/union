import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferListAddress, transferCount } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferCount, TransferList } from "$lib/schema/transfer-list"
import type { SortOrder } from "$lib/schema/sort-order"
import type { AddressCanonicalBytes } from "$lib/schema/address"

export const LIMIT = 10

export const transferListLatestAddressQuery = (
  addresses: Array<typeof AddressCanonicalBytes.Type>,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListLatestAddress($addresses: [String!]!, $limit: Int!) @cached(ttl: 1) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        where: {
          _or: [
            { sender_normalized: { _in: $addresses } },
            { receiver_normalized: { _in: $addresses } }
          ]
        },
        order_by: { sort_order: desc_nulls_last }) {
        ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { addresses, limit },
    refetchInterval: "1 second",
    writeData: data => {
      transferListAddress.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferListAddress.error = error
    }
  })

export const transferListPageLtAddressQuery = (
  page: typeof SortOrder.Type,
  addresses: Array<typeof AddressCanonicalBytes.Type>,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPageLtAddress($page: String!, $addresses: [String!]!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        where: {
          _and: [
            {sort_order: {_lt: $page}},
            {
              _or: [
                { sender_normalized: { _in: $addresses } },
                { receiver_normalized: { _in: $addresses } }
              ]
            }
          ]
        },
        order_by: {sort_order: desc_nulls_last}
      ) {
        ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, addresses, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferListAddress.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferListAddress.error = error
    }
  })

export const transferCountForAddressesQuery = (
  addresses: Array<typeof AddressCanonicalBytes.Type>
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders_aggregate: TransferCount }),
    document: graphql(
      `
    query TransferCountForAddresses($addresses: [String!]!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders_aggregate(
        where: {
          _or: [
            { sender_normalized: { _in: $addresses } },
            { receiver_normalized: { _in: $addresses } }
          ]
        }
      ) {
        aggregate {
          count
        }
      }
    }
  `
    ),
    variables: { addresses },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferCount.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders_aggregate))
    },
    writeError: error => {
      transferCount.error = error
    }
  })

export const transferListPageGtAddressQuery = (
  page: typeof SortOrder.Type,
  addresses: Array<typeof AddressCanonicalBytes.Type>,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPageGtAddress($page: String!, $addresses: [String!]!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        where: {
          _and: [
            {sort_order: {_gt: $page}},
            {
              _or: [
                { sender_normalized: { _in: $addresses } },
                { receiver_normalized: { _in: $addresses } }
              ]
            }
          ]
        },
        order_by: {sort_order: asc_nulls_last}
      ) {
        ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, addresses, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferListAddress.data = data.pipe(
        Option.map(d => d.v1_ibc_union_fungible_asset_orders.toReversed())
      )
    },
    writeError: error => {
      transferListAddress.error = error
    }
  })
