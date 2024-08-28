<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import ExplorerPrecise from "$lib/components/explorer-precise.svelte"

export let value: object

console.log(value)
</script>

<div {...$$restProps} class={cn("flex flex-col")}>
  {#each Object.entries(value) as [denom, data]}
    {#if data.asset}
      <div>
        <ExplorerPrecise amount={data.amount} decimals={data.asset.decimals} showToolTip displayDecimals={6}/> {data.asset.display_symbol}
      </div>
      {:else}
      <div>
        {data.amount} {truncate(data.denom, 6)}
      </div>
    {/if}
    <!--<div class="text-muted-foreground font-mono">{JSON.stringify(info)}</div>!-->
  {/each}
</div>
