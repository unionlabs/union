<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { copy } from "$lib/utilities/copy.ts";
import CopyIcon from "virtual:icons/lucide/copy"

export let value: {
  chain_display_name: string
  chain_id: string
}
</script>

<div  class={cn("flex flex-col ")} {...$$restProps}>
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
</div>
