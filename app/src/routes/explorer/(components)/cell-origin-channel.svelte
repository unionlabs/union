<script lang="ts">
  import { cn } from "$lib/utilities/shadcn.ts"
  import * as Tooltip from "$lib/components/ui/tooltip"
  import { truncate } from "$lib/utilities/format.ts"
  import { copy } from "$lib/utilities/copy.ts";


  export let value: {
    chain_display_name: string
    chain_id: string
    connection_id: string
    channel_id: string
    port_id: string
  }
</script>

<div {...$$restProps} class={cn("flex flex-col ")}>
  <div class="font-bold" use:copy>{value.chain_display_name}<span></span></div>
  <div use:copy>{value.chain_id}</div>
  <div use:copy>{value.connection_id}</div>
  <div use:copy>{value.channel_id}</div>
  <Tooltip.Root>
    <Tooltip.Trigger>
      <div class="text-start" use:copy={value.port_id}>{truncate(value.port_id, 10)}</div>
    </Tooltip.Trigger>
    <Tooltip.Content>
      {value.port_id}
    </Tooltip.Content>
  </Tooltip.Root>
</div>
