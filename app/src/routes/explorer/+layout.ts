import type { LayoutLoad } from "./$types.ts"
import IndexIcon from "$lib/components/union-icons/color/icon-index-color.svelte"
import ChannelsIcon from "$lib/components/union-icons/color/icon-channel-color.svelte"
import TransfersIcon from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import ConnectionIcon from "$lib/components/union-icons/color/icon-connection-color.svelte"

const tables = [
  //
  "channels",
  "transfers",
  "packets",
  "connections",
  "index-status"
] as const

export interface Table {
  route: (typeof tables)[number]
  icon: typeof TransfersIcon
  description: string
}

export const load = (loadEvent => ({
  sourceRoute: loadEvent.route,
  tables: [
    {
      route: "transfers",
      icon: TransfersIcon,
      description: "All transfers"
    },
    {
      route: "packets",
      icon: TransfersIcon,
      description: "All packets"
    },
    {
      route: "connections",
      icon: ConnectionIcon,
      description: "Confirmed IBC Connections based on on-chain four-way handshake events."
    },
    {
      route: "channels",
      icon: ChannelsIcon,
      description: "Open IBC Channels"
    },
    {
      route: "index-status",
      icon: IndexIcon,
      description: "Statuses of Hubble indices for connected chains"
    }
  ] as const satisfies Array<Table>
})) satisfies LayoutLoad
