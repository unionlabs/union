<script lang="ts">
import ColorBlocks from "$lib/components/union-icons/color/icon-blocks-color.svelte"
import ColorChannel from "$lib/components/union-icons/color/icon-channel-color.svelte"
import ColorConnection from "$lib/components/union-icons/color/icon-connection-color.svelte"
import ColorExplorer from "$lib/components/union-icons/color/icon-explorer-color.svelte"
import ColorFaucet from "$lib/components/union-icons/color/icon-faucet-color.svelte"
import ColorIndex from "$lib/components/union-icons/color/icon-index-color.svelte"
import ColorQueue from "$lib/components/union-icons/color/icon-queue-color.svelte"
import ColorSearch from "$lib/components/union-icons/color/icon-search-color.svelte"
import ColorTransfers from "$lib/components/union-icons/color/icon-transfers-color.svelte"
import ColorUserTransfers from "$lib/components/union-icons/color/icon-usertransfers-color.svelte"
import ColorWallet from "$lib/components/union-icons/color/icon-wallet-color.svelte"
import ColorPacket from "$lib/components/union-icons/color/icon-packet-color.svelte"
import ColorHome from "$lib/components/union-icons/color/icon-home-color.svelte"
import ColorSettings from "$lib/components/union-icons/color/icon-settings-color.svelte"

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
import MonoPacket from "$lib/components/union-icons/mono/icon-packet-mono.svelte"
import MonoHome from "$lib/components/union-icons/mono/icon-home-mono.svelte"
import MonoSettings from "$lib/components/union-icons/mono/icon-settings-mono.svelte"
import MonoAllTransfers from "$lib/components/union-icons/mono/icon-alltransfers-mono.svelte"
import MonoIbcConnections from "$lib/components/union-icons/mono/icon-ibcconnections-mono.svelte"
import MonoIbcChannels from "$lib/components/union-icons/mono/icon-ibcchannels-mono.svelte"
import MonoHubbleStatus from "$lib/components/union-icons/mono/icon-hubblestatus-mono.svelte"
import MonoSearchK from "$lib/components/union-icons/mono/icon-searchk-mono.svelte"

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
import InversePacket from "$lib/components/union-icons/inverse/icon-packet-inverse.svelte"
import InverseHome from "$lib/components/union-icons/inverse/icon-home-inverse.svelte"
import InverseSettings from "$lib/components/union-icons/inverse/icon-settings-inverse.svelte"

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
  | "packet"
  | "home"
  | "settings"
  | "alltransfers"
  | "ibcconnections"
  | "ibcchannels"
  | "hubblestatus"
  | "searchk"

type IconVariant = "color" | "mono" | "inverse"
type mode = "dark" | "light"

type IconConfig = {
  [key in IconName]: {
    variants: Partial<Record<IconVariant, any>>
  }
}

const icons: IconConfig = {
  blocks: {
    variants: {
      color: ColorBlocks,
      mono: MonoBlocks,
      inverse: InverseBlocks
    }
  },
  channel: {
    variants: {
      color: ColorChannel,
      mono: MonoChannel,
      inverse: InverseChannel
    }
  },
  connection: {
    variants: {
      color: ColorConnection,
      mono: MonoConnection,
      inverse: InverseConnection
    }
  },
  explorer: {
    variants: {
      color: ColorExplorer,
      mono: MonoExplorer,
      inverse: InverseExplorer
    }
  },
  faucet: {
    variants: {
      color: ColorFaucet,
      mono: MonoFaucet,
      inverse: InverseFaucet
    }
  },
  index: {
    variants: {
      color: ColorIndex,
      mono: MonoIndex,
      inverse: InverseIndex
    }
  },
  queue: {
    variants: {
      color: ColorQueue,
      mono: MonoQueue,
      inverse: InverseQueue
    }
  },
  search: {
    variants: {
      color: ColorSearch,
      mono: MonoSearch,
      inverse: InverseSearch
    }
  },
  transfers: {
    variants: {
      color: ColorTransfers,
      mono: MonoTransfers,
      inverse: InverseTransfers
    }
  },
  usertransfers: {
    variants: {
      color: ColorUserTransfers,
      mono: MonoUserTransfers,
      inverse: InverseUserTransfers
    }
  },
  wallet: {
    variants: {
      color: ColorWallet,
      mono: MonoWallet,
      inverse: InverseWallet
    }
  },
  packet: {
    variants: {
      color: ColorPacket,
      mono: MonoPacket,
      inverse: InversePacket
    }
  },
  home: {
    variants: {
      color: ColorHome,
      mono: MonoHome,
      inverse: InverseHome
    }
  },
  settings: {
    variants: {
      color: ColorSettings,
      mono: MonoSettings,
      inverse: InverseSettings
    }
  },
  alltransfers: {
    variants: {
      mono: MonoAllTransfers
    }
  },
  ibcconnections: {
    variants: {
      mono: MonoIbcConnections
    }
  },
  ibcchannels: {
    variants: {
      mono: MonoIbcChannels
    }
  },
  hubblestatus: {
    variants: {
      mono: MonoHubbleStatus
    }
  },
  searchk: {
    variants: {
      mono: MonoSearchK
    }
  }
}

let className: string | undefined = undefined
export let theme: IconVariant = "color"
export let name: IconName | undefined

export { className as class }

function getIconComponent(name: IconName | undefined, theme: IconVariant): any | undefined {
  return name && icons[name]?.variants[theme]
}

$: component = getIconComponent(name, theme)
</script>

{#if component}
  <svelte:component this={component} class={className} />
{:else}
  <p>Icon not found or not available in this variant.</p>
{/if}