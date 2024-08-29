import type { LayoutLoad } from "./$types.ts"
import TransfersIcon from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import { UnionIcons } from "$lib/components/union-icons/union-icons.ts"

const tables = [
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
      icon: UnionIcons.transfers.variants.color,
      description: "All transfers"
    },
    {
      route: "packets",
      icon: UnionIcons.packet.variants.color,
      description: "All packets"
    },
    {
      route: "connections",
      icon: UnionIcons.connection.variants.color,
      description: "Confirmed IBC Connections based on on-chain four-way handshake events."
    },
    {
      route: "channels",
      icon: UnionIcons.channel.variants.color,
      description: "Open IBC Channels"
    },
    {
      route: "index-status",
      icon: UnionIcons.index.variants.color,
      description: "Statuses of Hubble indices for connected chains"
    }
  ] as const satisfies Array<Table>
})) satisfies LayoutLoad
