import type { LayoutLoad } from "./$types"
import type { SvelteComponent } from "svelte"

import TvIcon from "virtual:icons/lucide/tv"
import BlocksIcon from "virtual:icons/lucide/blocks"
import ConnectionIcon from "virtual:icons/mdi/connection"
import SendHorizontalIcon from "virtual:icons/lucide/send-horizontal"

const tables = ["blocks", "packets", "channels", "connections"] as const

export interface Table {
  route: (typeof tables)[number]
  icon: typeof SvelteComponent
}

export const load = (_loadEvent => ({
  tables: [
    { route: "blocks", icon: BlocksIcon },
    { route: "channels", icon: TvIcon },
    { route: "packets", icon: SendHorizontalIcon },
    { route: "connections", icon: ConnectionIcon }
  ] as Array<Table>
})) satisfies LayoutLoad
