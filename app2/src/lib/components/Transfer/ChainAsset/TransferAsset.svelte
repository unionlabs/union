<script lang="ts">
import { cn } from "$lib/utils/index.js"
import { transfer } from "$lib/components/Transfer/transfer.svelte.js"
import type { Token } from "$lib/schema/token.ts"
import { type BalanceKey, balancesStore, createKey } from "$lib/stores/balances.svelte.ts"
import { Option } from "effect"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

//Fetching balance per asset, so we can only what's visible later.

type Props = {
  token: Token
  selectAsset: (token: Token) => void
}

let { token, selectAsset }: Props = $props()
let isSelected = $derived(transfer.raw.asset === token.denom)

let balanceState = $state({
  isLoading: false,
  amount: BigInt("0")
})

let balanceKey = $state<Option.Option<BalanceKey>>(Option.none())

$effect(() => {
  if (!Option.isSome(transfer.sourceChain)) return
  const sourceChain = transfer.sourceChain.value

  const addressOption = wallets.getAddressForChain(sourceChain)
  if (!Option.isSome(addressOption)) return

  const address = addressOption.value

  const newBalanceKey = createKey(sourceChain.universal_chain_id, address, token.denom)

  if (Option.isNone(balanceKey) || balanceKey.value !== newBalanceKey) {
    balanceState.isLoading = true
    balancesStore.fetchBalance(sourceChain, address, token.denom)
    balanceKey = Option.some(newBalanceKey)
  }

  const balance = balancesStore.getBalance(sourceChain.universal_chain_id, address, token.denom)

  balanceState.isLoading = false

  if (Option.isSome(balance)) {
    balanceState.amount = balance.value
  } else {
    balanceState.amount = BigInt("0")
  }
})
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
      {#if balanceState.isLoading}
        <Skeleton class="h-3 w-16"/>
      {:else}
        {balanceState.amount}
      {/if}
    </div>
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
         stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="9 18 15 12 9 6"></polyline>
    </svg>
  </div>
</button>