import { graphql } from "gql.tada"

export const channelsQuery =
  graphql(/* GraphQL */ `query ChannelsQuery($limit: Int = 500) @cached(ttl: 30) {
  v1_ibc_union_channel_recommendations(
    where: {source_chain: {enabled: {_eq: true}}, destination_chain: {enabled: {_eq: true}}},
    order_by: [
    {status: asc}, 
    {source_chain_id: asc}, 
    {source_connection_id: asc},
    {source_channel_id: asc},
    {source_port_id: asc},
    {destination_chain_id: asc}, 
    {destination_connection_id: asc}, 
    {destination_channel_id: asc},
    {destination_port_id: asc}
  ], limit: $limit) {
      source_chain_id
      source_connection_id
      source_channel_id
      source_port_id
      source_chain {
          enabled
          display_name
      }
      destination_chain_id
      destination_connection_id
      destination_channel_id
      destination_port_id
      destination_chain {
          enabled
          display_name
      }
      status
      version
  }
}`)

export const recommendedUcs03ChannelsQueryDocument = graphql(/*  GraphQL */ `
  query Ucs03Channels @cached(ttl: 60) {
    v1_ibc_union_channel_recommendations(where: {_and: [{version: {_eq: "ucs03-zkgm-0"}}, {destination_chain_id: {_neq: "11155111"}}, {destination_chain_id: {_neq: "17000"}}]}) {
      source_port_id
      source_chain_id
      source_channel_id
      source_connection_id
      destination_port_id
      destination_chain_id
      destination_channel_id
      destination_connection_id
    }
  }
`)
