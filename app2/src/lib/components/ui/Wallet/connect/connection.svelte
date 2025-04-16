<script lang="ts">
import type { State } from "@wagmi/core"
import { type CosmosWalletId } from "$lib/wallet/cosmos"
import { type AptosWalletId } from "$lib/wallet/aptos"
import { type EvmWalletId } from "$lib/wallet/evm"
import { Schema } from "effect"
import { RpcType } from "@unionlabs/sdk/schema"
import BaselineCloseIcon from "$lib/components/icons/BaselineCloseIcon.svelte"
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
  connectStatus === "connected" && address && address?.length > 0
    ? chain === "evm"
      ? address
      : chain === "aptos"
        ? address
        : address
    : ""
)

let copyClicked = $state(false)
const toggleCopy = () => (copyClicked = !copyClicked)
const onCopyClick = () => [toggleCopy(), setTimeout(() => toggleCopy(), 1_500)]

// filter items with duplicate names
let sanitizeWalletInformation =
  chainWalletsInformation.filter(
    (predicate, index, array) =>
      array.findIndex(t => t.name.toLowerCase().startsWith(predicate.name.toLowerCase())) === index
  ) ?? chainWalletsInformation

let walletListToRender = $derived(
  connectStatus === "connected" ? chainWalletsInformation : sanitizeWalletInformation
)

// Find the currently connected wallet to get its icon
let connectedWallet = $derived(
  chainWalletsInformation.find(wallet => wallet.id === connectedWalletId)
)
</script>

<h3 class="capitalize font-supermolot font-bold text-xl mb-4">{chain}</h3>

{#if connectStatus === "connected" && address?.length > 0}
  <!-- Wallet Card -->
  <div class="w-full bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg p-4 mb-4">
    <div class="flex items-center justify-between">
      <div class="flex gap-2 flex-col">
        <div class="flex items-center gap-4">
          <div>
            {#if connectedWallet?.icon}
              <img src={connectedWallet.icon} alt={connectedWallet.name} class="size-7" />
            {:else}
              <div class="size-7 bg-zinc-600 rounded-full"></div>
            {/if}
          </div>
          <div>
            <h4 class="capitalize font-bold text-lg">{connectedWallet.name}</h4>
          </div>
        </div>
        <Truncate class="font-mono text-zinc-400" value={connectText} maxLength={16} showCopy={false}/>
      </div>
      <button
              class="text-zinc-400 hover:text-white focus:outline-none"
              onclick={() => onDisconnectClick()}
      >
        <BaselineCloseIcon class="size-8 text-red-500"/>
      </button>
    </div>
  </div>
{:else}
  <!-- Wallet List -->
  <div class="flex flex-col gap-3">
    {#each walletListToRender as {name, id, icon, download}, index (index)}
      {@const walletIdentifier = id}
      {#if !(connectStatus === "connected" && connectedWalletId !== id)}
        <button
                role="row"
                tabindex={0}
                data-index={index}
                class="w-full bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg p-3
                      flex items-center justify-between cursor-pointer hover:bg-zinc-100 dark:hover:bg-zinc-700
                      transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-zinc-300"
                onclick={async () => {
            if (connectStatus === "disconnected") {
              console.info("disconnected, calling onConnectClick")
              return onConnectClick(walletIdentifier)
            }
            console.info("connected, calling onDisconnectClick")
            return onDisconnectClick()
          }}
        >
          <div class="flex items-center">
            <span class="text-base font-medium capitalize">{name}</span>
          </div>
          <div class="flex items-center">
            {#if (connectStatus === "connecting" || connectStatus === "reconnecting") && connectedWalletId === id}
              <p class="ml-2">🕙</p>
            {/if}
            <img src={icon} alt={name} class="size-6 dark:text-white"/>
          </div>
        </button>
      {/if}
    {/each}
  </div>
{/if}
