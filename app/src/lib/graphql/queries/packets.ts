import { graphql } from "gql.tada"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"

export const packetsLatestQuery = graphql(
  /* GraphQL */ `
    query PacketsLatestQuery($limit: Int = 100) {
      v1_packets(limit: $limit, order_by: { source_timestamp: desc_nulls_last }) {
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
      newer: v1_packets(
        limit: $limit
        order_by: [{ source_timestamp: asc }, { destination_timestamp: asc }]
        where: { source_timestamp: { _gte: $timestamp } }
      ) {
        ...PacketListData
      }
      older: v1_packets(
        limit: $limit
        order_by: [
          { source_timestamp: desc }
          { destination_timestamp: desc }
        ]
        where: { source_timestamp: { _lt: $timestamp } }
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
      v1_packets(
        limit: $limit 
        order_by: { source_timestamp: desc_nulls_last }
        where: { _or: [
          { source_chain_id: { _eq: $chain_id }}
          { destination_chain_id: { _eq: $chain_id }}
        ]}
        ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByChainTimestampQuery = graphql(
  /* GraphQL */ `
    query PacketsByChainTimestampQuery($limit: Int!, $chain_id: String!, $timestamp: timestamptz!) @cached(ttl: 1000) {
      newer: v1_packets(
        limit: $limit
        order_by: [{ source_timestamp: asc }, { destination_timestamp: asc }]
        where: {
          _and: [
            { source_timestamp: { _gte: $timestamp } }
            {
              _or: [
                { source_chain_id: { _eq: $chain_id }}
                { destination_chain_id: { _eq: $chain_id }}
              ]
            }
          ]
        }

      ) {
        ...PacketListData
      }
      older: v1_packets(
        limit: $limit
        order_by: [ { source_timestamp: desc } { destination_timestamp: desc } ]
        where: {
          _and: [
            { source_timestamp: { _lt: $timestamp } }
            {
              _or: [
                { source_chain_id: { _eq: $chain_id }}
                { destination_chain_id: { _eq: $chain_id }}
              ]
            }
          ]
        }
      ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByConnectionIdLatestQuery = graphql(
  /* GraphQL */ `
    query PacketsByConnectionIdLatestQuery($limit: Int!, $chain_id: String!, $connection_id: String!) {
      v1_packets(
        limit: $limit 
        order_by: { source_timestamp: desc_nulls_last }
        where: { 
          _or: [
            { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }}] }
            { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }}] }
          ]
        }
        ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByConnectionIdTimestampQuery = graphql(
  /* GraphQL */ `
    query PacketsByConnectionIdTimestampQuery($limit: Int!, $chain_id: String!, $connection_id: String!, $timestamp: timestamptz!) @cached(ttl: 1000) {
      newer: v1_packets(
        limit: $limit
        order_by: [{ source_timestamp: asc }, { destination_timestamp: asc }]
        where: {
          _and: [
            { source_timestamp: { _gte: $timestamp } }
            {
              _or: [
                { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }}] }
                { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }}] }
              ]
            }
          ]
        }

      ) {
        ...PacketListData
      }
      older: v1_packets(
        limit: $limit
        order_by: [ { source_timestamp: desc } { destination_timestamp: desc } ]
        where: {
          _and: [
            { source_timestamp: { _lt: $timestamp } }
            {
              _or: [
                { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }}] }
                { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }}] }
              ]
            }
          ]
        }
      ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByChannelIdLatestQuery = graphql(
  /* GraphQL */ `
    query PacketsByChannelIdLatestQuery($limit: Int!, $chain_id: String!, $connection_id: String!, $channel_id: String!) {
      v1_packets(
        limit: $limit 
        order_by: { source_timestamp: desc_nulls_last }
        where: { 
          _or: [
            { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }} {source_channel_id: { _eq: $channel_id }}] }
            { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }} {destination_channel_id: { _eq: $channel_id }}] }
          ]
        }
        ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)

export const packetsByChannelIdTimestampQuery = graphql(
  /* GraphQL */ `
    query PacketsByChannelIdTimestampQuery($limit: Int!, $chain_id: String!, $connection_id: String!, $channel_id: String!,  $timestamp: timestamptz!) @cached(ttl: 1000) {
      newer: v1_packets(
        limit: $limit
        order_by: [{ source_timestamp: asc }, { destination_timestamp: asc }]
        where: {
          _and: [
            { source_timestamp: { _gte: $timestamp } }
            {
              _or: [
                { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }} {source_channel_id: { _eq: $channel_id }}] }
                { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }} {destination_channel_id: { _eq: $channel_id }}] }
              ]
            }
          ]
        }

      ) {
        ...PacketListData
      }
      older: v1_packets(
        limit: $limit
        order_by: [ { source_timestamp: desc } { destination_timestamp: desc } ]
        where: {
          _and: [
            { source_timestamp: { _lt: $timestamp } }
            {
              _or: [
                { _and: [{source_chain_id: { _eq: $chain_id }} {source_connection_id: { _eq: $connection_id }} {source_channel_id: { _eq: $channel_id }}] }
                { _and: [{destination_chain_id: { _eq: $chain_id }} {destination_connection_id: { _eq: $connection_id }} {destination_channel_id: { _eq: $channel_id }}] }
              ]
            }
          ]
        }
      ) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)
