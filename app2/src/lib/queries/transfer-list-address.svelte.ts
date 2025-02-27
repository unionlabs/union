import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferListAddress } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"
import type { SortOrder } from "$lib/schema/sort-order"

export const LIMIT = 10

export const transferListLatestAddressQuery = (address: string, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListLatestAddress($address: String!, $limit: Int!) @cached(ttl: 1) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        where: {
          _or: [
            {sender_normalized: {_eq: $address}},
            {receiver_normalized: {_eq: $address}}
          ]
        },
        order_by: { sort_order: desc_nulls_last }) {
        ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { address, limit },
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
  address: string,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPageAddress($page: String!, $address: String!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        where: {
          _and: [
            {sort_order: {_lt: $page}},
            {
              _or: [
                {sender_normalized: {_eq: $address}},
                {receiver_normalized: {_eq: $address}}
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
    variables: { page, address, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferListAddress.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferListAddress.error = error
    }
  })

export const transferListPageGtAddressQuery = (
  page: typeof SortOrder.Type,
  address: string,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPageAddress($page: String!, $address: String!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        where: {
          _and: [
            {sort_order: {_gt: $page}},
            {
              _or: [
                {sender_normalized: {_eq: $address}},
                {receiver_normalized: {_eq: $address}}
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
    variables: { page, address, limit },
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
