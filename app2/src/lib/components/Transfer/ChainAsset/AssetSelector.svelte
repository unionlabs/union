<script lang="ts">
  import { Option } from "effect"
  import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
  import { wallets } from "$lib/stores/wallets.svelte.ts"
  import Input from "$lib/components/ui/Input.svelte"
  import Skeleton from "$lib/components/ui/Skeleton.svelte"
  import TransferAsset from "$lib/components/Transfer/ChainAsset/TransferAsset.svelte"
  import { Token } from "@unionlabs/sdk/schema"
  import {fade, fly, scale} from "svelte/transition"

  type Props = {
    onSelect: () => void
  }

  const { onSelect }: Props = $props()

  let searchQuery = $state("")
  let searchOpen = $state(false)

  const isWalletConnected = $derived.by(() => {
    if (Option.isNone(transfer.sourceChain)) return false
    const addressOption = wallets.getAddressForChain(transfer.sourceChain.value)
    return Option.isSome(addressOption)
  })

  const filteredTokens = $derived.by(() => {
    // If we don't have base tokens yet, return empty array
    if (Option.isNone(transfer.baseTokens)) return [] as Array<Token>

    let tokensToShow: Array<Token>

    // If wallet is connected, and we have sorted balances, use the sorted order
    if (isWalletConnected && Option.isSome(transfer.sortedBalances)) {
      // Extract tokens in order from sortedBalances
      const sortedDenoms = transfer.sortedBalances.value.map(item => item.token.denom)

      // Get full token details from baseTokens
      const baseTokens = transfer.baseTokens.value

      // Create token lookup map for efficient access
      const tokenMap = new Map(baseTokens.map(token => [token.denom, token]))

      // Create sorted token array with full details
      tokensToShow = sortedDenoms
        .map(denom => tokenMap.get(denom))
        .filter((token): token is Token => !!token)
    } else {
      // No wallet connected or no sorted balances, just use base tokens
      tokensToShow = [...transfer.baseTokens.value]
    }

    // If no search query, return all tokens
    if (!searchQuery) return tokensToShow

    // Filter by search query
    const query = searchQuery.toLowerCase()
    return tokensToShow.filter(
      token =>
        token.denom.toLowerCase().includes(query) ||
        (token.representations[0]?.name?.toLowerCase() || "").includes(query) ||
        (token.representations[0]?.symbol?.toLowerCase() || "").includes(query)
    )
  })

  function selectAsset(token: Token) {
    transfer.raw.updateField("asset", token.denom)
    onSelect()
  }

  function toggleSearch() {
    searchOpen = !searchOpen
    if (!searchOpen) {
      searchQuery = ""
    }
  }
</script>

<div class="h-full flex flex-col relative">
  <!-- Main content with scrolling -->
  <div class="overflow-y-auto flex-grow" in:fade={{ duration: 300 }}>
    <div class="w-full">
      {#if Option.isNone(transfer.sourceChain)}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          Please select a source chain first
        </div>
      {:else if Option.isNone(transfer.baseTokens)}
        <div>
          {#each Array(5) as _, i}
            <div class="flex items-center w-full px-4 py-2 border-b border-zinc-700">
              <div class="flex-1 min-w-0">
                <div class="mb-1">
                  <Skeleton class="h-4 w-24" randomWidth={true}/>
                </div>
                <Skeleton class="h-3 w-32" randomWidth={true}/>
              </div>
              <div class="ml-2">
                <Skeleton class="h-4 w-4"/>
              </div>
            </div>
          {/each}
        </div>
      {:else if filteredTokens.length === 0}
        <div class="flex items-center justify-center text-zinc-500 p-8">
          {searchQuery ? `No assets found matching "${searchQuery}"` : "No tokens found for this chain"}
        </div>
      {:else}
        <div class="flex flex-col gap-1 p-2 pb-16">
          {#each filteredTokens as token}
            {#key token.denom}
              <TransferAsset {token} {selectAsset} />
            {/key}
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Search button in bottom right -->
  <div class="absolute bottom-0 inset-x-0 z-10 flex justify-end w-full p-4">
    {#if searchOpen}
      <div class="flex  bg-zinc-800 rounded-md shadow-lg overflow-hidden"
           transition:scale={{ duration: 200, start: 0.8 }}>
        <Input
                type="text"
                placeholder="Search assets..."
                disabled={!Option.isSome(transfer.sourceChain)}
                value={searchQuery}
                class="bg-transparent border-0 pl-4 pr-2 py-2 w-48 focus:ring-0 w-full"
                on:input={(e) => (searchQuery = (e.currentTarget as HTMLInputElement).value)}
        />
        <button
                class="flex items-center justify-center h-10 w-10 bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                onclick={toggleSearch}
                aria-label="Close search"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
    {:else}
      <button
              class="flex items-center justify-center h-10 w-10 rounded-md bg-zinc-800 text-zinc-300 hover:bg-zinc-700 shadow-lg"
              onclick={toggleSearch}
              aria-label="Search assets"
              disabled={!Option.isSome(transfer.sourceChain)}
              transition:scale={{ duration: 200 }}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
      </button>
    {/if}
  </div>
</div>