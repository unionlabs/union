<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import { TokenRawAmount, type Chain, type Token } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { chains } from "$lib/stores/chains.svelte.ts"
import SharpArrowLeft from "$lib/components/icons/SharpArrowLeft.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"

type Props = {
  token: Token
  chain: Chain
  selectAsset: (token: Token) => void
}

let { token, chain, selectAsset }: Props = $props()

let isSelected = $derived(transfer.raw.asset === token.denom)

let tokenBalance = $derived.by(() => {
  if (Option.isNone(transfer.sortedBalances)) return Option.none()
  const found = transfer.sortedBalances.value.find(t => t.token.denom === token.denom)
  return found ? Option.some(found) : Option.none()
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
    <div class="flex items-center gap-1 overflow-x-scroll text-sm text-zinc-200">
      <div class="mr-1">
        {#if isLoading}
          <Skeleton class="h-3 w-16"/>
        {:else if Option.isSome(tokenBalance) && Option.isSome(tokenBalance.value.error)}
          <span class="text-red-400">Error</span>
        {:else if Option.isSome(tokenBalance)}
          {#if Option.isSome(tokenBalance.value.balance)}
            <TokenComponent {chain} denom={token.denom} amount={tokenBalance.value.balance.value} />
          {:else}
            <TokenComponent {chain} denom={token.denom} amount={TokenRawAmount.make(0n)} />
          {/if}
        {/if}
      </div>
    </div>
</button>
