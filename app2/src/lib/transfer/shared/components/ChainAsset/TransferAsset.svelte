<script lang="ts">
import SharpLinkOffIcon from "$lib/components/icons/SharpLinkOffIcon.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { cn } from "$lib/utils/index.js"
import { type Chain, type Token, TokenRawAmount } from "@unionlabs/sdk/schema"
import { Option } from "effect"

type Props = {
  token: Token
  chain: Chain
  selectAsset: (token: Token) => void
  index: number
}

let { token, chain, selectAsset, index }: Props = $props()

let isSelected = $derived(transferData.raw.asset === token.denom)

let tokenBalance = $derived.by(() => {
  if (Option.isNone(transferData.sortedBalances)) {
    return Option.none()
  }
  const found = transferData.sortedBalances.value.find(t => t.token.denom === token.denom)
  return found ? Option.some(found) : Option.none()
})

let isLoading = $derived(Option.isSome(transferData.sortedBalances) && Option.isNone(tokenBalance))

export const toDisplayName = (
  chain_id: string | undefined | null,
  chains: ReadonlyArray<Chain>,
): string => chains.find(c => c.chain_id === chain_id)?.display_name ?? chain_id ?? "unknown chain"
</script>

<button
  style="animation-delay: {index * 50}ms;"
  class={cn(
    "flex flex-col items-start w-full overflow-hidden px-4 py-3 text-left rounded transition-all duration-100 border cursor-pointer",
    "animate-slide-in-right opacity-0",
    isSelected 
      ? "bg-zinc-900 border-accent text-white"
      : "bg-zinc-900 border-zinc-800 hover:border-zinc-600 text-zinc-300",
  )}
  onclick={() => selectAsset(token)}
>
  <div class="flex items-center gap-1 overflow-x-auto text-sm text-zinc-200">
    <div class="mr-1">
      {#if isLoading}
        <Skeleton class="h-3 w-16" />
      {:else if Option.isSome(tokenBalance) && Option.isSome(tokenBalance.value.error)}
        <span class="text-red-400">
          {tokenBalance.value.error?.value._tag ?? "Error"}
        </span>
      {:else if Option.isSome(tokenBalance)}
        {#if Option.isSome(tokenBalance.value.balance)}
          <TokenComponent
            {chain}
            denom={token.denom}
            amount={tokenBalance.value.balance.value}
            icon={token.representations[0]?.logo_uri}
          />
        {:else}
          <TokenComponent
            {chain}
            denom={token.denom}
            amount={TokenRawAmount.make(0n)}
            icon={token.representations[0]?.logo_uri}
          />
        {/if}
      {:else}
        <div class="flex flex-row items-center gap-2">
          <SharpLinkOffIcon />
          <TokenComponent
            {chain}
            denom={token.denom}
            icon={token.representations[0]?.logo_uri}
          />
        </div>
      {/if}
    </div>
  </div>
</button>

<style>
  @keyframes slide-in-right {
    from {
      opacity: 0;
      transform: translateX(30px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateX(0) scale(1);
    }
  }
  
  .animate-slide-in-right {
    animation: slide-in-right 0.6s ease-out forwards;
  }
</style>
