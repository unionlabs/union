<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellCopy from "./cell-copy.svelte"

  interface Props {
    value: {
  chain_display_name: string
  chain_id: string
  connection_id: string
  client_id: string
};
    [key: string]: any
  }

  let { value, ...rest }: Props = $props();
</script>

<div  class={cn("flex flex-col ")} {...rest}>
  <CellCopy value={value.chain_display_name}/>
  <CellCopy value={value.chain_id}/>
  <CellCopy value={value.connection_id}/>
  <CellCopy value={value.client_id}/>
</div>
