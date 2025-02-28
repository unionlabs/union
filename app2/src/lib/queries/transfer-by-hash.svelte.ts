import { Option, Schema } from "effect"
import { createQueryGraphql } from "$lib/utils/queries"
import { TransferDetails } from "$lib/schema/transfer-details"
import { transferDetails } from "$lib/stores/transfer-details.svelte"
import { graphql } from "gql.tada"

export const transferByPacketHashQuery = (packetHash: string) =>
  createQueryGraphql({
    schema: Schema.Struct({
      v1_ibc_union_fungible_asset_orders: Schema.Array(TransferDetails)
    }),
    document: graphql(`
      query TransferByPacketHash($packet_hash: String!) {
        v1_ibc_union_fungible_asset_orders(where: {packet_hash: {_eq: $packet_hash}}) {
          sender_normalized
          source_chain_id
          source_connection_id
          source_channel_id
          packet_send_transaction_hash
          receiver_normalized
          destination_chain_id
          destination_connection_id
          destination_channel_id
          packet_send_timestamp
          packet_recv_timestamp
          base_token
          base_amount
          quote_amount
          quote_token
          traces {
            type
            height
            block_hash
            timestamp
            transaction_hash
            chain {
              chain_id
            }
          }
        }
      }
    `),
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
