<script lang="ts">
import { transferListLatestQuery } from "$lib/queries/transfer-list.svelte"
import { Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"

onMount(() => {
  const fiber = Effect.runFork(transferListLatestQuery)
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})
</script>


<Sections>
  <h1 class="font-bold text-4xl">Transfers</h1>
  <Card class="overflow-auto p-0 divide-y divide-zinc-800">
    {#if Option.isSome(transferList.data) && Option.isSome(chains.data)}
      {@const chainss = chains.data.value}
      {#each transferList.data.value as transfer}
        {@const sourceChain = getChain(chainss, transfer.source_chain_id)}
        {@const destinationChain = getChain(chainss, transfer.destination_chain_id)}
        <div class="flex gap-8 px-4 py-2 flex-cols-3">
          <div class="flex-1">
            <p class="uppercase text-zinc-500 text-xs font-semibold">from</p>
            {#if Option.isSome(sourceChain)}
              <ChainComponent chain={sourceChain.value}/>
            {/if}
          </div>
          <div class="flex-1">
            <p class="uppercase text-zinc-500 text-xs font-semibold">to</p>
            {#if Option.isSome(destinationChain)}
              <ChainComponent chain={destinationChain.value}/>
            {/if}
          </div>
          <div class="flex-1">
            yeah
          </div>
        </div>
      {/each}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {:else}
      {#each [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20]}
        <div class="h-[57px] p-4">
        </div>
      {/each}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {/if}
  </Card>
</Sections>
