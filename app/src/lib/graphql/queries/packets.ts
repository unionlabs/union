import { graphql } from "gql.tada"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"

export const packetsLatestQuery = graphql(
  /* GraphQL */ `
    query PacketsLatestQuery($limit: Int = 100) {
      v0_packets(limit: $limit, order_by: { source_time: desc_nulls_last }) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsTimestampQuery = graphql(
  /* GraphQL */ `
  query PacketsTimestampQuery($limit: Int! = 100, $timestamp: timestamptz!)
    @cached(ttl: 1000) {
      newer: v0_packets(
        limit: $limit
        order_by: [{ source_time: asc }, { destination_time: asc }]
        where: { source_time: { _gte: $timestamp } }
      ) {
        ...PacketListData
      }
      older: v0_packets(
        limit: $limit
        order_by: [
          { source_time: desc }
          { destination_time: desc }
        ]
        where: { source_time: { _lt: $timestamp } }
      ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByChainLatestQuery = graphql(
  /* GraphQL */ `
    query PacketsByChainLatestQuery($limit: Int, $chain_id: String!) {
      v0_packets(
        limit: $limit 
        order_by: { source_time: desc_nulls_last }
        where: { _or: [
          { from_chain_id: { _eq: $chain_id }}
          { to_chain_id: { _eq: $chain_id }}
        ]}
        ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)
