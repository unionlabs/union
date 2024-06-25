import type { LayoutLoad } from "./$types.ts"
import type { SvelteComponent } from "svelte"

import TransfersIcon from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import BlocksIcon from "$lib/components/union-icons/color/icon-blocks-color.svelte"
import ConnectionIcon from "$lib/components/union-icons/color/icon-connection-color.svelte"
import ChannelsIcon from "$lib/components/union-icons/color/icon-channel-color.svelte"
import IndexIcon from "$lib/components/union-icons/color/icon-index-color.svelte"
import PacketIcon from "$lib/components/union-icons/color/icon-packet-color.svelte"

const tables = ["blocks", "packets", "channels", "connections"] as const

export interface Table {
  route: (typeof tables)[number]
  icon: typeof SvelteComponent
  description: string
}

export const load = (loadEvent => ({
  sourceRoute: loadEvent.route,
  tables: [
    {
      route: "transfers",
      icon: TransfersIcon,
      description: "All UCS-01 transfers"
    },
    // {
    //   route: "blocks",
    //   icon: BlocksIcon,
    //   description: "Blocks from all chains indexed by Hubble"
    // },
    {
      route: "connections",
      icon: ConnectionIcon,
      description:
        "Confirmed IBC Connections based on on-chain four-way handshake events."
    },
    { route: "channels", icon: ChannelsIcon, description: "Open IBC Channels" },
    { route: "packets", icon: PacketIcon, description: "Packets sent through Union" },
    {
      route: "index-status",
      icon: IndexIcon,
      description: "Statuses of Hubble indices for connected chains"
    }
  ] as Array<Table>
})) satisfies LayoutLoad
