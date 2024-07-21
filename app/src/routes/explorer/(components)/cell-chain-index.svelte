<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellCopy from "./cell-copy.svelte";

export let value: {
  chain_display_name: string
  chain_id: string
}
</script>

<div  class={cn("flex flex-col ")} {...$$restProps}>
  <CellCopy value={value.chain_display_name} />
  <CellCopy value={value.chain_id} />
</div>
