<script lang="ts">
  import { cn } from "$lib/utilities/shadcn.ts"
  import CellCopy from "./cell-copy.svelte";

  export let value: {
    chain_display_name: string
    chain_id: string
    connection_id: string
    channel_id: string
    port_id: string
  }

</script>

<div {...$$restProps} class={cn("flex flex-col ")}>
  <CellCopy value={value.chain_display_name}/>
  <CellCopy value={value.chain_id}/>
  <CellCopy value={value.connection_id}/>
  <CellCopy value={value.channel_id}/>
  <CellCopy trunc={10} value={value.port_id}/>
</div>
