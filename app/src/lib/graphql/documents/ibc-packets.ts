import { graphql } from "gql.tada"

export const ibcRecvPacketQuery = graphql(/* graphql */ `
  query IBCRecvPacketQuery($limit: Int = 10) @cached(ttl: 1) {
    v0_recv_packet(limit: $limit, order_by: { time: desc }) {
      chain_id
      block_hash    
      transaction_index
      transaction_hash
      timeout_timestamp
      time
      source_port
      source_channel
      sequence
      height
      destination_port
      destination_channel
    }
  }
`)
