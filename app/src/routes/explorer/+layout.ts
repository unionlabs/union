import { redirect } from "@sveltejs/kit"
import type { LayoutLoad } from "./$types.ts"
import type { SvelteComponent } from "svelte"

import SendHorizontalIcon from "virtual:icons/lucide/send-horizontal"
import TransfersIcon from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import UserTransfersIcon from "$lib/components/union-icons/color/icon-usertransfers-color.svelte"
import BlocksIcon from "$lib/components/union-icons/color/icon-blocks-color.svelte"
import ConnectionIcon from "$lib/components/union-icons/color/icon-connection-color.svelte"
import ChannelsIcon from "$lib/components/union-icons/color/icon-channel.svelte"
import IndexIcon from "$lib/components/union-icons/color/icon-index.svelte"

const tables = ["blocks", "packets", "channels", "connections"] as const

export interface Table {
  route: (typeof tables)[number]
  icon: typeof SvelteComponent
  description: string
}

export const load = (loadEvent => {
  // Redirect if the user is visiting /explorer
  if (loadEvent.url.pathname === "/explorer") throw redirect(302, "/explorer/transfers")

  return {
    tables: [
      {
        route: "transfers",
        icon: TransfersIcon,
        description: "All UCS-01 transfers"
      },
      // {
      //   route: "user-transfers",
      //   icon: UserTransfersIcon,
      //   description: "Your UCS-01 transfers"
      // },
      {
        route: "blocks",
        icon: BlocksIcon,
        description: "Blocks from all chains indexed by Hubble"
      },
      {
        route: "connections",
        icon: ConnectionIcon,
        description:
          "IBC Connections based on on-chain handshake events. Status is only 'CONFIRM' if we have indexed the entire four-way handshake."
      },
      { route: "channels", icon: ChannelsIcon, description: "Open IBC Channels" },
      { route: "packets", icon: SendHorizontalIcon, description: "Packets sent through Union" },
      {
        route: "index-status",
        icon: IndexIcon,
        description: "Statuses of Hubble indices for connected chains"
      }
    ] as Array<Table>
  }
}) satisfies LayoutLoad
