import { redirect } from "@sveltejs/kit"
import type { LayoutLoad } from "./$types"
import type { SvelteComponent } from "svelte"

import TvIcon from "virtual:icons/lucide/tv"
import BlocksIcon from "virtual:icons/lucide/blocks"
import RocketIcon from "virtual:icons/lucide/rocket"
import DatabaseIcon from "virtual:icons/lucide/database"
import ConnectionIcon from "virtual:icons/mdi/connection"
import SendHorizontalIcon from "virtual:icons/lucide/send-horizontal"

const tables = ["blocks", "packets", "channels", "connections"] as const

export interface Table {
  route: (typeof tables)[number]
  icon: typeof SvelteComponent
  description: string
}

export const load = (loadEvent => {
  // Redirect if the user is visiting /explorer
  if (loadEvent.url.pathname === "/explorer") throw redirect(302, "/explorer/blocks")

  return {
    tables: [
      {
        route: "blocks",
        icon: BlocksIcon,
        description: "Blocks from all chains indexed by Hubble"
      },
      { route: "connections", icon: ConnectionIcon, description: "IBC Connections based on on-chain handshake events. Status is only 'CONFIRM' if we have indexed the entire four-way handshake." },
      { route: "channels", icon: TvIcon, description: "Open IBC Channels" },
      { route: "packets", icon: SendHorizontalIcon, description: "Packets sent through Union" },
      {
        route: "voyager-queue",
        icon: RocketIcon,
        description: "Voyager Relayer VM Operations Queue"
      },
      {
        route: "index-status",
        icon: DatabaseIcon,
        description: "Statuses of Hubble indices for connected chains"
      }
    ] as Array<Table>
  }
}) satisfies LayoutLoad
