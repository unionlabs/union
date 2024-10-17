import { URLS } from "$lib/constants"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"
import {
  packetsByChainLatestQuery,
  packetsByChainTimestampQuery,
  packetsByChannelIdLatestQuery,
  packetsByChannelIdTimestampQuery,
  packetsByConnectionIdLatestQuery,
  packetsByConnectionIdTimestampQuery,
  packetsLatestQuery,
  packetsTimestampQuery
} from "$lib/graphql/queries/packets"
import { derived, type Readable } from "svelte/store"
import { createQuery, keepPreviousData } from "@tanstack/svelte-query"
import { readFragment, type FragmentOf } from "gql.tada"
import request from "graphql-request"
import { packetDetailsQueryDocument } from "$lib/graphql/queries/packet-details"

const packetTransform = (p: FragmentOf<typeof packetListDataFragment>) => {
  const packet = readFragment(packetListDataFragment, p)
  return {
    url: `/explorer/packets/${packet.source_chain_id}/${packet.source_connection_id}/${packet.source_channel_id}/${packet.source_sequence}`,
    source: {
      chain_id: packet.source_chain_id ?? "unknown",
      connection_id: packet.source_connection_id ?? "unknown",
      channel_id: packet.source_channel_id ?? "unknown",
      port_id: packet.source_port_id ?? "unknown",
      sequence: packet.source_sequence ?? "unknown"
    },
    destination: {
      chain_id: packet.destination_chain_id ?? "unknown",
      connection_id: packet.destination_connection_id ?? "unknown",
      channel_id: packet.destination_channel_id ?? "unknown",
      port_id: packet.destination_port_id ?? "unknown",
      sequence: packet.destination_sequence ?? "unknown"
    },
    source_sequence: {
      sequence: packet.source_sequence,
      timestamp: packet.source_timestamp
    },
    destination_sequence: {
      sequence: packet.destination_sequence,
      timestamp: packet.destination_timestamp
    },
    timestamp: packet.source_timestamp,
    destination_time: packet.destination_timestamp
  }
}

type PacketsReturnType = Promise<Array<ReturnType<typeof packetTransform>>>

export async function packetsLatest({ limit = 12 }: { limit?: number } = {}): PacketsReturnType {
  const { v1_packets } = await request(URLS().GRAPHQL, packetsLatestQuery, {
    limit
  })
  return v1_packets.map(packetTransform)
}

export async function packetsTimestamp({
  limit,
  timestamp
}: { limit: number; timestamp: string }): PacketsReturnType {
  const { newer, older } = await request(URLS().GRAPHQL, packetsTimestampQuery, {
    timestamp,
    limit: limit / 2
  })

  return [...newer.toReversed(), ...older].map(packetTransform)
}

export async function packetsByChainIdLatest({
  limit,
  chain_id
}: { limit: number; chain_id: string }): PacketsReturnType {
  const { v1_packets } = await request(URLS().GRAPHQL, packetsByChainLatestQuery, {
    limit,
    chain_id
  })
  return v1_packets.map(packetTransform)
}

export async function packetsByChainIdTimestamp({
  limit,
  chain_id,
  timestamp
}: { limit: number; chain_id: string; timestamp: string }): PacketsReturnType {
  const { newer, older } = await request(URLS().GRAPHQL, packetsByChainTimestampQuery, {
    limit,
    chain_id,
    timestamp
  })
  return [...newer.toReversed(), ...older].map(packetTransform)
}

export async function packetsByConnectionIdLatest({
  limit,
  chain_id,
  connection_id
}: { limit: number; chain_id: string; connection_id: string }): PacketsReturnType {
  const { v1_packets } = await request(URLS().GRAPHQL, packetsByConnectionIdLatestQuery, {
    limit,
    chain_id,
    connection_id
  })
  return v1_packets.map(packetTransform)
}

export async function packetsByConnectionIdTimestamp({
  limit,
  chain_id,
  connection_id,
  timestamp
}: {
  limit: number
  chain_id: string
  connection_id: string
  timestamp: string
}): PacketsReturnType {
  const { newer, older } = await request(URLS().GRAPHQL, packetsByConnectionIdTimestampQuery, {
    limit,
    chain_id,
    connection_id,
    timestamp
  })
  return [...newer.toReversed(), ...older].map(packetTransform)
}

export async function packetsByChannelIdLatest({
  limit,
  chain_id,
  connection_id,
  channel_id
}: {
  limit: number
  chain_id: string
  connection_id: string
  channel_id: string
}): PacketsReturnType {
  const { v1_packets } = await request(URLS().GRAPHQL, packetsByChannelIdLatestQuery, {
    limit,
    chain_id,
    connection_id,
    channel_id
  })
  return v1_packets.map(packetTransform)
}

export async function packetsByChannelIdTimestamp({
  limit,
  chain_id,
  connection_id,
  channel_id,
  timestamp
}: {
  limit: number
  chain_id: string
  connection_id: string
  channel_id: string
  timestamp: string
}): PacketsReturnType {
  const { newer, older } = await request(URLS().GRAPHQL, packetsByChannelIdTimestampQuery, {
    limit,
    chain_id,
    connection_id,
    channel_id,
    timestamp
  })
  return [...newer.toReversed(), ...older].map(packetTransform)
}

export const packetsQuery = (limit: number, timestamp: Readable<string | null>) =>
  createQuery(
    derived([timestamp], ([$timestamp]) =>
      $timestamp
        ? {
            queryKey: ["packets", $timestamp],
            refetchOnMount: false,
            refetchOnReconnect: false,
            placeholderData: keepPreviousData,
            staleTime: Number.POSITIVE_INFINITY,
            queryFn: async () => await packetsTimestamp({ limit, timestamp: $timestamp })
          }
        : {
            queryKey: ["packets", "latest"],
            refetchInterval: 5_000,
            placeholderData: keepPreviousData,
            queryFn: async () => await packetsLatest({ limit })
          }
    )
  )

export const packetsByChainIdQuery = (
  limit: number,
  chain_id: string,
  timestamp: Readable<string | null>
) =>
  createQuery(
    derived([timestamp], ([$timestamp]) =>
      $timestamp
        ? {
            queryKey: ["packets", chain_id, $timestamp],
            refetchOnMount: false,
            refetchOnReconnect: false,
            placeholderData: keepPreviousData,
            staleTime: Number.POSITIVE_INFINITY,
            queryFn: async () =>
              await packetsByChainIdTimestamp({ limit, chain_id, timestamp: $timestamp })
          }
        : {
            queryKey: ["packets", chain_id, "latest"],
            refetchInterval: 5_000,
            placeholderData: keepPreviousData,
            queryFn: async () => await packetsByChainIdLatest({ limit, chain_id })
          }
    )
  )

export const packetsByConnectionIdQuery = (
  limit: number,
  chain_id: string,
  connection_id: string,
  timestamp: Readable<string | null>
) =>
  createQuery(
    derived([timestamp], ([$timestamp]) =>
      $timestamp
        ? {
            queryKey: ["packets", chain_id, connection_id, $timestamp],
            refetchOnMount: false,
            refetchOnReconnect: false,
            placeholderData: keepPreviousData,
            staleTime: Number.POSITIVE_INFINITY,
            queryFn: async () =>
              await packetsByConnectionIdTimestamp({
                limit,
                chain_id,
                connection_id,
                timestamp: $timestamp
              })
          }
        : {
            queryKey: ["packets", chain_id, connection_id, "latest"],
            refetchInterval: 5_000,
            placeholderData: keepPreviousData,
            queryFn: async () =>
              await packetsByConnectionIdLatest({ limit, chain_id, connection_id })
          }
    )
  )

export const packetsByChannelIdQuery = (
  limit: number,
  chain_id: string,
  connection_id: string,
  channel_id: string,
  timestamp: Readable<string | null>
) =>
  createQuery(
    derived([timestamp], ([$timestamp]) =>
      $timestamp
        ? {
            queryKey: ["packets", chain_id, connection_id, channel_id, $timestamp],
            refetchOnMount: false,
            refetchOnReconnect: false,
            placeholderData: keepPreviousData,
            staleTime: Number.POSITIVE_INFINITY,
            queryFn: async () =>
              await packetsByChannelIdTimestamp({
                limit,
                chain_id,
                connection_id,
                channel_id,
                timestamp: $timestamp
              })
          }
        : {
            queryKey: ["packets", chain_id, connection_id, channel_id, "latest"],
            refetchInterval: 5_000,
            placeholderData: keepPreviousData,
            queryFn: async () =>
              await packetsByChannelIdLatest({ limit, chain_id, connection_id, channel_id })
          }
    )
  )

export const packetDetailsQuery = (
  chain_id: string,
  connection_id: string,
  channel_id: string,
  sequence: number
) =>
  createQuery({
    queryKey: ["packets", chain_id, connection_id, channel_id, sequence],
    refetchInterval: 5_000,
    placeholderData: keepPreviousData,
    queryFn: async () =>
      await request(URLS().GRAPHQL, packetDetailsQueryDocument, {
        chain_id,
        connection_id,
        channel_id,
        sequence
      })
  })
