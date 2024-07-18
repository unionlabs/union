<script lang="ts">
  import { cn } from "$lib/utilities/shadcn.ts"
  import * as Tooltip from "$lib/components/ui/tooltip"
  import { truncate } from "$lib/utilities/format.ts"
  import { copy } from "$lib/utilities/copy.ts";
  import CopyIcon from "virtual:icons/lucide/copy"


  export let value: {
    chain_display_name: string
    chain_id: string
    connection_id: string
    channel_id: string
    port_id: string
  }
</script>

<div {...$$restProps} class={cn("flex flex-col ")}>
  <div class="font-bold inline-flex items-center gap-2 group" use:copy>
    {value.chain_display_name}
    <span class="opacity-0 group-hover:opacity-100 hover:text-accent transition">
      <CopyIcon class="size-4"/>
    </span>
  </div>
  <div class="inline-flex items-center gap-2 group" use:copy>
    {value.chain_id}
    <span class="opacity-0 group-hover:opacity-100 hover:text-accent transition">
      <CopyIcon class="size-4"/>
    </span>
  </div>
  <div class="inline-flex items-center gap-2 group" use:copy>
    {value.connection_id}
    <span class="opacity-0 group-hover:opacity-100 hover:text-accent transition">
      <CopyIcon class="size-4"/>
    </span>
  </div>
  <div class="inline-flex items-center gap-2 group" use:copy>
    {value.channel_id}
    <span class="opacity-0 group-hover:opacity-100 hover:text-accent transition">
      <CopyIcon class="size-4"/>
    </span>
  </div>
  <Tooltip.Root>
    <Tooltip.Trigger>
      <div class="text-start flex items-center gap-2 group w-full" use:copy={value.port_id}>
        {truncate(value.port_id, 10)}
        <span class="opacity-0 group-hover:opacity-100 hover:text-accent transition">
          <CopyIcon class="size-4"/>
        </span>
      </div>
    </Tooltip.Trigger>
    <Tooltip.Content>
      {value.port_id}
    </Tooltip.Content>
  </Tooltip.Root>
</div>
