import { packetListItemFragment } from "$lib/queries/fragments/packet-list-item"
import { packetList } from "$lib/stores/packets.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import type { SortOrder } from "@unionlabs/sdk/schema"
import { PacketList } from "@unionlabs/sdk/schema"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"

const LIMIT = 10

export let packetListLatestQuery = (limit = LIMIT, mainnetOnly = false) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsLatest($limit: Int!, $network: String) @cached(ttl: 1) {
      v2_packets(args: {
        p_limit: $limit,
        p_network: $network
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment],
    ),
    variables: {
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "1 second",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets))
    },
    writeError: error => {
      packetList.error = error
    },
  })

export let packetListPageLtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT,
  mainnetOnly = false,
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsPage($page: String!, $limit: Int!, $network: String)
    @cached(ttl: 30) {
      v2_packets(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "lt",
        p_network: $network
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment],
    ),
    variables: {
      page,
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "30 seconds",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets))
    },
    writeError: error => {
      packetList.error = error
    },
  })

export let packetListPageGtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT,
  mainnetOnly = false,
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_packets: PacketList }),
    document: graphql(
      `
    query PacketsPage($page: String!, $limit: Int!, $network: String) @cached(ttl: 30) {
      v2_packets(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "gt",
        p_network: $network
      }) {
      ...PacketListItem
      }
    }
  `,
      [packetListItemFragment],
    ),
    variables: {
      page,
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "30 seconds",
    writeData: data => {
      packetList.data = data.pipe(Option.map(d => d.v2_packets.toReversed()))
    },
    writeError: error => {
      packetList.error = error
    },
  })
