<script lang="ts">
import type { Chain } from "$lib/types"
import ExternalLinkIcon from "virtual:icons/lucide/external-link"

export let chains: Array<Chain>
import * as Card from "$lib/components/ui/card/index.ts"

let chainsWithFaucets = chains.filter(
  chain => chain.assets.filter(asset => asset.faucets.length > 0).length > 0
)

let filterAndOrderAssets = (assets: Chain["assets"]): Chain["assets"] => {
  const filtered = assets.filter(asset => asset.faucets.length > 0)
  filtered.sort((a, _) => (a.denom === "native" ? -1 : 1))

  filtered.map(asset => {
    let a = asset
    a.faucets.sort((x, y) =>
      x.display_name > y.display_name ? 1 : y.display_name > x.display_name ? -1 : 0
    )
    return a
  })

  return filtered
}
</script>

{#each chainsWithFaucets as chain}
  <Card.Root class="w-full max-w-lg">
      <Card.Header>
        <Card.Title>Faucets on {chain.display_name}</Card.Title>
        <p class="text-sm">Faucets on {chain.display_name} are provided by third parties and listed here for your convenience.</p>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4">
      {#each filterAndOrderAssets(chain.assets) as assetWithFaucet}
        <section>
          <div class="flex items-center gap-2">
          <h3 class="font-supermolot font-bold text-xl">{assetWithFaucet.display_symbol}</h3>
          {#if assetWithFaucet.denom === "native"}<div class="uppercase font-supermolot font-semibold font-condensed text-xs px-2 rounded-full bg-primary text-primary-foreground">gas token</div>{/if}
          </div>
          <ul>
            {#each assetWithFaucet.faucets as faucet}
              <li>
                <a
                  class="flex items-center gap-x-2  hover:underline"
                  href={faucet.url}
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  {faucet.display_name}
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


