import SharpTransferIcon from "$lib/components/icons/SharpTransferIcon.svelte"
import SharpListIcon from "$lib/components/icons/SharpListIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import SharpChannelsIcon from "$lib/components/icons/SharpChannelsIcon.svelte"
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
        icon: OutlineControlPointDuplicate,
        subroutes: [
          {
            path: "/transfers",
            title: "History"
          },
          {
            path: "/faucet",
            title: "Faucet"
          }
        ]
      }
    ]
  },
  {
    items: [
      {
        path: "/explorer",
        title: "Explorer",
        icon: SharpChannelsIcon,
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
