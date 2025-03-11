<script lang="ts">
import {
  transferListLatestQuery,
  transferListPageGtQuery,
  transferListPageLtQuery
} from "$lib/queries/transfer-list.svelte"
import { Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import TransferListPagination from "$lib/components/ui/TransferListPagination.svelte"
import { page } from "$app/state"
import { goto } from "$app/navigation"

import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"

onMount(() => {
  const pageParam = page.url.searchParams.get("page")

  const initializeQuery = async () => {
    let effect: Effect.Effect

    if (pageParam) {
      if (pageParam.startsWith("-")) {
        // Greater-than query (prev page)
        const sortOrder = pageParam.substring(1)
        effect = transferListPageGtQuery(sortOrder, settingsStore.pageLimit)
      } else {
        // Less-than query (next page)
        effect = transferListPageLtQuery(pageParam, settingsStore.pageLimit)
      }
    } else {
      // No page param, load latest
      effect = transferListLatestQuery(settingsStore.pageLimit)
    }

    await transferList.runEffect(effect)
  }

  initializeQuery()

  return () => {
    transferList.interruptFiber()
  }
})

const onLive = async () => {
  if (Option.isSome(transferList.data)) {
    await transferList.runEffect(transferListLatestQuery(settingsStore.pageLimit))
    // Remove page param from URL
    goto("?", { replaceState: true })
  }
}

const onPrevPage = async () => {
  if (Option.isSome(transferList.data)) {
    let firstSortOrder = transferList.data.value.at(0)?.sort_order
    if (!firstSortOrder) return
    await transferList.runEffect(transferListPageGtQuery(firstSortOrder, settingsStore.pageLimit))
    // Update URL with the new page param, prefixed with '-' for greater-than queries
    goto(`?page=-${firstSortOrder}`, { replaceState: true })
  }
}

const onNextPage = async () => {
  if (Option.isSome(transferList.data)) {
    let lastSortOrder = transferList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    await transferList.runEffect(transferListPageLtQuery(lastSortOrder, settingsStore.pageLimit))
    // Update URL with the new page param (no prefix for less-than queries)
    goto(`?page=${lastSortOrder}`, { replaceState: true })
  }
}
</script>

<Sections>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(transferList.data) && Option.isSome(chains.data)}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
      {#each transferList.data.value as transfer(transfer.sort_order)}
        <TransferListItemComponent {transfer} />
      {:else}
        <div class="p-4 text-center text-gray-500">No transfers found</div>
      {/each}
    {:else}
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <TransferListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
  <TransferListPagination 
    data={transferList.data}
    {onLive}
    {onPrevPage}
    {onNextPage}
  />
</Sections>
