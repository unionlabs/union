<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { formatUnits } from "viem"
import { chains } from "$lib/stores/chains.svelte.ts"
import SharpArrowLeft from "$lib/components/icons/SharpArrowLeft.svelte"

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

export const toDisplayName = (
  chain_id: string | undefined | null,
  chains: ReadonlyArray<Chain>
): string => chains.find(c => c.chain_id === chain_id)?.display_name ?? chain_id ?? "unknown chain"
</script>

<button
        class={cn(
                "flex flex-col items-start w-full overflow-x-scroll px-4 py-3 text-left bg-zinc-900 hover:bg-zinc-800 transition-colors cursor-pointer rounded",
                isSelected ? "bg-zinc-700 text-white" : "text-zinc-300"
              )}
        onclick={() => {
          console.log(token)
         selectAsset(token)
        }}
>
    <div class="flex items-center flex gap-1 items-center overflow-x-scroll text-sm text-zinc-200">
      <div class="mr-1">
        {#if isLoading}
          <Skeleton class="h-3 w-16"/>
        {:else if Option.isSome(tokenBalance) && Option.isSome(tokenBalance.value.error)}
          <span class="text-red-400">Error</span>
        {:else}
          {displayAmount}
        {/if}
      </div>
      <div class="font-medium">
        {token.representations[0]?.symbol ?? token.denom}
      </div>
    </div>
    <div class="text-zinc-400 text-nowrap text-xs flex items-center gap-1">
      {#if Option.isSome(chains.data)}
        {#each token.wrapping as wrapping, i}
          {#if i !== 0}
            <SharpArrowLeft class="text-sky-300"/>
          {/if}
          {toDisplayName(
            wrapping.unwrapped_chain.universal_chain_id,
            chains.data.value,
          )}
        {/each}
      {/if}
    </div>
</button>