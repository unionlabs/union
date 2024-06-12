<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Duration } from "svelte-ux"
import { DurationUnits } from "svelte-ux"

export let value: {
  name: string
  chain: unknown
  connection: unknown
  channel: unknown
  timestamp: unknown
  transaction_hash: string | null
}
</script>

<div  class={cn("flex flex-col text-xs text-muted-foreground")} {...$$restProps}>
  <div class="text-md font-semibold text-foreground">{value.name}</div>
  <div>{value.chain}</div>
  <div>{value.connection}</div>
  <div>{value.channel}</div>
  <div>{#if value.timestamp && value.timestamp !== undefined}<Duration totalUnits={3} variant="short" minUnits={DurationUnits.Second} start={value.timestamp}/>{:else}in transit{/if}</div>
  {#if value.transaction_hash !== null}<div>{value.transaction_hash}</div>{/if}
</div>
