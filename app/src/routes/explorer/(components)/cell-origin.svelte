<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { Duration } from "svelte-ux"
import { DurationUnits } from "svelte-ux"

export let value: {
  name: string
  chain: unknown
  connection: unknown
  channel: unknown
}
</script>

<div  class={cn("flex flex-col ")} {...$$restProps}>
  <div class="font-bold">{value.name}</div>
  <div>{value.chain}</div>
  <div>{value.connection}</div>
  <div>{value.channel}</div>
</div>
