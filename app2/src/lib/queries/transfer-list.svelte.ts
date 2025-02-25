import { createQueryGraphql } from "$lib/utils/queries"
import { ParseResult, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "$lib/schema/transfer-list"

const ResponseSchema = Schema.Struct({ v1_ibc_union_fungible_asset_orders: TransferList })

const TransferListFromResponse = Schema.transformOrFail(ResponseSchema, TransferList, {
  strict: true,
  decode: input => ParseResult.succeed(input.v1_ibc_union_fungible_asset_orders),
  encode: (x, _, ast) => ParseResult.fail(new ParseResult.Forbidden(ast, x, "I will never encode"))
})

export let transferListQuery = createQueryGraphql({
  schema: TransferListFromResponse,
  document: graphql(
    `
    query TransferList {
      v1_ibc_union_fungible_asset_orders(
        limit: 20,
        order_by: { packet_send_timestamp: desc_nulls_last}) {
      ...TransferListItem
      }
    }
  `,
    [transferListItemFragment]
  ),
  refetchInterval: "5 seconds",
  writeData: data => {
    transferList.data = data
  },
  writeError: error => {
    transferList.error = error
  }
})
