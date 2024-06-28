<script lang="ts">
import type { Chain, UserAddresses } from "$lib/types.ts"
import { userBalancesQuery } from "$lib/queries/balance"
import { truncate } from "$lib/utilities/format.ts"

export let userAddr: UserAddresses
export let chains: Array<Chain>
export let chainId: string
export let symbol: boolean

let cosmosBalances = userBalancesQuery({
  chains: chains.filter(c => c.chain_id === chainId),
  userAddr
})
</script>

{#each $cosmosBalances as balance, index}
  {#if balance.isLoading}
    <span class="text-muted-foreground">Loading...</span>
  {:else if balance.isError}
    <span class="text-red-500">{balance.error}</span>
  {:else if balance.isSuccess}
    {#if !(balance.data instanceof Error)}
      {#each balance.data as asset}
        <span>{asset.balance} {symbol ? ` ${truncate(asset.symbol, 8)}` : ''}</span>
      {/each}
    {/if}
  {/if}
{/each}
