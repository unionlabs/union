<script lang="ts">
import { Option } from "effect"
import { cn } from "$lib/utils"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import Input from "$lib/components/ui/Input.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import type { Token } from "$lib/schema/token.ts"
import TransferAsset from "$lib/components/Transfer/ChainAsset/TransferAsset.svelte"
import type { Chain } from "$lib/schema/chain.ts"

type Props = {
  chain: Chain
  onSelect: () => void
}

const { onSelect }: Props = $props()

let searchQuery = $state("")

const chainTokens = $derived.by(() => {
  if (Option.isNone(transfer.sortedBalances)) return []
  return transfer.sortedBalances.value.map(item => item.token)
})

// Filter the tokens based on search
const filteredTokens = $derived.by(() => {
  const query = searchQuery.toLowerCase()
  return chainTokens.filter(
    token =>
      token.denom.toLowerCase().includes(query) ||
      (token.representations[0]?.name?.toLowerCase() || "").includes(query)
  )
})

function selectAsset(token: Token) {
  transfer.raw.updateField("asset", token.denom)
  onSelect()
}
</script>

<div class="border-t border-zinc-700">
  <div class="p-4 sticky top-0 z-10 border-b border-zinc-700">
    <!-- Search Bar -->
    <Input
            type="text"
            class={cn("text-sm")}
            placeholder="Search assets..."
            disabled={!Option.isSome(transfer.sourceChain)}
            value={searchQuery}
            oninput={(e) => (searchQuery = (e.currentTarget as HTMLInputElement).value)}
    />
  </div>

  <div class="overflow-y-auto max-h-full">
    {#if Option.isNone(transfer.sourceChain)}
      <div class="flex items-center justify-center text-zinc-500 p-8">
        Please select a source chain first
      </div>
    {:else if Option.isNone(transfer.sortedBalances)}
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
    {:else if chainTokens.length === 0}
      <div class="flex items-center justify-center text-zinc-500 p-8">
        No balances found for this chain
      </div>
    {:else if filteredTokens.length === 0}
      <div class="flex items-center justify-center text-zinc-500 p-8">
        No assets found matching "{searchQuery}"
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