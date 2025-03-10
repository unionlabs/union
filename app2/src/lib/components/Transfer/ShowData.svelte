<script lang="ts">

  import {getTransfer} from "$lib/components/Transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()
</script>
<ul class="text-xs text-sky-300 w-full truncate">
  <li>
    <span class="text-zinc-300">sourceChain</span>:
    {#if transfer.sourceChain?.chain_id}
      {transfer.sourceChain.chain_id}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">destinationChain</span>:
    {#if transfer.destinationChain?.chain_id}
      {transfer.destinationChain.chain_id}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">receiver</span>:
    {#if transfer.derivedReceiver}
      {transfer.derivedReceiver}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">baseToken</span>:
    {#if transfer.baseToken?.denom}
      {transfer.baseToken.denom}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">baseAmount</span>:
    {#if transfer.parsedAmount}
      {transfer.parsedAmount.toString()}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">quoteToken</span>:
    {#if transfer.quoteToken}
      {transfer.quoteToken}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">quoteAmount</span>:
    {#if transfer.parsedAmount}
      {transfer.parsedAmount.toString()}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">wethToken</span>:
    {#if transfer.wethQuoteToken}
      {transfer.wethQuoteToken}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">ucs03address</span>:
    {#if transfer.ucs03address}
      {transfer.ucs03address}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">sourceChannelId</span>:
    {#if transfer.channel?.source_channel_id}
      {transfer.channel.source_channel_id}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">timeoutTimestamp</span>:
    {#if transfer.args.timeoutTimestamp}
      {transfer.args.timeoutTimestamp}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">timeoutHeight</span>:
    {#if transfer.args.timeoutHeight !== null}
      {transfer.args.timeoutHeight}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>

</ul>