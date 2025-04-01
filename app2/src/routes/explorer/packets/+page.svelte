
<script lang="ts">
import {
  packetListLatestQuery,
  packetListPageGtQuery,
  packetListPageLtQuery
} from "$lib/queries/packet-list.svelte"
import { Effect, Schema, Option } from "effect"
import { onMount } from "svelte"
import { packetList } from "$lib/stores/packets.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import PacketListPagination from "$lib/components/ui/PacketListPagination.svelte"
import { page } from "$app/state"
import { goto } from "$app/navigation"
import { SortOrder } from "@unionlabs/sdk/schema"

import { settingsStore } from "$lib/stores/settings.svelte"
import PacketListItemComponent from "$lib/components/model/PacketListItemComponent.svelte"
import PacketListItemComponentSkeleton from "$lib/components/model/PacketListItemComponentSkeleton.svelte"

onMount(() => {
  const pageParam = page.url.searchParams.get("page")

  const initializeQuery = async () => {
    let effect: Effect.Effect<any>

    if (pageParam) {
      try {
        if (pageParam.startsWith("-")) {
          // Greater-than query (prev page)
          const rawSortOrder = pageParam.substring(1)
          // Validate that the sort order is valid
          const parsedSortOrder = Schema.decodeSync(SortOrder)(rawSortOrder, {
            errors: "all",
            onExcessProperty: "ignore"
          })

          effect = packetListPageGtQuery(parsedSortOrder, settingsStore.pageLimit)
        } else {
          // Less-than query (next page)
          // Validate that the sort order is valid
          const parsedSortOrder = Schema.decodeSync(SortOrder)(pageParam, {
            errors: "all",
            onExcessProperty: "ignore"
          })

          effect = packetListPageLtQuery(parsedSortOrder, settingsStore.pageLimit)
        }
      } catch (error) {
        console.error("Invalid sort order in URL:", error)
        // Fall back to latest if the sort order is invalid
        effect = packetListLatestQuery(settingsStore.pageLimit)
        // Remove invalid page param from URL
        goto("?", { replaceState: true })
      }
    } else {
      // No page param, load latest
      effect = packetListLatestQuery(settingsStore.pageLimit)
    }

    await packetList.runEffect(effect)
  }

  initializeQuery()

  return () => {
    packetList.interruptFiber()
  }
})

const onLive = async () => {
  if (Option.isSome(packetList.data)) {
    await packetList.runEffect(packetListLatestQuery(settingsStore.pageLimit))
    // Remove page param from URL
    goto("?", { replaceState: true })
  }
}

const onPrevPage = async () => {
  if (Option.isSome(packetList.data)) {
    let firstSortOrder = packetList.data.value.at(0)?.sort_order
    if (!firstSortOrder) return

    // Validate that the sort order is valid
    try {
      const parsedSortOrder = Schema.decodeSync(SortOrder)(firstSortOrder, {
        errors: "all",
        onExcessProperty: "ignore"
      })

      await packetList.runEffect(packetListPageGtQuery(parsedSortOrder, settingsStore.pageLimit))
      // Update URL with the new page param, prefixed with '-' for greater-than queries
      goto(`?page=-${parsedSortOrder}`, { replaceState: true })
    } catch (error) {
      console.error("Invalid sort order:", error)
    }
  }
}

const onNextPage = async () => {
  if (Option.isSome(packetList.data)) {
    let lastSortOrder = packetList.data.value.at(-1)?.sort_order
    if (!lastSortOrder) return

    // Validate that the sort order is valid
    try {
      const parsedSortOrder = Schema.decodeSync(SortOrder)(lastSortOrder, {
        errors: "all",
        onExcessProperty: "ignore"
      })

      await packetList.runEffect(packetListPageLtQuery(parsedSortOrder, settingsStore.pageLimit))
      // Update URL with the new page param (no prefix for less-than queries)
      goto(`?page=${parsedSortOrder}`, { replaceState: true })
    } catch (error) {
      console.error("Invalid sort order:", error)
    }
  }
}
</script>

<Sections>
  <Card class="overflow-auto" divided>
    {#if Option.isSome(packetList.data) && Option.isSome(chains.data)}
      {#if Option.isSome(packetList.error)}
        <ErrorComponent error={packetList.error.value}/>
      {/if}
      {#each packetList.data.value as packet(packet.sort_order)}
        <PacketListItemComponent {packet} />
      {:else}
        <div class="p-4 text-center text-zinc-500">No packets found</div>
      {/each}
    {:else}
      {#if Option.isSome(packetList.error)}
        <ErrorComponent error={packetList.error.value}/>
      {/if}
      {#each Array(settingsStore.pageLimit).fill(0)}
        <PacketListItemComponentSkeleton />
      {/each}
    {/if}
  </Card>
  <PacketListPagination 
    data={packetList.data}
    {onLive}
    {onPrevPage}
    {onNextPage}
  />
</Sections>
