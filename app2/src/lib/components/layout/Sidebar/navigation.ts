import SharpTransferIcon from "$lib/components/icons/SharpTransferIcon.svelte"
import SharpListIcon from "$lib/components/icons/SharpListIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import SharpPacketsIcon from "$lib/components/icons/SharpPacketsIcon.svelte"
import SharpConnectionsIcon from "$lib/components/icons/SharpConnectionsIcon.svelte"
import SharpChannelsIcon from "$lib/components/icons/SharpChannelsIcon.svelte"
import SharpClientsIcon from "$lib/components/icons/SharpClientsIcon.svelte"
import OutlineControlPointDuplicate from "$lib/components/icons/OutlineControlPointDuplicate.svelte"
import type { Component } from "svelte"

export interface NavSubItem {
  path: string
  title: string
}

export interface NavItem {
  path: string
  title: string
  icon: Component
  subroutes?: Array<NavSubItem>
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
      }
    ]
  },
  {
    title: "Explorer",
    items: [
      {
        path: "/explorer",
        title: "Explorer",
        icon: SharpListIcon,
        subroutes: [
          {
            path: "/explorer/transfers",
            title: "Transfers"
          },
          {
            path: "/explorer/packets",
            title: "Packets"
          },
          {
            path: "/explorer/connections",
            title: "Connections"
          },
          {
            path: "/explorer/channels",
            title: "Channels"
          },
          {
            path: "/explorer/clients",
            title: "Clients"
          }
        ]
      }
    ]
  },
  {
    title: "Developer",
    items: [
      {
        path: "/transfer/native",
        title: "Transfer Native",
        icon: SharpTransferIcon
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
      },
      {
        path: "/balances",
        title: "Your Balances",
        icon: OutlineControlPointDuplicate
      },
      {
        path: "/explorer/tokens",
        title: "Tokens",
        icon: OutlineControlPointDuplicate
      }
    ]
  }
]
