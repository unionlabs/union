<script lang="ts">
import ChainDetails from "$lib/chain-details.svelte"
import type { Chain } from "$lib/types"
import { cn } from "$lib/utilities/shadcn.ts"
import { fromHex, isHex } from "viem"
import Address from "../address.svelte"
import CellCopy from "./cell-copy.svelte"
import { bech32AddressToHex } from "@unionlabs/client"

export let chains: Array<Chain>

export let value: {
  chain_display_name: string | undefined
  chain_id: string | undefined
  connection_id: string | undefined
  channel_id: string | undefined
  port_id: string | undefined
}

export const chain = chains.find(c => c.chain_id === value.chain_id)
export const port =
  chain?.rpc_type === "cosmos" && isHex(value.port_id)
    ? bech32AddressToHex({ address: fromHex(value.port_id, "string") })
    : value.port_id
</script>

<div {...$$restProps} class={cn("flex flex-col items-start")}>

  {#if chain}
  <div class="font-bold">
    <ChainDetails {chain}/>
  </div>
  {#if value.chain_id && value.channel_id && value.connection_id}<div class="text-muted-foreground">{value.chain_id} | {value.connection_id} | {value.channel_id}</div>{/if}
  {#if port && value.chain_id}<Address {chains} chainId={value.chain_id} address={port}/>{/if}
  {:else}
    <div>chain {value.chain_id} not found</div>
  {/if}
</div>
