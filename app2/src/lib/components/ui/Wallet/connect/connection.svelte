<script lang="ts">
import SharpPowerIcon from "$lib/components/icons/SharpPowerIcon.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { type AptosWalletId } from "$lib/wallet/aptos"
import { type CosmosWalletId } from "$lib/wallet/cosmos"
import { type EvmWalletId } from "$lib/wallet/evm"
import { RpcType } from "@unionlabs/sdk/schema"
import type { State } from "@wagmi/core"
import { Schema } from "effect"
import { Option } from "effect"
import Label from "../../Label.svelte"
import Truncate from "../../Truncate.svelte"

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
  onDisconnectClick,
  showDivider = true,
} = $props<{
  chain: Chain
  address: string | undefined
  connectStatus: ChainConnectStatus
  chainWalletsInformation: ChainWalletsInformation
  connectedWalletId:
    | (Chain extends "cosmos" ? CosmosWalletId
      : Chain extends "aptos" ? AptosWalletId
      : EvmWalletId)
    | null
    | undefined
  onConnectClick: (walletIdentifier: string) => void | Promise<any>
  onDisconnectClick: () => void
  showDivider?: boolean
}>()

let connectText = $derived(
  connectStatus === "connected" && address && address?.length > 0 ? address : "",
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
let sanitizeWalletInformation = chainWalletsInformation.filter(
  (wallet: { name: string }, index: number, array: ChainWalletsInformation) =>
    array.findIndex((t: { name: string }) =>
      t.name.toLowerCase().startsWith(wallet.name.toLowerCase())
    ) === index,
) ?? chainWalletsInformation

let walletListToRender = $derived(
  connectStatus === "connected" ? chainWalletsInformation : sanitizeWalletInformation,
)

// Find the currently connected wallet to get its icon
let connectedWallet = $derived(
  chainWalletsInformation.find((wallet: { id: string }) => wallet.id === connectedWalletId),
)
</script>

<div class="flex flex-col gap-2">
  {#if showDivider}
    <div class="flex items-center gap-4 mb-2">
      <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
      <Label
        class="text-zinc-500 dark:text-zinc-400 {connectStatus === 'connected' ? 'opacity-60' : ''}"
      >{chain}</Label>
      <div class="flex-1 h-px bg-zinc-200 dark:bg-zinc-800"></div>
    </div>
  {/if}

  {#if connectStatus === "connected" && address?.length > 0}
    <!-- Wallet Card -->
    <div class="w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4 transition-colors">
      <div class="flex items-center justify-between">
        <div class="flex gap-3 items-center">
          {#if connectedWallet?.icon}
            <img
              src={connectedWallet.icon}
              alt={connectedWallet.name}
              class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1"
            />
          {/if}
          <div class="flex flex-col">
            <h4 class="capitalize font-semibold text-base text-zinc-900 dark:text-zinc-50">
              {connectedWallet.name}
            </h4>
            <Truncate
              class="font-mono text-sm text-zinc-500 dark:text-zinc-400"
              value={connectText}
              maxLength={16}
              showCopy={true}
            />
          </div>
        </div>

        <div class="flex flex-col items-center gap-2">
          <button
            class="p-2 rounded-lg border border-zinc-800 hover:bg-zinc-800 transition-colors cursor-pointer"
            onclick={() => onDisconnectClick()}
            aria-label="Disconnect wallet"
          >
            <SharpPowerIcon class="size-3 text-zinc-400 hover:text-red-500 transition-colors" />
          </button>

          <!-- {#if Option.isSome(dashboard.user)}
            <button
              class="p-2 rounded-lg border border-zinc-800 hover:text-red-500 hover:bg-zinc-800 transition-colors cursor-pointer"
              onclick={() => onDisconnectClick()}
              aria-label="Disconnect wallet"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-3"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
              </svg>
            </button>
          {/if} -->
        </div>
      </div>
    </div>
  {/if}

  <!-- Wallet List -->
  <div class="grid grid-cols-1 gap-2">
    {#each walletListToRender as { name, id, icon, download }, index (index)}
      {@const walletIdentifier = id}
      {#if !(connectStatus === "connected" && connectedWalletId === id)}
        <button
          role="row"
          tabindex={0}
          data-index={index}
          class="
            w-full bg-zinc-100 dark:bg-zinc-900 rounded-lg p-4
            flex items-center justify-between cursor-pointer hover:bg-zinc-200 dark:hover:bg-zinc-800
            transition-colors duration-200 ease-in-out focus:outline-none
            text-base font-medium capitalize relative h-14
            {connectStatus === 'connected' ? 'opacity-60 grayscale hover:opacity-100 hover:grayscale-0' : ''}
          "
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
                class="size-8 rounded-lg bg-white dark:bg-zinc-800 p-1"
              />
            {/if}
            <span class="text-zinc-900 dark:text-zinc-50">{name}</span>
          </div>
          {#if (connectStatus === "connecting" || connectStatus === "reconnecting")
          && connectedWalletId === id}
            <span class="text-zinc-400">⋯</span>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
</div>
