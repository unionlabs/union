<script lang="ts">
import type { TransferListItem } from "$lib/schema/transfer-list"
import { Option } from "effect"
import { DateTime } from "effect"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "./ChainComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Label from "../ui/Label.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
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
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="flex gap-8 px-4 py-2 h-[60px] cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors"
    onclick={handleClick}
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
    <div class="flex-1">
      <Label>Base Token</Label>
      {#if Option.isSome(sourceChain)}
        <TokenComponent 
          chain={sourceChain.value} 
          denom={transfer.base_token} 
          amount={transfer.base_amount}
        />
      {/if}
    </div>
    {#if settingsStore.showQuoteTokens}
      <div class="flex-1">
        <Label>Quote Token</Label>
        {#if Option.isSome(destinationChain)}
          <TokenComponent 
            chain={destinationChain.value} 
            denom={transfer.quote_token} 
            amount={transfer.quote_amount}
          />
        {/if}
      </div>
    {/if}
  </div>
{/if}
