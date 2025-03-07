<script lang="ts">
import type { TransferListItem } from "$lib/schema/transfer-list"
import { Option } from "effect"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "./ChainComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Label from "../ui/Label.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { goto } from "$app/navigation"
import SharpRightArrowIcon from "../icons/SharpRightArrowIcon.svelte"
import DateTimeComponent from "../ui/DateTimeComponent.svelte"

interface Props {
  transfer: TransferListItem
  showSeconds?: boolean
}

const { transfer, showSeconds = true }: Props = $props()

const handleClick = () => {
  goto(`/explorer/transfers/${transfer.packet_hash}`)
}
</script>

{#if Option.isSome(chains.data)}
  {@const chainss = chains.data.value}
  {@const sourceChain = getChain(chainss, transfer.source_chain.chain_id)}
  {@const destinationChain = getChain(
    chainss,
    transfer.destination_chain.chain_id,
  )}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex justify-between gap-8 px-4 py-3 h-16 cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-colors duration-75 items-center"
    onclick={handleClick}
  >
    <div>
      {#if Option.isSome(sourceChain)}
        <TokenComponent
          chain={sourceChain.value}
          denom={transfer.base_token}
          amount={transfer.base_amount}
        />
      {/if}
      <div class="flex items-center gap-1 text-zinc-400 text-sm">
        {#if Option.isSome(sourceChain)}
          <ChainComponent class="font-normal" chain={sourceChain.value} />
        {/if}
        <SharpRightArrowIcon class="size-5" />
        {#if Option.isSome(destinationChain)}
          <ChainComponent class="font-normal" chain={destinationChain.value} />
        {/if}
      </div>
    </div>
    <DateTimeComponent
      class="text-sm"
      value={transfer.transfer_send_timestamp}
      {showSeconds}
    />
  </div>
{/if}
