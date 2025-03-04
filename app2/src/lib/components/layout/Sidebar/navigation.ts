import SharpRightArrowIcon from "$lib/components/icons/SharpRightArrowIcon.svelte"
import type { Component } from "svelte"

export interface NavItem {
  path: string
  title: string
  icon: Component
}

export interface NavSection {
  title?: string
  items: NavItem[]
}

export const navigation: NavSection[] = [
  {
    items: [
      {
        path: "/transfer",
        title: "Submit Transfer",
        icon: SharpRightArrowIcon
      },
      {
        path: "/transfers",
        title: "Your Transfers",
        icon: SharpRightArrowIcon
      }
    ]
  },
  {
    title: "Explorer",
    items: [
      {
        path: "/explorer/transfers",
        title: "Transfers",
        icon: SharpRightArrowIcon
      },
      {
        path: "/explorer/packets",
        title: "Packets",
        icon: SharpRightArrowIcon
      },
      {
        path: "/explorer/connections",
        title: "Connections",
        icon: SharpRightArrowIcon
      },
      {
        path: "/explorer/channels",
        title: "Channels",
        icon: SharpRightArrowIcon
      },
      {
        path: "/explorer/clients",
        title: "Clients",
        icon: SharpRightArrowIcon
      }
    ]
  }
]
