import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { packetDetails } from "$lib/stores/packets.svelte"
import { PacketDetails, type PacketHash } from "@unionlabs/sdk/schema"

export const packetDetailsQuery = (packetHash: PacketHash) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: Schema.Array(PacketDetails) }),
    document: graphql(`
      query PacketDetails($packet_hash: String!) @cached(ttl: 30) {
        v2_packets(args: {
          p_packet_hash: $packet_hash
        }) {
          packet_hash
          channel_version
          data
          destination_chain_id
          destination_channel_id
          destination_client_id
          destination_connection_id
          destination_port_id
          destination_universal_chain_id
          packet_ack_block_hash
          packet_ack_height
          packet_ack_maker
          packet_ack_timestamp
          packet_ack_transaction_hash
          packet_recv_block_hash
          packet_recv_height
          packet_recv_maker
          packet_recv_maker_msg
          packet_recv_timestamp
          packet_recv_transaction_hash
          packet_send_block_hash
          packet_send_height
          packet_send_timestamp
          packet_send_transaction_hash
          sort_order
          source_channel_id
          source_client_id
          source_connection_id
          source_port_id
          source_universal_chain_id
          status
          timeout_height
          timeout_timestamp
          write_ack_block_hash
          write_ack_height
          write_ack_timestamp
          write_ack_transaction_hash
          decoded
          decoded_flattened
          acknowledgement
          traces {
            type
            height
            block_hash
            timestamp
            transaction_hash
            chain {
              universal_chain_id
            }
          }
          
        }
      }
    `),
    variables: { packet_hash: packetHash },
    refetchInterval: "30 seconds",
    writeData: data => {
      data.pipe(
        Option.map(d => {
          if (d.v2_packets.length === 0) {
            throw { _tag: "NotFound", message: "Packet not found" }
          }
          return d.v2_packets[0]
        }),
        Option.tap(packet => {
          packetDetails.data = Option.some(packet)
        })
      )
    },
    writeError: error => {
      packetDetails.error = error
    }
  })
