<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellTimestamp from "./cell-timestamp.svelte"

export let value: {
  sequence: number | undefined
  timestamp: string | undefined
}
</script>

<div class={cn("flex flex-col")} {...$$restProps}>
  {#if value.sequence}<div class="text-xl">{value.sequence}</div>{/if}
  <div class="text-muted-foreground">{#if value.timestamp}<CellTimestamp value={value.timestamp}/>{/if}</div>
</div>
