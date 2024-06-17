<script lang="ts">
import AssetCard from "$lib/components/asset-card.svelte"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types"
export let userAddr: UserAddresses
export let chains: Array<Chain>
import ScrollArea from "./ui/scroll-area/scroll-area.svelte"

let userBalances = userBalancesQuery({
  chains,
  userAddr
})
</script>

{#each $userBalances as balance, index}
  <div>
    <div class="pl-6 pb-3 flex items-baseline gap-3">
      <h3 class="font-bold font-supermolot text-2xl">{chains[index].display_name}</h3>
      <div class="text-xs font-mono text-muted-foreground">
        {userAddr.evm.canonical}
      </div>
    </div>
    {#if balance.isLoading}
      <p class="text-muted-foreground">Loading...</p>
    {:else if balance.isError}
      <p class="text-red-500">{balance.error}</p>
    {:else if balance.isSuccess}
    <ScrollArea orientation="horizontal">
      <div class="flex gap-4 px-6">
        {#if !(balance.data instanceof Error)}
          {#each balance.data as asset}
            <AssetCard {asset} chain={chains[index]} />
          {/each}
        {/if}
      </div>
    </ScrollArea>
    {/if}
  </div>
{/each}
