<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import type { Token } from "$lib/schema/token.ts"
import { Option } from "effect"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { formatUnits } from "viem"

type Props = {
  token: Token
  selectAsset: (token: Token) => void
}

let { token, selectAsset }: Props = $props()

let isSelected = $derived(transfer.raw.asset === token.denom)

let tokenBalance = $derived.by(() => {
  if (Option.isNone(transfer.sortedBalances)) return Option.none()
  const found = transfer.sortedBalances.value.find(t => t.token.denom === token.denom)
  return found ? Option.some(found) : Option.none()
})

let displayAmount = $derived.by(() => {
  if (Option.isNone(tokenBalance)) return "0.00"

  const balanceInfo = tokenBalance.value

  if (Option.isNone(balanceInfo.balance)) return "0.00"

  const decimals = balanceInfo.decimals || token.representations[0]?.decimals || 0

  const balanceValue = Option.getOrElse(balanceInfo.balance, () => "0")
  return formatUnits(BigInt(balanceValue), decimals)
})

let isLoading = $derived(Option.isSome(transfer.sortedBalances) && Option.isNone(tokenBalance))
</script>

<button
        class={cn(
                "flex items-center w-full px-4 py-2 text-left hover:bg-zinc-700 transition-colors border-b border-zinc-700 cursor-pointer",
                isSelected ? "bg-zinc-700 text-white" : "text-zinc-300"
              )}
        onclick={() => selectAsset(token)}
>
  <div class="flex-1 min-w-0">
    <div class="font-medium text-sm truncate">
      {token.representations[0]?.name ?? token.denom}
    </div>
    {#if token.representations[0]?.name}
      <div class="text-xs text-zinc-400 truncate w-24 truncate">
        {token.denom}
      </div>
    {/if}
  </div>
  <div class="ml-2 text-right flex items-center">
    <div class="text-xs text-zinc-400 mr-2">
      {#if isLoading}
        <Skeleton class="h-3 w-16"/>
      {:else if Option.isSome(tokenBalance) && Option.isSome(tokenBalance.value.error)}
        <span class="text-red-400">Error</span>
      {:else}
        {displayAmount}
      {/if}
    </div>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
         stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="9 18 15 12 9 6"></polyline>
    </svg>
  </div>
</button>