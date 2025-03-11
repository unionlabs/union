<script lang="ts">
import {
  transferListLatestQuery,
  transferListPageGtQuery,
  transferListPageLtQuery
} from "$lib/queries/transfer-list.svelte"
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
import { flip } from "svelte/animate"
import { fly } from "svelte/transition"
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import TransferListPagination from "$lib/components/ui/TransferListPagination.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { page } from "$app/stores"
import { goto } from "$app/navigation"

import { settingsStore } from "$lib/stores/settings.svelte"
import TransferListItemComponent from "$lib/components/model/TransferListItemComponent.svelte"
import TransferListItemComponentSkeleton from "$lib/components/model/TransferListItemComponentSkeleton.svelte"

let fiber: Fiber.Fiber<any, any>

onMount(() => {
  const pageParam = $page.url.searchParams.get("page")

  if (pageParam) {
    if (pageParam.startsWith("-")) {
      // Greater-than query (prev page)
      const sortOrder = pageParam.substring(1)
      fiber = Effect.runFork(transferListPageGtQuery(sortOrder, settingsStore.pageLimit))
    } else {
      // Less-than query (next page)
      fiber = Effect.runFork(transferListPageLtQuery(pageParam, settingsStore.pageLimit))
    }
  } else {
    // No page param, load latest
    fiber = Effect.runFork(transferListLatestQuery(settingsStore.pageLimit))
  }

  return () => Effect.runPromise(Fiber.interrupt(fiber))
})

const onLive = async () => {
  if (Option.isSome(transferList.data)) {
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListLatestQuery(settingsStore.pageLimit))
    // Remove page param from URL
    goto("?", { replaceState: true })
  }
}

const onPrevPage = async () => {
  if (Option.isSome(transferList.data)) {
    let firstSortOrder = transferList.data.value.at(0)?.sort_order
    if (!firstSortOrder) return
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListPageGtQuery(firstSortOrder, settingsStore.pageLimit))
    // Update URL with the new page param, prefixed with '-' for greater-than queries
    goto(`?page=-${firstSortOrder}`, { replaceState: true })
  }
}

const onNextPage = async () => {
  if (Option.isSome(transferList.data)) {
    let lastSortOrder = transferList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return
    transferList.data = Option.none()
    await Effect.runPromise(Fiber.interrupt(fiber))
    fiber = Effect.runFork(transferListPageLtQuery(lastSortOrder, settingsStore.pageLimit))
    // Update URL with the new page param (no prefix for less-than queries)
    goto(`?page=${lastSortOrder}`, { replaceState: true })
  }
}
</script>

<Sections>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(transferList.data) && Option.isSome(chains.data)}
      {@const chainss = chains.data.value}
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
