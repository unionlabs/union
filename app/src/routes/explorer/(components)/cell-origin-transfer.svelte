<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import * as Tooltip from "$lib/components/ui/tooltip"

export let value: {
  address: string
  chain_display_name: string
}
</script>

<div class={cn("flex flex-col items-start")} {...$$restProps}>
  {#if value.chain_display_name}
    <div class="font-bold">
      {value.chain_display_name}
    </div>
  {/if}
  <Tooltip.Root>
    <Tooltip.Trigger>
      <div>{truncate(value.address, 8)}</div>
    </Tooltip.Trigger>
    <Tooltip.Content>
      <p>{value.address}</p>
    </Tooltip.Content>
  </Tooltip.Root>
</div>
