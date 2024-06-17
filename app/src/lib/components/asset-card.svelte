<script lang="ts">
  import { truncate } from "$lib/utilities/format";

  export let asset: { symbol: string, balance: string };

</script>
<div class="asset-scene">
  <div class="asset-card asset-card asset-card--show-side my-4">
    <div class="asset-card__side">
      <div class="uppercase font-bold">{truncate(asset.symbol, 6)}</div>
      <div>{asset.balance}</div>
    </div>
    <div class="asset-card__front">
      {truncate(asset.symbol, 8)} {asset.balance}
    </div>
  </div>
</div>


<style lang="postcss">
  .asset-scene {
    perspective: 1000px;
    --height: 160px;
    --width: 240px;
    --side-width: calc(var(--width) / 3);
    height: calc(var(--height) * 1.2);
    width: calc(var(--width) * 1.2);
    transition: width 1s;
  }
  .asset-scene:hover {
    width: calc(var(--side-width) * 1.2);
  }

  .asset-card {
    position: relative;
    transform-style: preserve-3d;
    transition: transform 1s;
  }

  .asset-card:hover {
    transform: translateZ(calc(var(--side-width) * -1)) rotateY(90deg);
  }

  .asset-card__side {
    @apply border p-4 bg-card absolute;
    opacity: 50%;
    width: var(--side-width);
    height: var(--height);
    writing-mode: vertical-rl;
    white-space: nowrap;
    left: var(--side-width);
    transform: rotateY(-90deg) translateZ(calc(var(--width) / 2));
  }
  .asset-card__front {
    opacity: 50%;
    @apply border p-4 bg-card absolute;
    width: var(--width);
    height: var(--height);
    transform: rotateY(0deg) translateZ(calc(var(--side-width) / 2));
  }
</style>
