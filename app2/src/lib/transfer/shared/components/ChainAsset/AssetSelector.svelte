<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import TransferAsset from "$lib/transfer/shared/components/ChainAsset/TransferAsset.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { Token } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { tick } from "svelte"

type Props = {
  onSelect: () => void
}

const { onSelect }: Props = $props()

let searchQuery = $state("")
let searchOpen = $state(false)
let searchInput: HTMLInputElement | null = null
let topFadeOpacity = $state(0)
let bottomFadeOpacity = $state(1)
let searchOpacity = $state(1)

function handleScroll(e: Event) {
  const target = e.target as HTMLElement
  // Gradually fade in over first 30px of scroll
  topFadeOpacity = Math.min(target.scrollTop / 100, 1)

  // Gradually fade out bottom fade when near bottom
  const scrollFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight
  bottomFadeOpacity = Math.min(scrollFromBottom / 100, 1)

  // Fade out search when near bottom (last 80px), but keep visible if search is open
  searchOpacity = searchOpen ? 1 : Math.min(scrollFromBottom / 50, 1)
}

const isWalletConnected = $derived.by(() => {
  if (Option.isNone(transferData.sourceChain)) {
    return false
  }
  const addressOption = wallets.getAddressForChain(transferData.sourceChain.value)
  return Option.isSome(addressOption)
})

const filteredTokens = $derived.by(() => {
  if (Option.isNone(transferData.baseTokens)) {
    return [] as Array<Token>
  }

  let tokensToShow: Array<Token>

  if (isWalletConnected && Option.isSome(transferData.sortedBalances)) {
    const sortedDenoms = transferData.sortedBalances.value.map(item => item.token.denom)
    const baseTokens = transferData.baseTokens.value
    const tokenMap = new Map(baseTokens.map(token => [token.denom, token]))

    tokensToShow = sortedDenoms
      .map(denom => tokenMap.get(denom))
      .filter((token): token is Token => {
        if (!token) {
          return false
        }
        // Only check whitelist on mainnet
        if (Option.isSome(transferData.sourceChain) && transferData.sourceChain.value.testnet) {
          return true
        }
        return token.whitelisted === true
      })
  } else {
    tokensToShow = transferData.baseTokens.value.filter(token => {
      // Only check whitelist on mainnet
      if (Option.isSome(transferData.sourceChain) && transferData.sourceChain.value.testnet) {
        return true
      }
      return token.whitelisted === true
    })
  }

  if (!searchQuery) {
    return tokensToShow
  }

  const query = searchQuery.toLowerCase()
  return tokensToShow.filter(
    token =>
      token.denom.toLowerCase().includes(query)
      || (token.representations[0]?.name?.toLowerCase() || "").includes(query)
      || (token.representations[0]?.symbol?.toLowerCase() || "").includes(query),
  )
})

function selectAsset(token: Token) {
  console.log("[AssetSelector] selectAsset", token)
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

<div class="h-full flex flex-col relative">
  <!-- Top gradient fade -->
  {#if topFadeOpacity > 0}
    <div
      class="absolute top-0 left-0 right-0 h-12 bg-gradient-to-b from-zinc-925 to-transparent pointer-events-none z-10"
      style="opacity: {topFadeOpacity}"
    >
    </div>
  {/if}

  <div
    class="overflow-y-auto flex-grow"
    onscroll={handleScroll}
  >
    <div class="w-full">
      {#if Option.isNone(transferData.sourceChain)}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          Please select a source chain first
        </div>
      {:else if Option.isNone(transferData.baseTokens)}
        <div>
          {#each Array(5) as _}
            <div class="flex items-center w-full px-4 py-2 border-b border-zinc-700">
              <div class="flex-1 min-w-0">
                <div class="mb-1">
                  <Skeleton
                    class="h-4 w-24"
                    randomWidth={true}
                  />
                </div>
                <Skeleton
                  class="h-3 w-32"
                  randomWidth={true}
                />
              </div>
              <div class="ml-2">
                <Skeleton class="h-4 w-4" />
              </div>
            </div>
          {/each}
        </div>
      {:else if filteredTokens.length === 0}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          {
            searchQuery
            ? `No assets found matching "${searchQuery}"`
            : "No tokens found for this chain"
          }
        </div>
      {:else}
        <div class="flex flex-col gap-1 p-2">
          {#each filteredTokens as token, index}
            {#key token.denom}
              <TransferAsset
                chain={transferData.sourceChain.value}
                {token}
                {selectAsset}
                {index}
              />
            {/key}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  {#if bottomFadeOpacity > 0}
    <div
      class="absolute bottom-0 inset-x-0 h-20 bg-gradient-to-t from-zinc-925 to-transparent pointer-events-none"
      style="opacity: {bottomFadeOpacity}"
    >
    </div>
  {/if}
  <div
    class="absolute bottom-0 inset-x-0 z-10 flex justify-end w-full p-4 transition-opacity duration-150 pointer-events-none"
    style="opacity: {searchOpacity}"
  >
    <div
      class="
        flex items-center rounded overflow-hidden transition-all duration-200 ease-in-out border pointer-events-auto {searchOpen
        ? 'w-full bg-zinc-900 border-accent'
        : 'w-10 bg-zinc-900 border-zinc-800 hover:border-zinc-600'}
      "
    >
      {#if searchOpen}
        <div class="flex-grow flex items-center transition-all duration-300 ease-in-out">
          <input
            bind:this={searchInput}
            type="text"
            placeholder="Search assets..."
            disabled={!Option.isSome(transferData.sourceChain)}
            value={searchQuery}
            class="bg-transparent border-0 pl-4 pr-2 py-2 h-10 focus:ring-0 w-full text-zinc-100 placeholder-zinc-400 focus:outline-none"
            oninput={(e) => (searchQuery = (e.currentTarget as HTMLInputElement).value)}
          />
        </div>
      {/if}
      <button
        class="flex items-center justify-center h-10 w-10 text-zinc-400 hover:text-zinc-200 flex-shrink-0 transition-all duration-100 cursor-pointer hover:bg-zinc-800 rounded"
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
            <line
              x1="18"
              y1="6"
              x2="6"
              y2="18"
            >
            </line>
            <line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
            >
            </line>
          {:else}
            <circle
              cx="11"
              cy="11"
              r="8"
            >
            </circle>
            <line
              x1="21"
              y1="21"
              x2="16.65"
              y2="16.65"
            >
            </line>
          {/if}
        </svg>
      </button>
    </div>
  </div>
</div>
