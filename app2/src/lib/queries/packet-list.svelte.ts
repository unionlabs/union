import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { packetList } from "$lib/stores/packets.svelte"
import { packetListItemFragment } from "$lib/queries/fragments/packet-list-item"
import { PacketList } from "@unionlabs/sdk/schema"
import type { SortOrder } from "@unionlabs/sdk/schema"

export const LIMIT = 10

export let packetListLatestQuery = (limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsLatest($limit: Int!) @cached(ttl: 1) {
      v2_packets(args: {
        p_limit: $limit
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment]
    ),
    variables: { limit },
    refetchInterval: "1 second",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets))
    },
    writeError: error => {
      packetList.error = error
    }
  })

export let packetListPageLtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsPage($page: String!, $limit: Int!)
    @cached(ttl: 30) {
      v2_packets(args: {
        p_limit: $limit,
        p_sort_order: $page
        p_comparison: "lt"
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets))
    },
    writeError: error => {
      packetList.error = error
    }
  })

export let packetListPageGtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsPage($page: String!, $limit: Int!) @cached(ttl: 30) {
      v2_packets(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "gt"
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets.toReversed()))
    },
    writeError: error => {
      packetList.error = error
    }
  })
