<script lang="ts">
import { transferListLatestQuery, transferListPageQuery } from "$lib/queries/transfer-list.svelte"
import { DateTime, Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"

let fiber: Fiber.Fiber

onMount(() => {
  fiber = Effect.runFork(transferListLatestQuery)
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})

const onNextPage = async () => {
  if (Option.isSome(transferList.data)) {
    let lastSortOrder = transferList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListPageQuery(lastSortOrder))
  }
}
</script>


<Sections>
  <h1 class="font-bold text-4xl">Transfers</h1>
  <Card class="overflow-auto p-0 divide-y divide-zinc-800">
    {#if Option.isSome(transferList.data) && Option.isSome(chains.data)}
      {@const chainss = chains.data.value}
      {#each transferList.data.value as transfer, index (transfer)}
        {@const sourceChain = getChain(chainss, transfer.source_chain_id)}
        {@const destinationChain = getChain(chainss, transfer.destination_chain_id)}
        <div class="flex gap-8 px-4 py-2 flex-cols-3" animate:flip>
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
      {/each}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {:else}
      {#each [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]}
        <div class="h-[57px] last:h-[56px]"></div>
      {/each}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {/if}
  </Card>
  <div class="flex">
    <div class="rounded shadow flex">
      <button class="cursor-pointer border-l border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tl rounded-bl">
        ←
      </button>
      <div class="bg-zinc-900 border-t border-b border-zinc-800 flex items-center px-4">
        Current
      </div>
      <button onclick={onNextPage} class="cursor-pointer border-r border-t border-b bg-zinc-700 border-zinc-600 h-10 w-10 rounded-tr rounded-br">
        →
      </button>
    </div>
  </div>
</Sections>
