<script lang="ts">
import { truncate } from "$lib/utilities/format"
import type { Chain } from "$lib/types"
import { Button } from "$lib/components/ui/button/index.ts"
import Precise from "$lib/components/precise.svelte"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import { showUnsupported } from "$lib/stores/user.ts"

//Create correct type here
export let asset: { symbol: string; balance: bigint; denom?: string; address: string }
export let chain: Chain

//Finds the asset in chains.assets and checks if supported,
const identifier = asset.denom ? asset.denom : asset.address
$: supportedAsset = getSupportedAsset(chain, identifier)
</script>

{#if supportedAsset || $showUnsupported}
  <div class="asset-scene">
    <div class="asset-card asset-card asset-card--show-side my-4">
      <div class="asset-card__side" style="background-color: {supportedAsset ? '' : 'darkgray'}">
        <div class="uppercase font-bold">{truncate(supportedAsset ? supportedAsset.display_symbol : asset.symbol, 6)}</div>
        <div><Precise {asset} {chain} /></div>
      </div>
      <div class="asset-card__front flex flex-col" style="background-color: {supportedAsset ? '' : 'darkgray'}">
        <div class="uppercase font-bold">{truncate(supportedAsset ? supportedAsset.display_symbol : asset.symbol, 8)}</div>
        <div class="flex-1 text-xl font-mono"><Precise {asset} {chain} showToolTip /></div>
        <Button
          href={`/transfer?source=${encodeURIComponent(chain.chain_id)}&asset=${encodeURIComponent(asset.symbol)}`}
          variant="default"
          class={supportedAsset ? "" : "opacity-50"}
        >Transfer</Button>
      </div>
    </div>
  </div>
{/if}


<style lang="postcss">
    .asset-scene {
        --speed: 0.5s;
        perspective: 1000px;
        --height: 160px;
        --width: 240px;
        --side-width: calc(var(--width) / 3);
        height: calc(var(--height) * 1.2);
        width: calc(var(--side-width) * 1.0);
        transition: width var(--speed);
    }
    .asset-scene:hover {
        width: calc(var(--width) * 1.0);
    }

    .asset-card {
        position: relative;
        transform-style: preserve-3d;
        transition: transform var(--speed);
        transform: translateZ(calc(var(--side-width) * -1)) rotateY(90deg);
    }

    .asset-card:hover {
        transform: none;
    }

    .asset-card__side {
        @apply border p-4 bg-card absolute;
        width: var(--side-width);
        height: var(--height);
        writing-mode: vertical-rl;
        white-space: nowrap;
        left: var(--side-width);
        transform: rotateY(-90deg) translateZ(calc(var(--width) / 2));
    }
    .asset-card__front {
        @apply border p-4 bg-card absolute;
        width: var(--width);
        height: var(--height);
        transform: rotateY(0deg) translateZ(calc(var(--side-width) / 2));
    }
</style>

