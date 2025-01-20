<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import Truncate from "../truncate.svelte"
import CellTimestamp from "./cell-timestamp.svelte"

export let value: {
  transaction_hash: string | undefined
  timestamp: string | undefined
}

// NOTE: this cell component name is outdated.
// It's no longer a sequence, but rather a tx_hash timestamp combo
</script>

<div class={cn("flex flex-col items-start")} {...$$restProps}>
  {#if value.transaction_hash}<Truncate value={value.transaction_hash} type="hash"/>{/if}
  <div class="text-muted-foreground">{#if value.timestamp}<CellTimestamp value={value.timestamp}/>{/if}</div>
</div>
