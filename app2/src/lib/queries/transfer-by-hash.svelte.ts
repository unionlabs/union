import { Option, Schema } from "effect"
import { createQueryGraphql } from "$lib/utils/queries"
import { TransferListItem } from "$lib/schema/transfer-list"
import { transferDetails } from "$lib/stores/transfer-details.svelte"
import { graphql } from "gql.tada"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"

export const transferByPacketHashQuery = (packetHash: string) =>
  createQueryGraphql({
    schema: Schema.Struct({
      v1_ibc_union_fungible_asset_orders: Schema.Array(TransferListItem)
    }),
    document: graphql(
      `
      query TransferByPacketHash($packet_hash: String!) {
        v1_ibc_union_fungible_asset_orders(where: {packet_hash: {_eq: $packet_hash}}) {
          ...TransferListItem
        }
      }
      `,
      [transferListItemFragment]
    ),
    variables: { packet_hash: packetHash },
    refetchInterval: "1 second",
    writeData: data => {
      if (
        data
          .pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders.length))
          .pipe(Option.getOrElse(() => 0)) === 0
      ) {
        transferDetails.error = Option.some({ _tag: "NotFound", message: "Transfer not found" })
      }
      transferDetails.data = data.pipe(Option.map(d => d.v1_ibc_union_fungible_asset_orders[0]))
    },
    writeError: error => {
      transferDetails.error = error
    }
  })
