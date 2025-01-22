import { ofetch } from "ofetch"
import { GRAQPHQL_URL } from "#mod"
import { graphql } from "gql.tada"
import { request } from "graphql-request"
import type { ByteArray } from "viem"

const queryHeaders = new Headers({
  Accept: "application/json",
  "User-Agent": "typescript-sdk",
  "Content-Type": "application/json"
})

const channelsQuery = graphql(/*  GraphQL */ `
  query Ucs03Channels {
    v1_ibc_union_channel_recommendations(where: {version: {_eq: "ucs03-zkgm-0"}}) {
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

export const getRecommendedChannels = async () => {
  return (await request(GRAQPHQL_URL, channelsQuery)).v1_ibc_union_channel_recommendations
}

export const getChannelInfo = (
  source_chain_id: string,
  destination_chain_id: string,
  channels: Awaited<ReturnType<typeof getRecommendedChannels>>
): {
  source_port_id: string
  source_channel_id: number
  source_connection_id: number
  destination_port_id: string
  destination_channel_id: number
  destination_connection_id: number
} | null => {
  let rawChannel = channels.find(
    chan => (
      chan.source_chain_id === source_chain_id, chan.destination_chain_id === destination_chain_id
    )
  )
  if (
    // Validate that all required fields are included by the garphql api.
    !rawChannel ||
    rawChannel.source_connection_id === null ||
    rawChannel.source_channel_id === null ||
    !rawChannel.source_port_id ||
    rawChannel.destination_connection_id === null ||
    rawChannel.destination_channel_id === null ||
    !rawChannel.destination_port_id
  ) {
    return null
  }

  let source_port_id = String(rawChannel.source_port_id)
  if (source_port_id.length < 4) return null
  source_port_id = source_port_id.slice(2)

  let destination_port_id = String(rawChannel.destination_port_id)
  if (destination_port_id.length < 4) return null
  destination_port_id = destination_port_id.slice(2)

  console.log("source port", rawChannel.source_port_id)
  console.log("source port string", String(rawChannel.source_port_id))

  return {
    source_connection_id: rawChannel.source_connection_id,
    source_channel_id: rawChannel.source_channel_id,
    source_port_id,
    destination_connection_id: rawChannel.destination_connection_id,
    destination_channel_id: rawChannel.destination_channel_id,
    destination_port_id
  }
}
