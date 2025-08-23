import DiscordIcon from "$lib/components/icons/DiscordIcon.svelte"
import GithubIcon from "$lib/components/icons/GithubIcon.svelte"
import OutlineControlPointDuplicate from "$lib/components/icons/OutlineControlPointDuplicate.svelte"
import SharpChannelsIcon from "$lib/components/icons/SharpChannelsIcon.svelte"
import SharpDashboardIcon from "$lib/components/icons/SharpDashboardIcon.svelte"
import SharpStakeIcon from "$lib/components/icons/SharpStakeIcon.svelte"
import SharpTransferIcon from "$lib/components/icons/SharpTransferIcon.svelte"
import TwitterIcon from "$lib/components/icons/TwitterIcon.svelte"
import type { Component } from "svelte"

export interface NavSubItem {
  path: string
  title: string
  editions?: Array<string>
  new?: boolean
}

export interface NavItem {
  path: string
  title: string
  icon: Component
  subroutes?: Array<NavSubItem>
  external?: boolean
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
            path: "/transfer/multisig",
            title: "Keplr Multisig",
          },
          {
            path: "/transfers",
            title: "History",
          },
          {
            path: "/faucet",
            title: "Faucets",
            editions: ["app"],
          },
        ],
      },
    ],
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
            title: "Transfers",
          },
          {
            path: "/explorer/packets",
            title: "Packets",
          },
          {
            path: "/explorer/find-packet",
            title: "Find Packet",
          },
          {
            path: "/explorer/orbital",
            title: "Orbital",
            new: true,
          },
          // {
          //   path: "/explorer/connections",
          //   title: "Connections"
          // },
          // {
          //   path: "/explorer/channels",
          //   title: "Channels"
          // },
          {
            path: "/explorer/clients",
            title: "Clients",
            new: true,
          },
        ],
      },
    ],
  },
  {
    items: [{
      path: "/stake",
      title: "Stake",
      icon: SharpStakeIcon,
    }],
  },
  {
    title: "Developer",
    items: [
      {
        path: "/transfer/native",
        title: "Transfer Native",
        icon: SharpTransferIcon,
      },
      {
        path: "/dashboard",
        title: "Dashboard",
        icon: SharpDashboardIcon,
      },
      {
        path: "/balances",
        title: "Your Balances",
        icon: OutlineControlPointDuplicate,
      },
      {
        path: "/explorer/tokens",
        title: "Tokens",
        icon: OutlineControlPointDuplicate,
      },
    ],
  },
  {
    title: "More Union",
    items: [
      {
        path: "https://discord.union.build",
        title: "Discord",
        icon: DiscordIcon,
        external: true,
      },
      {
        path: "https://x.com/@union_build",
        title: "@union_build",
        icon: TwitterIcon,
        external: true,
      },
      {
        path: "https://github.com/unionlabs",
        title: "@unionlabs",
        icon: GithubIcon,
        external: true,
      },
    ],
  },
]
