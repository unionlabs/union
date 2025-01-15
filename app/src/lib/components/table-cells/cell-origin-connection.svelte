<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellCopy from "./cell-copy.svelte"

export let value: {
  chain_display_name: string
  chain_id: string
  connection_id: string
  client_id: string
}
</script>

<div  class={cn("flex flex-col ")} {...$$restProps}>
  <CellCopy value={value.chain_display_name}/>
  <CellCopy label="Chain ID:"  value={value.chain_id}/>
  <CellCopy label="Connection ID:" value={value.connection_id}/>
  <CellCopy label="Client ID:" value={value.client_id}/>
</div>
