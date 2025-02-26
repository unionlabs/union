import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"
import { SortOrder } from "$lib/schema/sort-order"

export let transferListLatestQuery = createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
  document: graphql(
    `
    query TransferListLatest @cached(ttl: 1) {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        order_by: { sort_order: desc_nulls_last}) {
      ...TransferListItem
      }
    }
  `,
    [transferListItemFragment]
  ),
  variables: {},
  refetchInterval: "1 second",
  writeData: data => {
    transferList.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders))
  },
  writeError: error => {
    transferList.error = error
  }
})

export let transferListPageQuery = (page: typeof SortOrder.Type) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!) @cached(ttl: 30) {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        where: {sort_order: {_lt: $page}}
        order_by: { sort_order: desc_nulls_last}) {
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
