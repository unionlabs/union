<script lang="ts">
import AssetCard from '$lib/components/asset-card.svelte';
import { cosmosBalancesQuery, evmBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types"
export let userAddr: UserAddresses
export let chains: Array<Chain>
import { truncate } from "$lib/utilities/format"
import { rawToBech32, rawToHex } from "$lib/utilities/address"
import { onMount } from "svelte"
import ScrollArea from "./ui/scroll-area/scroll-area.svelte";


let evmChains = chains.filter(c => c.rpc_type === "evm")
let evmBalances = evmBalancesQuery({
  chains: evmChains,
  address: userAddr.evm.canonical,
})

let cosmosChains = chains.filter(c => c.rpc_type === "cosmos")

let cosmosBalances = cosmosBalancesQuery({
  chains: cosmosChains,
  address: userAddr.cosmos.bytes
})
onMount(() => {
  console.log(userAddr)
})
</script>



{#each $evmBalances as balance, index}
  <div>
    <div class="pl-6 pb-3 flex items-baseline gap-3">
      <h3 class="font-bold font-supermolot text-2xl">{evmChains[index].display_name}</h3>
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
            <AssetCard {asset}/>
          {/each}
        {/if}
      </div>
    </ScrollArea>
    {/if}
  </div>
{/each}

{#each $cosmosBalances as balance, index}
  <div>
    <div class="pl-6 pb-3 flex items-baseline gap-3">
      <h3 class="font-bold font-supermolot text-2xl">{cosmosChains[index].display_name}</h3>
      <div class="text-xs font-mono text-muted-foreground">
        {rawToBech32(cosmosChains[index].addr_prefix, userAddr.cosmos.bytes)}
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
            <AssetCard {asset}/>
          {/each}
        {/if}
      </div>
    </ScrollArea>
    {/if}
  </div>
{/each}
