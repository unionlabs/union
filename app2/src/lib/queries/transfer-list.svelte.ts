import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"
import type { SortOrder } from "$lib/schema/sort-order"

export let transferListLatestQuery = createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
  document: graphql(
    `
    query TransferListLatest {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        distinct_on: sort_order
        order_by: { sort_order: desc_nulls_last}) {
      ...TransferListItem
      }
    }
  `,
    [transferListItemFragment]
  ),
  variables: {},
  refetchInterval: "200 millis",
  writeData: data => {
    transferList.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
  },
  writeError: error => {
    transferList.error = error
  }
})

export let transferListPageLtQuery = (page: typeof SortOrder.Type) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        distinct_on: sort_order
        where: {sort_order: {_lt: $page}},
        order_by: {sort_order: desc_nulls_last}
      ) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
    },
    writeError: error => {
      transferList.error = error
    }
  })

export let transferListPageGtQuery = (page: typeof SortOrder.Type) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        distinct_on: sort_order
        where: {sort_order: {_gt: $page}},
        order_by: {sort_order: asc_nulls_last}
      ) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page },
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
