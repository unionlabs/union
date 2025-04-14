<script lang="ts">
  import {transfer} from "$lib/components/Transfer/transfer.svelte.ts"
  import {Option} from "effect"
</script>

<ul class="text-xs text-sky-300 w-full truncate">
  <li>
    <span class="text-zinc-300">sourceChain</span>:
    {#if Option.isSome(transfer.sourceChain)}
      {transfer.sourceChain.value.chain_id}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">destinationChain</span>:
    {#if Option.isSome(transfer.destinationChain)}
      {transfer.destinationChain.value.chain_id}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">receiver</span>:
    {#if Option.isSome(transfer.derivedReceiver)}
      {transfer.derivedReceiver.value}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">baseToken</span>:
    {#if Option.isSome(transfer.baseToken)}
      {transfer.baseToken.value.denom}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">baseAmount</span>:
    {#if Option.isSome(transfer.parsedAmount)}
      {transfer.parsedAmount.value.toString()}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">quoteToken</span>:
    {#if Option.isSome(transfer.quoteToken)}
      {JSON.stringify(transfer.quoteToken.value)}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">quoteAmount</span>:
    {#if Option.isSome(transfer.parsedAmount)}
      {transfer.parsedAmount.value.toString()}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">wethToken</span>:
    {#if Option.isSome(transfer.wethQuoteToken)}
      {JSON.stringify(transfer.wethQuoteToken.value)}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">ucs03address</span>:
    {#if Option.isSome(transfer.ucs03address)}
      {transfer.ucs03address.value}
    {:else}
      <span class="text-red-500">No value</span>
    {/if}
  </li>
  <li>
    <span class="text-zinc-300">sourceChannelId</span>:
    {#if Option.isSome(transfer.channel) && transfer.channel.value.source_channel_id}
      {transfer.channel.value.source_channel_id}
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