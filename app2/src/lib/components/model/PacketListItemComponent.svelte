<script lang="ts">
import type { PacketListItem } from "$lib/schema/packet"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import { Option } from "effect"
import ChainComponent from "./ChainComponent.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import { goto } from "$app/navigation"

type Props = {
  packet: PacketListItem
}

const { packet }: Props = $props()

const sourceChain = $derived(
  Option.flatMap(chains.data, chainsData =>
    getChain(chainsData, packet.source_universal_chain_id.split(".")[1])
  )
)

const destinationChain = $derived(
  Option.flatMap(chains.data, chainsData =>
    getChain(chainsData, packet.destination_universal_chain_id.split(".")[1])
  )
)

const handleClick = () => {
  goto(`/explorer/packets/${packet.packet_hash}`)
}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex flex-col md:flex-row gap-4 p-4 border-b border-zinc-800 hover:bg-zinc-900 cursor-pointer"
  onclick={handleClick}
>
  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">From:</div>
      {#if Option.isSome(sourceChain)}
        <ChainComponent chain={sourceChain.value} />
      {:else}
        <div class="text-zinc-500">{packet.source_universal_chain_id}</div>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">To:</div>
      {#if Option.isSome(destinationChain)}
        <ChainComponent chain={destinationChain.value} />
      {:else}
        <div class="text-zinc-500">{packet.destination_universal_chain_id}</div>
      {/if}
    </div>
  </div>

  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Status:</div>
      <div class="text-sm">{packet.status}</div>
    </div>
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Channel:</div>
      <div class="text-sm">{packet.channel_version}</div>
    </div>
  </div>

  <div class="flex flex-col gap-2 flex-1">
    <div class="flex items-center gap-2">
      <div class="text-xs text-zinc-500">Sent:</div>
      <DateTimeComponent value={packet.packet_send_timestamp} />
    </div>
    {#if Option.isSome(packet.packet_recv_timestamp)}
      <div class="flex items-center gap-2">
        <div class="text-xs text-zinc-500">Received:</div>
        <DateTimeComponent value={packet.packet_recv_timestamp.value} />
      </div>
    {/if}
  </div>
</div>
