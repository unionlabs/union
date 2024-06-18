<script lang="ts">
  import type { Chain } from "$lib/types";
  import { Button } from '$lib/components/ui/button';

  export let chains: Chain[];

  let chainsWithFaucets = chains.filter(chain => chain.assets.filter(asset => asset.faucets.length > 0).length > 0);
</script>

{#each chainsWithFaucets as chain}
  <h2 class="font-supermolot font-bold text-lg">{chain.display_name}</h2>
  {#each chain.assets.filter(asset => asset.faucets.length > 0) as assetWithFaucet}
    <h3 class="font-supermolot font-bold text-lg">{assetWithFaucet.display_symbol}</h3>
    <ul>
      {#each assetWithFaucet.faucets as faucet, index}
        <li><Button variant="secondary" href={faucet.url}>{index}: {faucet.display_name}</Button>
      {/each}
    </ul>
  {/each}
{/each}


