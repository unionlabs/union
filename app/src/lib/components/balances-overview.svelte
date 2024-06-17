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

let evmBalances = evmBalancesQuery({
  chainId: "11155111",
  address: userAddr.evm.canonical,
  tokenSpecification: "erc20"
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


<!--
<div>
  <h3 class="font-bold">Sepolia</h3>
  {#if $evmBalances.isLoading}
    Loading...
  {:else if $evmBalances.isError}
    Error: {$evmBalances.error.message}
  {:else if $evmBalances.isSuccess}
    <div>
      {#each $evmBalances.data as asset}
        <div>{truncate(asset.symbol, 8)} | {asset.balance}</div>
      {/each}
    </div>
  {/if}
</div>
!-->

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
          {#each balance.data as asset}
            <AssetCard {asset}/>
          {/each}
        {/if}
      </div>
    </ScrollArea>
    {/if}
  </div>
{/each}
