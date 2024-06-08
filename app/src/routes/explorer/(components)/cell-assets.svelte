<script lang="ts">
import { truncate } from "$lib/utilities/format"
import { cn } from "$lib/utilities/shadcn.ts"

export let value: object
</script>

<div  class={cn("flex flex-col")} {...$$restProps}>
  {#each Object.entries(value) as [denom, info]}
    <div class="font-semibold">{info.amount} {truncate(denom, 6)}</div>
    <!--<div class="text-muted-foreground font-mono">{JSON.stringify(info)}</div>!-->

  {/each}
  
</div>
