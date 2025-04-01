<script lang="ts">
import { Option } from "effect"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import Input from "$lib/components/ui/Input.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import TransferAsset from "$lib/components/Transfer/ChainAsset/TransferAsset.svelte"
import { Token } from "@unionlabs/sdk/schema"

type Props = {
  onSelect: () => void
}

const { onSelect }: Props = $props()

let searchQuery = $state("")

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
</script>

<div class="p-4 border-y border-zinc-700">
  <!-- Search Bar -->
  <Input
          type="text"
          placeholder="Search assets..."
          disabled={!Option.isSome(transfer.sourceChain)}
          value={searchQuery}
          oninput={(e) => (searchQuery = (e.currentTarget as HTMLInputElement).value)}
  />
</div>

<div class="overflow-y-scroll mb-12">
  <div class="w-full h-full">
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
        <div>
          {#each filteredTokens as token}
            {#key token.denom}
              <TransferAsset {token} {selectAsset} />
            {/key}
          {/each}
        </div>
      {/if}
  </div>
</div>