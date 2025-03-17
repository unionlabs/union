<script lang="ts">
import { Option } from "effect"
import { cn } from "$lib/utils"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import Input from "$lib/components/ui/Input.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import type { Token } from "$lib/schema/token.ts"

type Props = {
  onSelect: () => void
}

const { onSelect }: Props = $props()

let searchQuery = $state("")

const filteredTokens = $derived.by(() => {
  const query = searchQuery.toLowerCase()
  return Option.getOrElse(transfer.baseTokens, () => []).filter(
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

  <div class="overflow-y-auto max-h-64">
    {#if Option.isNone(transfer.sourceChain)}
      <div class="flex items-center justify-center text-zinc-500 p-8">
        Please select a source chain first
      </div>
    {:else}
      {@const tokenData = tokensStore.getData(transfer.sourceChain.value.universal_chain_id)}
      {@const error = tokensStore.getError(transfer.sourceChain.value.universal_chain_id)}

      {#if Option.isSome(error)}
        <div class="flex items-center justify-center text-red-500 p-8">
          Error: {error.value.message}
        </div>
      {:else if Option.isNone(tokenData)}
        <div>
          {#each Array(5) as _, i}
            <div class="flex items-center w-full px-4 py-2 border-b border-zinc-700">
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
          No assets found
        </div>
      {:else}
        <div>
          {#each filteredTokens as token}
            <button
                    class={cn(
                "flex items-center w-full px-4 py-2 text-left hover:bg-zinc-700 transition-colors border-b border-zinc-700",
                transfer.raw.asset === token.denom ? "bg-zinc-700 text-white" : "text-zinc-300"
              )}
                    onclick={() => selectAsset(token)}
            >
              <div class="flex-1 min-w-0">
                <div class="font-medium text-sm truncate">
                  {token.representations[0]?.name ?? token.denom}
                </div>
                {#if token.representations[0]?.name}
                  <div class="text-xs text-zinc-400 truncate">
                    {token.denom}
                  </div>
                {/if}
              </div>
              <div class="ml-2 text-zinc-400">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                     stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="9 18 15 12 9 6"></polyline>
                </svg>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>