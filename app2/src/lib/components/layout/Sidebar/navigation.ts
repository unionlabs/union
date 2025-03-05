import SharpTransferIcon from "$lib/components/icons/SharpTransferIcon.svelte"
import SharpListIcon from "$lib/components/icons/SharpListIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import SharpPacketsIcon from "$lib/components/icons/SharpPacketsIcon.svelte"
import SharpConnectionsIcon from "$lib/components/icons/SharpConnectionsIcon.svelte"
import SharpChannelsIcon from "$lib/components/icons/SharpChannelsIcon.svelte"
import SharpClientsIcon from "$lib/components/icons/SharpClientsIcon.svelte"
import type { Component } from "svelte"

export interface NavItem {
  path: string
  title: string
  icon: Component
}

export interface NavSection {
  title?: string
  items: Array<NavItem>
}

export const navigation: Array<NavSection> = [
  {
    items: [
      {
        path: "/transfer",
        title: "Transfer",
        icon: SharpTransferIcon
      },
      {
        path: "/transfers",
        title: "Your Transfers",
        icon: SharpListIcon
      },
      {
        path: "/dashboard",
        title: "Dashboard",
        icon: SharpDashboardIcon
      },
      {
        path: "/stake",
        title: "Stake with Escher",
        icon: SharpStakeIcon
      }
    ]
  },
  {
    title: "Explorer",
    items: [
      {
        path: "/explorer/tokens",
        title: "Tokens",
        icon: SharpListIcon
      },
      {
        path: "/explorer/transfers",
        title: "Transfers",
        icon: SharpListIcon
      },
      {
        path: "/explorer/packets",
        title: "Packets",
        icon: SharpPacketsIcon
      },
      {
        path: "/explorer/connections",
        title: "Connections",
        icon: SharpConnectionsIcon
      },
      {
        path: "/explorer/channels",
        title: "Channels",
        icon: SharpChannelsIcon
      },
      {
        path: "/explorer/clients",
        title: "Clients",
        icon: SharpClientsIcon
      }
    ]
  }
]
