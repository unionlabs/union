<script lang="ts">
import type { PacketListItem } from "@unionlabs/sdk/schema"
import { getChain } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import ChainComponent from "./ChainComponent.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import { goto } from "$app/navigation"
import SharpRightArrowIcon from "../icons/SharpRightArrowIcon.svelte"
import PacketHashComponent from "./PacketHashComponent.svelte"

type Props = {
  packet: PacketListItem
}

const { packet }: Props = $props()

const sourceChain = $derived(
  Option.flatMap(chains.data, chainsData => getChain(chainsData, packet.source_universal_chain_id))
)

const destinationChain = $derived(
  Option.flatMap(chains.data, chainsData =>
    getChain(chainsData, packet.destination_universal_chain_id)
  )
)

const handleClick = () => {
  goto(`/explorer/packets/${packet.packet_hash}`)
}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex justify-between gap-8 px-4 py-3 h-16 cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-900 transition-colors duration-75 items-center"
    onclick={handleClick}
  >
    <div>
      <PacketHashComponent hash={packet.packet_hash} />
      <div class="flex items-center gap-1 text-zinc-400 text-sm">
        {#if Option.isSome(sourceChain)}
          <ChainComponent chain={sourceChain.value} />
        {:else}
          <div class="text-zinc-500">{packet.source_universal_chain_id}</div>
        {/if}
        <SharpRightArrowIcon class="size-5" />
        {#if Option.isSome(destinationChain)}
          <ChainComponent chain={destinationChain.value} />
        {:else}
          <div class="text-zinc-500">{packet.destination_universal_chain_id}</div>
        {/if}
      </div>
    </div>
  <DateTimeComponent
    class="text-sm hidden sm:block"
    value={packet.packet_send_timestamp} />
</div>
