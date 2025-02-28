<script lang="ts">
import type { TransferListItem } from "$lib/schema/transfer-list"
import { Option } from "effect"
import { DateTime } from "effect"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "./ChainComponent.svelte"
import Label from "../ui/Label.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { goto } from "$app/navigation"

const { transfer }: { transfer: TransferListItem } = $props()

const handleClick = () => {
  goto(`/explorer/transfers/${transfer.packet_hash}`)
}
</script>

{#if Option.isSome(chains.data)}
  {@const chainss = chains.data.value}
  {@const sourceChain = getChain(chainss, transfer.source_chain_id)}
  {@const destinationChain = getChain(chainss, transfer.destination_chain_id)}
  <div 
    class="flex gap-8 px-4 py-2 h-[60px] cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
    on:click={handleClick}
  >
    <div class="flex-1">
      <Label>from</Label>
      {#if Option.isSome(sourceChain)}
        <ChainComponent chain={sourceChain.value}/>
      {/if}
    </div>
    <div class="flex-1">
      <Label>to</Label>
      {#if Option.isSome(destinationChain)}
        <ChainComponent chain={destinationChain.value}/>
      {/if}
    </div>
    <div class="flex-1">
      <Label>Time</Label>
      {DateTime.formatIso(transfer.packet_send_timestamp)}
    </div>
  </div>
{/if}
