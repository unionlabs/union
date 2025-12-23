<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import PacketListPagination from "$lib/components/ui/PacketListPagination.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import {
  packetListLatestQuery,
  packetListPageGtQuery,
  packetListPageLtQuery,
} from "$lib/queries/packet-list.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { packetList } from "$lib/stores/packets.svelte"
import { Effect, Option } from "effect"
import { onMount } from "svelte"

import PacketListItemComponent from "$lib/components/model/PacketListItemComponent.svelte"
import PacketListItemComponentSkeleton from "$lib/components/model/PacketListItemComponentSkeleton.svelte"
import Switch from "$lib/components/ui/Switch.svelte"
import type { AppContext } from "$lib/runtime"
import { settingsStore } from "$lib/stores/settings.svelte"

const initializeQuery = async () => {
  const pageParam = page.url.searchParams.get("page")
  // XXX: no mutable effects
  let effect: Effect.Effect<any, never, AppContext>

  if (pageParam) {
    if (pageParam.startsWith("-")) {
      // Greater-than query (prev page)
      const sortOrder = pageParam.substring(1)
      // @ts-ignore sortOrder is not strictly a SortOrder, but this is desired behavior
      effect = packetListPageGtQuery(sortOrder, settingsStore.pageLimit, settingsStore.mainnetOnly)
    } else {
      // Less-than query (next page)
      // @ts-ignore pageParam is not strictly a SortOrder, but this is desired behavior
      effect = packetListPageLtQuery(pageParam, settingsStore.pageLimit, settingsStore.mainnetOnly)
    }
  } else {
    // No page param, load latest
    effect = packetListLatestQuery(settingsStore.pageLimit, settingsStore.mainnetOnly)
  }

  await packetList.runEffect(effect)
}

onMount(() => {
  initializeQuery()

  return () => {
    packetList.interruptFiber()
  }
})

const onLive = async () => {
  if (Option.isSome(packetList.data)) {
    await packetList.runEffect(
      packetListLatestQuery(settingsStore.pageLimit, settingsStore.mainnetOnly),
    )
    // Remove page param from URL
    goto("?", { replaceState: true })
  }
}

const onPrevPage = async () => {
  if (Option.isSome(packetList.data)) {
    let firstSortOrder = packetList.data.value.at(0)?.sort_order
    if (!firstSortOrder) {
      return
    }
    await packetList.runEffect(
      packetListPageGtQuery(firstSortOrder, settingsStore.pageLimit, settingsStore.mainnetOnly),
    )
    // Update URL with the new page param, prefixed with '-' for greater-than queries
    goto(`?page=-${firstSortOrder}`, { replaceState: true })
  }
}

const onNextPage = async () => {
  if (Option.isSome(packetList.data)) {
    let lastSortOrder = packetList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) {
      return
    }
    await packetList.runEffect(
      packetListPageLtQuery(lastSortOrder, settingsStore.pageLimit, settingsStore.mainnetOnly),
    )
    // Update URL with the new page param (no prefix for less-than queries)
    goto(`?page=${lastSortOrder}`, { replaceState: true })
  }
}
</script>

<Sections>
  <Card
    class="overflow-auto"
    divided
  >
    {#if Option.isSome(packetList.data) && Option.isSome(chains.data)}
      {#if Option.isSome(packetList.error)}
        <ErrorComponent error={packetList.error.value} />
      {/if}
      {#each packetList.data.value as packet (packet.sort_order)}
        <PacketListItemComponent {packet} />
      {:else}
        <div class="p-4 text-center text-zinc-500">No packets found</div>
      {/each}
    {:else}
      {#if Option.isSome(packetList.error)}
        <ErrorComponent error={packetList.error.value} />
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <PacketListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
  <div class="flex flex-col sm:flex-row sm:items-center gap-4">
    <PacketListPagination
      data={packetList.data}
      {onLive}
      {onPrevPage}
      {onNextPage}
    />
    <div class="flex items-center gap-2">
      <Switch
        checked={settingsStore.mainnetOnly}
        label="Mainnet Only"
        change={(value) => {
          settingsStore.mainnetOnly = value
          initializeQuery()
        }}
      />
    </div>
  </div>
</Sections>
