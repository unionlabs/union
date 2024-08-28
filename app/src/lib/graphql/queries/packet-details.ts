import { graphql } from "../index.ts"
import { packetDetailsFragment } from "$lib/graphql/fragments/packets.ts"

export const packetDetailsQueryDocument = graphql(
  /* GraphQL */ `
  query PacketDetailsQuery($chain_id: String!, $connection_id: String!, $channel_id: String! $sequence: bigint)
  @cached(ttl: 1) {
    v1_packets(
      where: { _and: [
        {source_chain_id: { _eq: $chain_id }} 
        {source_connection_id: { _eq: $connection_id }} 
        {source_channel_id: { _eq: $channel_id }}
        {source_sequence: { _eq: $sequence }}
      ] }
    ) {
      ...PacketDetails
    }
  }
`,
  [packetDetailsFragment]
)