<script lang="ts">
import { truncate } from "$lib/utilities/format"
import { cn } from "$lib/utilities/shadcn.ts"
import ExplorerPrecise from "$lib/components/explorer-precise.svelte"

export let value: object
</script>

<div {...$$restProps} class={cn("flex flex-col")}>
  {#each Object.entries(value) as [denom, data]}
    {#if data.info}
      <div>
        <ExplorerPrecise amount={data.amount} decimals={data.info.decimals} showToolTip displayDecimals={6}/> {data.info.display_symbol}
      </div>
      {:else}
      <div>
        {data.amount} {truncate(denom, 6)}
      </div>
    {/if}
    <!--<div class="text-muted-foreground font-mono">{JSON.stringify(info)}</div>!-->
  {/each}
</div>
