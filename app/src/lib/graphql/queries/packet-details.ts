import { graphql } from "../index.ts"
import { packetDetailsFragment } from "$lib/graphql/fragments/packets.ts"

export const packetDetailsQueryDocument = graphql(
  /* GraphQL */ `
  query PacketDetailsQuery($chain_id: String!, $connection_id: Int!, $channel_id: Int!, $packet_send_transaction_hash: String!)
  @cached(ttl: 1) {
    v1_ibc_union_packets(
      where: { _and: [
        {source_chain_id: { _eq: $chain_id }} 
        {source_connection_id: { _eq: $connection_id }} 
        {source_channel_id: { _eq: $channel_id }}
        {packet_send_transaction_hash: { _eq: $packet_send_transaction_hash }}
      ] }
    ) {
      ...PacketDetails
    }
  }
`,
  [packetDetailsFragment]
)
