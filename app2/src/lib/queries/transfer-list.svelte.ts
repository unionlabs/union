import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"

export let transferListLatestQuery = createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList }),
  document: graphql(
    `
    query TransferList @cached(ttl: 1) {
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
