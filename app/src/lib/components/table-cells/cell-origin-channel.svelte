<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellCopy from "./cell-copy.svelte"

export let value: {
  chain_display_name: string | undefined
  chain_id: string | undefined
  connection_id: string | undefined
  channel_id: string | undefined
  port_id: string | undefined
}
</script>

<div {...$$restProps} class={cn("flex flex-col ")}>
  {#if value.chain_display_name}<CellCopy value={value.chain_display_name}/>{/if}
  {#if value.chain_id}<CellCopy value={value.chain_id}/>{/if}
  {#if value.connection_id}<CellCopy value={value.connection_id}/>{/if}
  {#if value.channel_id}<CellCopy value={value.channel_id}/>{/if}
  <div class="text-muted-foreground">{#if value.port_id}<CellCopy trunc={10} value={value.port_id}/>{/if}</div>
</div>
