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
}
</script>

<div  class={cn("flex flex-col text-xs")} {...$$restProps}>
  <div class="text-md font-semibold">{value.name}</div>
  <div class="text-muted-foreground">{value.chain}</div>
  <div class="text-muted-foreground">{value.connection}</div>
  <div class="text-muted-foreground">{value.channel}</div>
  <div class="text-muted-foreground">{#if value.timestamp && value.timestamp !== undefined}<Duration totalUnits={3} variant="short" minUnits={DurationUnits.Second} start={value.timestamp}/>{:else}in transit{/if}</div>
</div>
