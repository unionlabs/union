<script lang="ts">
import type { Chain } from "$lib/types"
import { cn } from "$lib/utilities/shadcn.ts"
import Address from "../address.svelte"
import CellCopy from "./cell-copy.svelte"

export let chains: Array<Chain>

export let value: {
  chain_display_name: string | undefined
  chain_id: string | undefined
  connection_id: string | undefined
  channel_id: string | undefined
  port_id: string | undefined
}
</script>

<div {...$$restProps} class={cn("flex flex-col items-start")}>
  {#if value.chain_display_name}<CellCopy class="font-bold" value={value.chain_display_name}/>{/if}
  {#if value.chain_id && value.channel_id && value.connection_id}<div class="text-muted-foreground">{value.chain_id} | {value.connection_id} | {value.channel_id}</div>{/if}
  {#if value.port_id && value.chain_id}<Address {chains} chainId={value.chain_id} address={value.port_id}/>{/if}
</div>
