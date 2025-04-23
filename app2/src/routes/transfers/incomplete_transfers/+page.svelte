<script lang="ts">

import {
  runInWindowAllPairs
}  from "$lib/queries/missing-transfer-list.svelte"
import { Effect, Option } from "effect"
import { onMount } from "svelte"
import { incompleteTransferList } from "$lib/stores/incomplete-transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"

import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"

onMount(() => {


  runInWindowAllPairs()

  return () => {
    incompleteTransferList.interruptFiber()
  }
})

</script>

<Sections>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(incompleteTransferList.error)}
      <ErrorComponent error={incompleteTransferList.error.value}/>
    {/if}
    {#if Option.isSome(incompleteTransferList.data) && Option.isSome(chains.data)}
      {#each incompleteTransferList.data.value as transfer, i (i)}
    <TransferListItemComponent {transfer} />
      {:else}
        <div class="p-4 text-center text-gray-500">No transfers found</div>
      {/each}
    {:else}
      {#if Option.isSome(incompleteTransferList.error)}
        <ErrorComponent error={incompleteTransferList.error.value}/>
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <TransferListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
</Sections>
