import { URLS } from "$lib/constants"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"
import { packetsLatestQueryDocument } from "$lib/graphql/queries/packets"
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

export const packetsQuery = () =>
  createQuery({
    queryKey: ["packets"],
    refetchInterval: 5_000,
    queryFn: async () => request(URLS.GRAPHQL, packetsLatestQueryDocument, {}),
    select: data => data.v0_packets.map(packetTransform)
  })
