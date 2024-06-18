<script lang="ts">
  import type { Chain } from "$lib/types";
  import { Button } from '$lib/components/ui/button';
import ExternalLinkIcon from "virtual:icons/lucide/external-link"

  export let chains: Chain[];
  import * as Card from "$lib/components/ui/card/index.ts"

  let chainsWithFaucets = chains.filter(chain => chain.assets.filter(asset => asset.faucets.length > 0).length > 0);
</script>

{#each chainsWithFaucets as chain}
  <Card.Root class="w-full max-w-lg">
      <Card.Header>
        <Card.Title>Faucets on {chain.display_name}</Card.Title>
        <p class="text-sm">Faucets on {chain.display_name} are provided by third parties and listed here for your convenience.</p>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4">
      {#each chain.assets.filter(asset => asset.faucets.length > 0) as assetWithFaucet}
        <section>
          <h3 class="font-supermolot font-bold text-lg">{assetWithFaucet.display_symbol}</h3>
          <ul>
            {#each assetWithFaucet.faucets as faucet, index}
              <li>
                <a
                  class="flex items-center gap-x-2  hover:underline"
                  href={faucet.url}
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  {index + 1}: {faucet.display_name}
                  <ExternalLinkIcon class="size-4" />
                </a>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
      </Card.Content>
  </Card.Root>
{/each}


