import { graphql } from "gql.tada"

export const packetListItemFragment = graphql(`
    fragment PacketListItem on v2_packet_type {
        packet_hash
        channel_version
        destination_chain_id
        destination_channel_id
        destination_universal_chain_id
        source_channel_id
        source_universal_chain_id
        packet_send_timestamp
        packet_recv_timestamp
        packet_ack_timestamp
        sort_order
        status
    }
`)
