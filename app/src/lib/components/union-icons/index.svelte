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

import MonoInverseBlocks from "$lib/components/union-icons/mono-inverse/icon-blocks-mono-inverse.svelte"
import MonoInverseChannel from "$lib/components/union-icons/mono-inverse/icon-channel-mono-inverse.svelte"
import MonoInverseConnection from "$lib/components/union-icons/mono-inverse/icon-connection-mono-inverse.svelte"
import MonoInverseExplorer from "$lib/components/union-icons/mono-inverse/icon-explorer-mono-inverse.svelte"
import MonoInverseFaucet from "$lib/components/union-icons/mono-inverse/icon-faucet-mono-inverse.svelte"
import MonoInverseIndex from "$lib/components/union-icons/mono-inverse/icon-index-mono-inverse.svelte"
import MonoInverseQueue from "$lib/components/union-icons/mono-inverse/icon-queue-mono-inverse.svelte"
import MonoInverseSearch from "$lib/components/union-icons/mono-inverse/icon-search-mono-inverse.svelte"
import MonoInverseTransfers from "$lib/components/union-icons/mono-inverse/icon-transfers-mono-inverse.svelte"
import MonoInverseUserTransfers from "$lib/components/union-icons/mono-inverse/icon-usertransfers-mono-inverse.svelte"
import MonoInverseWallet from "$lib/components/union-icons/mono-inverse/icon-wallet-mono-inverse.svelte"
import MonoInversePacket from "$lib/components/union-icons/mono-inverse/icon-packet-mono-inverse.svelte"
import MonoInverseHome from "$lib/components/union-icons/mono-inverse/icon-home-mono-inverse.svelte"
import MonoInverseSettings from "$lib/components/union-icons/mono-inverse/icon-packet-mono-inverse.svelte"
import MonoInverseAllTransfers from "$lib/components/union-icons/mono-inverse/icon-alltransfers-mono-inverse.svelte"
import MonoInverseIbcConnections from "$lib/components/union-icons/mono-inverse/icon-ibcconnections-mono-inverse.svelte"
import MonoInverseIbcChannels from "$lib/components/union-icons/mono-inverse/icon-ibcchannels-mono-inverse.svelte"
import MonoInverseHubbleStatus from "$lib/components/union-icons/mono-inverse/icon-hubblestatus-mono-inverse.svelte"
import MonoInverseSearchK from "$lib/components/union-icons/mono-inverse/icon-searchk-mono-inverse.svelte"

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

type IconVariant = "color" | "mono" | "inverse" | "monoInverse"

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
      inverse: InverseBlocks,
      monoInverse: MonoInverseBlocks
    }
  },
  channel: {
    variants: {
      color: ColorChannel,
      mono: MonoChannel,
      inverse: InverseChannel,
      monoInverse: MonoInverseChannel
    }
  },
  connection: {
    variants: {
      color: ColorConnection,
      mono: MonoConnection,
      inverse: InverseConnection,
      monoInverse: MonoInverseConnection
    }
  },
  explorer: {
    variants: {
      color: ColorExplorer,
      mono: MonoExplorer,
      inverse: InverseExplorer,
      monoInverse: MonoInverseExplorer
    }
  },
  faucet: {
    variants: {
      color: ColorFaucet,
      mono: MonoFaucet,
      inverse: InverseFaucet,
      monoInverse: MonoInverseFaucet
    }
  },
  index: {
    variants: {
      color: ColorIndex,
      mono: MonoIndex,
      inverse: InverseIndex,
      monoInverse: MonoInverseIndex
    }
  },
  queue: {
    variants: {
      color: ColorQueue,
      mono: MonoQueue,
      inverse: InverseQueue,
      monoInverse: MonoInverseQueue
    }
  },
  search: {
    variants: {
      color: ColorSearch,
      mono: MonoSearch,
      inverse: InverseSearch,
      monoInverse: MonoInverseSearch
    }
  },
  transfers: {
    variants: {
      color: ColorTransfers,
      mono: MonoTransfers,
      inverse: InverseTransfers,
      monoInverse: MonoInverseTransfers
    }
  },
  usertransfers: {
    variants: {
      color: ColorUserTransfers,
      mono: MonoUserTransfers,
      inverse: InverseUserTransfers,
      monoInverse: MonoInverseUserTransfers
    }
  },
  wallet: {
    variants: {
      color: ColorWallet,
      mono: MonoWallet,
      inverse: InverseWallet,
      monoInverse: MonoInverseWallet
    }
  },
  packet: {
    variants: {
      color: ColorPacket,
      mono: MonoPacket,
      inverse: InversePacket,
      monoInverse: MonoInversePacket
    }
  },
  home: {
    variants: {
      color: ColorHome,
      mono: MonoHome,
      inverse: InverseHome,
      monoInverse: MonoInverseHome
    }
  },
  settings: {
    variants: {
      color: ColorSettings,
      mono: MonoSettings,
      inverse: InverseSettings,
      monoInverse: MonoInverseSettings
    }
  },
  alltransfers: {
    variants: {
      mono: MonoAllTransfers,
      monoInverse: MonoInverseAllTransfers
    }
  },
  ibcconnections: {
    variants: {
      mono: MonoIbcConnections,
      monoInverse: MonoInverseIbcConnections
    }
  },
  ibcchannels: {
    variants: {
      mono: MonoIbcChannels,
      monoInverse: MonoInverseIbcChannels
    }
  },
  hubblestatus: {
    variants: {
      mono: MonoHubbleStatus,
      monoInverse: MonoInverseHubbleStatus
    }
  },
  searchk: {
    variants: {
      mono: MonoSearchK,
      monoInverse: MonoInverseSearchK
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