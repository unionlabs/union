import { URLS } from "$lib/constants"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"
import { packetsLatestQuery } from "$lib/graphql/queries/packets"
import { derived, type Readable } from "svelte/store"
import { createQuery } from "@tanstack/svelte-query"
import { readFragment, type FragmentOf } from "gql.tada"
import request from "graphql-request"

const packetTransform = (p: FragmentOf<typeof packetListDataFragment>) => {
  const packet = readFragment(packetListDataFragment, p)
  return {
    source: {
      chain_id: packet.from_chain_id ?? "unknown",
      connection_id: packet.from_connection_id ?? "unknown",
      channel_id: packet.from_channel_id ?? "unknown",
      port_id: packet.from_port_id ?? "unknown"
    },
    destination: {
      chain_id: packet.to_chain_id ?? "unknown",
      connection_id: packet.to_connection_id ?? "unknown",
      channel_id: packet.to_channel_id ?? "unknown",
      port_id: packet.to_port_id ?? "unknown"
    },
    timestamp: packet.source_time,
    destination_time: packet.destination_time
  }
}

type PacketsReturnType = Promise<Array<ReturnType<typeof packetTransform>>>

export async function packetsLatest({ limit = 12 }: { limit?: number } = {}): PacketsReturnType {
  const { v0_packets } = await request(URLS.GRAPHQL, packetsLatestQuery, {
    limit
  })
  return v0_packets.map(packetTransform)
}

export const packetsQuery = (timestamp: Readable<string | null>) =>
  createQuery(
    derived([timestamp], ([$timestamp]) => ({
      queryKey: ["packets", "latest"],
      refetchInterval: 5_000,
      queryFn: async () => await packetsLatest({})
    }))
  )
