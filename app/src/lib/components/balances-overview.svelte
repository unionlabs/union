<script lang="ts">
import type { Chain, UserAddresses } from "$lib/types"
import { userBalancesQuery } from "$lib/queries/balance"
import AssetCard from "$lib/components/asset-card.svelte"
import ScrollArea from "./ui/scroll-area/scroll-area.svelte"
import LoadingLogo from "./loading-logo.svelte"
    import { userAddrOnChain } from "$lib/utilities/address";

export let userAddr: UserAddresses
export let chains: Array<Chain>

let userBalances = userBalancesQuery({
  chains,
  userAddr
})
</script>

{#each $userBalances as balance, index}
  {@const chain = chains[index]}
  <div class="pt-6">
    <div class="pl-3 sm:pl-6 flex flex-col sm:flex-row items-baseline gap-3">
      <h3 class="font-bold font-supermolot text-2xl">{chain.display_name}</h3>
      <div class="text-xs font-mono text-muted-foreground">
        {userAddrOnChain(userAddr, chain)}
      </div>
    </div>
    {#if !!balance.data}
      <ScrollArea orientation="horizontal">
        <div class="flex gap-4 px-3 sm:px-6 overflow-x-scroll">
          {#if !(balance.data instanceof Error)}
            {#each balance.data as asset}
              <AssetCard {asset} chain={chains[index]} />
            {/each}
          {/if}
        </div>
      </ScrollArea>
    {:else if balance.isLoading}
      <div class="h-[192px] flex items-center">
        <LoadingLogo/>
      </div>
    {:else if balance.isError}
      <p class="text-red-500">{balance.error}</p>
    {/if}
  </div>
{/each}
