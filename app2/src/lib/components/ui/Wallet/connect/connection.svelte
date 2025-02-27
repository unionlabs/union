<script lang="ts">
import { truncateEvmAddress, truncateUnionAddress } from "$lib/wallet/utilities/format.ts"
import type { State } from "@wagmi/core"
import { type CosmosWalletId } from "$lib/wallet/cosmos"
import { type AptosWalletId } from "$lib/wallet/aptos"
import { type EvmWalletId } from "$lib/wallet/evm"
import { Schema } from "effect"
import { RpcType } from "$lib/schema/chain.ts"

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
      ? truncateEvmAddress(address, -1)
      : chain === "aptos"
        ? address
        : truncateUnionAddress(address, -1)
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
  <div class="w-full bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 mb-4">
    <div class="flex items-center justify-between mb-2">
      <div class="flex items-center">
        <div class="bg-neutral-700 rounded-lg p-2 mr-3">
          {#if connectedWallet?.icon}
            <img src={connectedWallet.icon} alt={connectedWallet.name} class="size-7" />
          {:else}
            <div class="size-7 bg-neutral-600 rounded-full"></div>
          {/if}
        </div>
        <div>
          <h4 class="capitalize font-bold text-lg">{connectedWallet.name}</h4>
        </div>
      </div>
      <button
              class="text-neutral-400 hover:text-white focus:outline-none"
              onclick={() => onDisconnectClick()}
      >
        <span>❌</span>
      </button>
    </div>
    <div class="bg-neutral-700 rounded p-3 flex justify-between items-center">
      <span class="font-mono text-sm truncate">{connectText}</span>
      <button
              class="p-1 hover:bg-neutral-600 rounded focus:outline-none"
              onclick={() => onCopyClick()}
      >
        {#if copyClicked}
          <p>✅</p>
        {:else}
          <p>📝</p>
        {/if}
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
                class="w-full bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg p-3
                      flex items-center justify-between cursor-pointer hover:bg-neutral-100 dark:hover:bg-neutral-700
                      transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-neutral-300"
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