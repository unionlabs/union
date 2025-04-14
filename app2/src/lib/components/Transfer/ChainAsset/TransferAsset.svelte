<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import { type Chain, type Token, TokenRawAmount } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import SharpLinkOffIcon from "$lib/components/icons/SharpLinkOffIcon.svelte"

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
    "flex flex-col items-start w-full overflow-x-auto px-4 py-3 text-left bg-zinc-900 hover:bg-zinc-800 transition-colors cursor-pointer rounded",
    isSelected ? "bg-zinc-700 text-white" : "text-zinc-300",
  )}
  onclick={() => {
    console.log(token);
    selectAsset(token);
  }}
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
          {@const icon = token.representations[0].logo_uri}
          <TokenComponent
            {chain}
            denom={token.denom}
            amount={tokenBalance.value.balance.value}
            {icon}
          />
        {:else}
          {@const icon = token.representations[0].logo_uri}
          <TokenComponent
            {chain}
            denom={token.denom}
            amount={TokenRawAmount.make(0n)}
            {icon}
          />
        {/if}
      {:else}
        {@const icon = token.representations[0].logo_uri}
        <div class="flex flex-row items-center gap-2">
          <SharpLinkOffIcon />
          <TokenComponent {chain} denom={token.denom} {icon} />
        </div>
      {/if}
    </div>
  </div>
</button>
