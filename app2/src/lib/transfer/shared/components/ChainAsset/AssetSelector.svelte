<script lang="ts">
import { Option } from "effect"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import TransferAsset from "$lib/transfer/shared/components/ChainAsset/TransferAsset.svelte"
import { Token } from "@unionlabs/sdk/schema"
import { fade } from "svelte/transition"
import { tick } from "svelte"

type Props = {
  onSelect: () => void
}

const { onSelect }: Props = $props()

let searchQuery = $state("")
let searchOpen = $state(false)
let searchInput: HTMLInputElement | null = null

const isWalletConnected = $derived.by(() => {
  if (Option.isNone(transferData.sourceChain)) return false
  const addressOption = wallets.getAddressForChain(transferData.sourceChain.value)
  return Option.isSome(addressOption)
})

const filteredTokens = $derived.by(() => {
  if (Option.isNone(transferData.baseTokens)) return [] as Array<Token>

  let tokensToShow: Array<Token>

  if (isWalletConnected && Option.isSome(transferData.sortedBalances)) {
    const sortedDenoms = transferData.sortedBalances.value.map(item => item.token.denom)
    const baseTokens = transferData.baseTokens.value
    const tokenMap = new Map(baseTokens.map(token => [token.denom, token]))
    tokensToShow = sortedDenoms
      .map(denom => tokenMap.get(denom))
      .filter((token): token is Token => !!token)
  } else {
    tokensToShow = [...transferData.baseTokens.value]
  }

  if (!searchQuery) return tokensToShow

  const query = searchQuery.toLowerCase()
  return tokensToShow.filter(
    token =>
      token.denom.toLowerCase().includes(query) ||
      (token.representations[0]?.name?.toLowerCase() || "").includes(query) ||
      (token.representations[0]?.symbol?.toLowerCase() || "").includes(query)
  )
})

function selectAsset(token: Token) {
  transferData.raw.updateField("asset", token.denom)
  onSelect()
}

async function toggleSearch() {
  searchOpen = !searchOpen

  if (searchOpen) {
    // Wait for the input to be visible in the DOM
    await tick()
    // Focus the input once it opens
    if (searchInput) {
      searchInput.focus()
    }
  } else {
    searchQuery = ""
  }
}
</script>

<div datatestid="asset-selector" class="h-full flex flex-col relative">
  <div class="overflow-y-auto flex-grow" in:fade={{ duration: 300 }}>
    <div class="w-full">
      {#if Option.isNone(transferData.sourceChain)}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          Please select a source chain first
        </div>
      {:else if Option.isNone(transferData.baseTokens)}
        <div>
          {#each Array(5) as _, i}
            <div
              class="flex items-center w-full px-4 py-2 border-b border-zinc-700"
            >
              <div class="flex-1 min-w-0">
                <div class="mb-1">
                  <Skeleton class="h-4 w-24" randomWidth={true} />
                </div>
                <Skeleton class="h-3 w-32" randomWidth={true} />
              </div>
              <div class="ml-2">
                <Skeleton class="h-4 w-4" />
              </div>
            </div>
          {/each}
        </div>
      {:else if filteredTokens.length === 0}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          {searchQuery
            ? `No assets found matching "${searchQuery}"`
            : "No tokens found for this chain"}
        </div>
      {:else}
        <div class="flex flex-col gap-1 p-2 pb-16">
          {#each filteredTokens as token}
            {#key token.denom}
              <TransferAsset
                chain={transferData.sourceChain.value}
                {token}
                {selectAsset}
              />
            {/key}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div
    class="absolute bottom-0 inset-x-0 z-0 {searchOpen
      ? 'h-20'
      : 'h-0'} transition-all bg-gradient-to-t from-zinc-925 to-transparent blur-fade-bottom-up"
  ></div>
  <div class="absolute bottom-0 inset-x-0 z-10 flex justify-end w-full p-4">
    <div
      class="flex items-center bg-zinc-800 rounded-md shadow-lg overflow-hidden transition-all duration-300 ease-in-out {searchOpen
        ? 'w-full'
        : 'w-10'}"
    >
      {#if searchOpen}
        <div
          class="flex-grow flex items-center transition-all duration-300 ease-in-out"
        >
          <input
            bind:this={searchInput}
            type="text"
            placeholder="Search assets..."
            disabled={!Option.isSome(transferData.sourceChain)}
            value={searchQuery}
            class="bg-transparent border-0 pl-4 pr-2 py-2 h-10 focus:ring-0 w-full text-zinc-100 focus:outline-none focus:ring-0 focus:none"
            oninput={(e) =>
              (searchQuery = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>
      {/if}
      <button
        class="flex items-center justify-center h-10 w-10 {searchOpen
          ? 'bg-zinc-700 hover:bg-zinc-600'
          : 'bg-zinc-800 hover:bg-zinc-700'} text-zinc-300 flex-shrink-0 transition-colors duration-300 cursor-pointer"
        onclick={toggleSearch}
        aria-label={searchOpen ? "Close search" : "Search assets"}
        disabled={!Option.isSome(transferData.sourceChain)}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          {#if searchOpen}
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          {:else}
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          {/if}
        </svg>
      </button>
    </div>
  </div>
</div>
