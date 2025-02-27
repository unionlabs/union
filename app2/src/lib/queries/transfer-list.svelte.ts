import { createQueryGraphql } from "$lib/utils/queries"

export const LIMIT = 10
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"
import type { SortOrder } from "$lib/schema/sort-order"

export let transferListLatestQuery = (limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListLatest($limit: Int!) @cached(ttl: 1) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        order_by: { sort_order: desc_nulls_last}) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { limit },
    refetchInterval: "1 second",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferList.error = error
    }
  })

export let transferListPageLtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        where: {sort_order: {_lt: $page}},
        order_by: {sort_order: desc_nulls_last}
      ) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferList.error = error
    }
  })

export let transferListPageGtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: $limit,
        distinct_on: sort_order,
        where: {sort_order: {_gt: $page}},
        order_by: {sort_order: asc_nulls_last}
      ) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(
        Option.map(d => d.v1_ibc_union_fungible_asset_orders.toReversed())
      )
    },
    writeError: error => {
      transferList.error = error
    }
  })
