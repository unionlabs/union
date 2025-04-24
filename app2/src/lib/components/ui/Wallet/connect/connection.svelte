<script lang="ts">
import type { State } from "@wagmi/core"
import { type CosmosWalletId } from "$lib/wallet/cosmos"
import { type AptosWalletId } from "$lib/wallet/aptos"
import { type EvmWalletId } from "$lib/wallet/evm"
import { Schema } from "effect"
import { RpcType } from "@unionlabs/sdk/schema"
import Truncate from "../../Truncate.svelte"
import Label from "../../Label.svelte"
import SharpPowerIcon from "$lib/components/icons/SharpPowerIcon.svelte"

type Chain = Schema.Schema.Type<typeof RpcType>
type ChainConnectStatus = State["status"]
type ChainWalletsInformation = ReadonlyArray<{
  id: string
  name: string
  icon: string
  download: string
}>

let {
  chain,
  address,
  connectStatus,
  chainWalletsInformation,
  connectedWalletId,
  onConnectClick,
  onDisconnectClick
} = $props<{
  chain: Chain
  address: string | undefined
  connectStatus: ChainConnectStatus
  chainWalletsInformation: ChainWalletsInformation
  connectedWalletId:
    | (Chain extends "cosmos"
        ? CosmosWalletId
        : Chain extends "aptos"
          ? AptosWalletId
          : EvmWalletId)
    | null
    | undefined
  onConnectClick: (walletIdentifier: string) => void | Promise<void>
  onDisconnectClick: () => void
}>()

let connectText = $derived(
  connectStatus === "connected" && address && address?.length > 0 ? address : ""
)

let copyClicked = $state(false)
const toggleCopy = () => (copyClicked = !copyClicked)
const onCopyClick = () => {
  if (address) {
    navigator.clipboard.writeText(address)
    toggleCopy()
    setTimeout(() => toggleCopy(), 1_500)
  }
}

// filter items with duplicate names
let sanitizeWalletInformation =
  chainWalletsInformation.filter(
    (wallet: { name: string }, index: number, array: ChainWalletsInformation) =>
      array.findIndex((t: { name: string }) =>
        t.name.toLowerCase().startsWith(wallet.name.toLowerCase())
      ) === index
  ) ?? chainWalletsInformation

let walletListToRender = $derived(
  connectStatus === "connected" ? chainWalletsInformation : sanitizeWalletInformation
)

// Find the currently connected wallet to get its icon
let connectedWallet = $derived(
  chainWalletsInformation.find((wallet: { id: string }) => wallet.id === connectedWalletId)
)
</script>

<div class="flex flex-col mb-8">
  <div class="flex flex-col gap-2">

    <div class="flex items-center gap-4 mb-2">
      <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
      <Label class="text-zinc-500 dark:text-zinc-400">{chain}</Label>
      <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
    </div>

    {#if connectStatus === "connected" && address?.length > 0}
      <!-- Wallet Card -->
      <div class="w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4">
        <div class="flex items-center justify-between">
          <div class="flex gap-2 flex-col">
            <div class="flex items-center gap-3">
              {#if connectedWallet?.icon}
                <img src={connectedWallet.icon} alt={connectedWallet.name} class="size-5" />
              {/if}
              <h4 class="capitalize font-bold text-base">{connectedWallet.name}</h4>
            </div>
            <Truncate class="font-mono text-zinc-400" value={connectText} maxLength={16} showCopy={true}/>
          </div>
          <div class="flex flex-col gap-1.5">
            <button
              class="group p-2 rounded border border-zinc-200 dark:border-zinc-700 hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors cursor-pointer"
              onclick={() => onDisconnectClick()}
              aria-label="Disconnect wallet"
            >
              <SharpPowerIcon class="size-4 text-zinc-400 group-hover:text-red-500 transition-colors" />
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Wallet List -->
    <div class="flex flex-col gap-2">
      {#each walletListToRender as {name, id, icon, download}, index (index)}
        {@const walletIdentifier = id}
        {#if !(connectStatus === "connected" && connectedWalletId === id)}
          <button
            role="row"
            tabindex={0}
            data-index={index}
            class="w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4
                  flex items-center justify-between cursor-pointer hover:bg-zinc-200 dark:hover:bg-zinc-800
                  transition-colors duration-200 ease-in-out focus:outline-none
                  text-base font-medium capitalize relative h-12"
            onclick={async () => {
              if (connectStatus === "disconnected") {
                console.info("disconnected, calling onConnectClick")
                return onConnectClick(walletIdentifier)
              }
              console.info("connected, calling onDisconnectClick")
              return onDisconnectClick()
            }}
            aria-label={`${connectStatus === "disconnected" ? "Connect" : "Disconnect"} ${name} wallet`}
          >
            <div class="flex items-center gap-3">
              {#if icon}
                <img 
                  src={icon} 
                  alt={name} 
                  class="size-5 {connectStatus === 'connected' ? 'grayscale opacity-50' : ''}" 
                />
              {/if}
              <span>{name}</span>
            </div>
            {#if (connectStatus === "connecting" || connectStatus === "reconnecting") && connectedWalletId === id}
              <span class="text-zinc-400">⋯</span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>
  </div>
</div>
