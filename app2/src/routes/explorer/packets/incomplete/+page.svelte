<script lang="ts">
import { runInWindowAllPairs } from "$lib/queries/missing-packets-list"
import { Effect, Option } from "effect"
import { onMount } from "svelte"
import { incompletePacketsList } from "$lib/stores/incomplete-packets.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"

import { settingsStore } from "$lib/stores/settings.svelte"
import PacketListItemComponent from "$lib/components/model/PacketListItemComponent.svelte"
import PacketListItemComponentSkeleton from "$lib/components/model/PacketListItemComponentSkeleton.svelte"

onMount(() => {
  runInWindowAllPairs("mainnet")

  return () => {
    incompletePacketsList.interruptFiber()
  }
})
</script>

<Sections>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(incompletePacketsList.error)}
      <ErrorComponent error={incompletePacketsList.error.value}/>
    {/if}
    {#if Option.isSome(incompletePacketsList.data) && Option.isSome(chains.data)}
      {#each incompletePacketsList.data.value as packet, i (i)}
    <PacketListItemComponent {packet} />
      {:else}
        <div class="p-4 text-center text-gray-500">No packets found</div>
      {/each}
    {:else}
      {#if Option.isSome(incompletePacketsList.error)}
        <ErrorComponent error={incompletePacketsList.error.value}/>
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <PacketListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
</Sections>
