<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import * as Tooltip from "$lib/components/ui/tooltip"
    import { truncate } from "$lib/utilities/format";

export let value: {
  chain_display_name: string
  address: string
}
</script>

<div  class={cn("flex flex-col items-start")} {...$$restProps}>
  <div class="font-bold">{value.chain_display_name}</div>
  <Tooltip.Root>
    <Tooltip.Trigger>
      <div>{truncate(value.address,8)}</div>
    </Tooltip.Trigger>
      <Tooltip.Content>
        <p>{value.address}</p>
      </Tooltip.Content>
  </Tooltip.Root>
</div>
