import { transferDetails } from "$lib/stores/transfer-details.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { TransferDetails } from "@unionlabs/sdk/schema"
import { Array as A, Option, pipe, Schema } from "effect"
import { NoSuchElementException } from "effect/Cause"
import { graphql } from "gql.tada"

export const transferByPacketHashQuery = (packetHash: string) =>
  createQueryGraphql({
    schema: Schema.Struct({
      v2_transfers: Schema.Array(TransferDetails),
    }),
    document: graphql(`
      query TransferByPacketHash($packet_hash: String!) {
        v2_transfers(args: {
          p_packet_hash: $packet_hash
        }) {
          sender_canonical
          sender_display
          source_chain {
            universal_chain_id
          }
          transfer_send_transaction_hash
          receiver_canonical
          receiver_display
          destination_chain {
            universal_chain_id
          }
          transfer_send_timestamp
          transfer_recv_timestamp
          transfer_timeout_transaction_hash
          base_token
          base_amount
          quote_amount
          quote_token
          success
          traces {
            type
            height
            block_hash
            timestamp
            transaction_hash
            chain {
              universal_chain_id
              rpc_type
            }
          }
        }
      }
    `),
    variables: { packet_hash: packetHash },
    refetchInterval: "1 second",
    writeData: (data) =>
      pipe(
        data,
        Option.map(x => x.v2_transfers),
        Option.flatMap(A.head),
        Option.match({
          onNone: () => {
            transferDetails.error = Option.some(new NoSuchElementException())
          },
          onSome: (data) => {
            transferDetails.data = Option.some(data)
          },
        }),
      ),
    writeError: error => {
      transferDetails.error = error
    },
  })
