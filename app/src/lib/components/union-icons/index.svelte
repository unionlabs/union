<script lang="ts">
import ColorBlocks from "$lib/components/union-icons/color/icon-blocks-color.svelte"
import ColorChannel from "$lib/components/union-icons/color/icon-channel.svelte"
import ColorConnection from "$lib/components/union-icons/color/icon-connection-color.svelte"
import ColorExplorer from "$lib/components/union-icons/color/icon-explorer-color.svelte"
import ColorFaucet from "$lib/components/union-icons/color/icon-faucet-color.svelte"
import ColorIndex from "$lib/components/union-icons/color/icon-index.svelte"
import ColorQueue from "$lib/components/union-icons/color/icon-queue.svelte"
import ColorSearch from "$lib/components/union-icons/color/icon-search-color.svelte"
import ColorTransfers from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import ColorUserTransfers from "$lib/components/union-icons/color/icon-usertransfers-color.svelte"
import ColorWallet from "$lib/components/union-icons/color/icon-wallet-color.svelte"

import MonoBlocks from "$lib/components/union-icons/mono/icon-blocks-mono.svelte"
import MonoChannel from "$lib/components/union-icons/mono/icon-channel-mono.svelte"
import MonoConnection from "$lib/components/union-icons/mono/icon-connection-mono.svelte"
import MonoExplorer from "$lib/components/union-icons/mono/icon-explorer-mono.svelte"
import MonoFaucet from "$lib/components/union-icons/mono/icon-faucet-mono.svelte"
import MonoIndex from "$lib/components/union-icons/mono/icon-index-mono.svelte"
import MonoQueue from "$lib/components/union-icons/mono/icon-queue-mono.svelte"
import MonoSearch from "$lib/components/union-icons/mono/icon-search-mono.svelte"
import MonoTransfers from "$lib/components/union-icons/mono/icon-transfers-mono.svelte"
import MonoUserTransfers from "$lib/components/union-icons/mono/icon-usertransfers-mono.svelte"
import MonoWallet from "$lib/components/union-icons/mono/icon-wallet-mono.svelte"

import InverseBlocks from "$lib/components/union-icons/inverse/icon-blocks-inverse.svelte"
import InverseChannel from "$lib/components/union-icons/inverse/icon-channel-inverse.svelte"
import InverseConnection from "$lib/components/union-icons/inverse/icon-connection-inverse.svelte"
import InverseExplorer from "$lib/components/union-icons/inverse/icon-explorer-inverse.svelte"
import InverseFaucet from "$lib/components/union-icons/inverse/icon-faucet-inverse.svelte"
import InverseIndex from "$lib/components/union-icons/inverse/icon-index-inverse.svelte"
import InverseQueue from "$lib/components/union-icons/inverse/icon-queue-inverse.svelte"
import InverseSearch from "$lib/components/union-icons/inverse/icon-search-inverse.svelte"
import InverseTransfers from "$lib/components/union-icons/inverse/icon-transfers-inverse.svelte"
import InverseUserTransfers from "$lib/components/union-icons/inverse/icon-usertransfers-inverse.svelte"
import InverseWallet from "$lib/components/union-icons/inverse/icon-wallet-inverse.svelte"

type IconName =
  | "blocks"
  | "channel"
  | "connection"
  | "explorer"
  | "faucet"
  | "index"
  | "queue"
  | "search"
  | "transfers"
  | "usertransfers"
  | "wallet"

let icons = [
  {
    name: "blocks",
    variants: {
      color: ColorBlocks,
      mono: MonoBlocks,
      inverse: InverseBlocks
    }
  },
  {
    name: "channel",
    variants: {
      color: ColorChannel,
      mono: MonoChannel,
      inverse: InverseChannel
    }
  },
  {
    name: "connection",
    variants: {
      color: ColorConnection,
      mono: MonoConnection,
      inverse: InverseConnection
    }
  },
  {
    name: "explorer",
    variants: {
      color: ColorExplorer,
      mono: MonoExplorer,
      inverse: InverseExplorer
    }
  },
  {
    name: "faucet",
    variants: {
      color: ColorFaucet,
      mono: MonoFaucet,
      inverse: InverseFaucet
    }
  },
  {
    name: "index",
    variants: {
      color: ColorIndex,
      mono: MonoIndex,
      inverse: InverseIndex
    }
  },
  {
    name: "queue",
    variants: {
      color: ColorQueue,
      mono: MonoQueue,
      inverse: InverseQueue
    }
  },
  {
    name: "search",
    variants: {
      color: ColorSearch,
      mono: MonoSearch,
      inverse: InverseSearch
    }
  },
  {
    name: "transfers",
    variants: {
      color: ColorTransfers,
      mono: MonoTransfers,
      inverse: InverseTransfers
    }
  },
  {
    name: "usertransfers",
    variants: {
      color: ColorUserTransfers,
      mono: MonoUserTransfers,
      inverse: InverseUserTransfers
    }
  },
  {
    name: "wallet",
    variants: {
      color: ColorWallet,
      mono: MonoWallet,
      inverse: InverseWallet
    }
  }
]

let className: string | undefined = undefined
export let theme: "color" | "mono" | "inverse" = "color"
export let name: IconName | undefined

export { className as class }

let iconName = icons.find(e => e.name === name)
$: display = iconName ? iconName.variants[theme] : null
</script>

{#if display}
  <svelte:component this={display} class={className}/>
{:else}
  <p>Icon not found.</p>
{/if}

