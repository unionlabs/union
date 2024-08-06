<script lang="ts">
import { page } from "$app/stores"
import { onNavigate } from "$app/navigation"
import { derived, writable, type Writable } from "svelte/store"
import DevTools from "$lib/components/dev-tools.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { decodeTimestampSearchParam } from "./timestamps.ts"
import TableTransfers from "./(components)/table-transfers.svelte"
import { currentUtcTimestampWithBuffer } from "$lib/utilities/date.ts"
import { createQuery, useQueryClient, keepPreviousData } from "@tanstack/svelte-query"
import { latestTransfers, paginatedAddressesTransfers } from "./paginated-transfers.ts"

/**
 * the timestamp is the source of truth, used as query key and url search param
 */

const QUERY_LIMIT = 6

let timestamp: Writable<string | null> = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : null
)

const queryClient = useQueryClient()

let liveTransfers = createQuery(
  derived([timestamp], ([$timestamp]) =>
    $timestamp
      ? {
          queryKey: ["transfers", $timestamp],
          refetchOnMount: false,
          refetchOnReconnect: false,
          placeholderData: keepPreviousData,
          staleTime: Number.POSITIVE_INFINITY,
          queryFn: async () =>
            await paginatedAddressesTransfers({
              timestamp: $timestamp as string, // otherwise its disabled
              limit: QUERY_LIMIT
            })
        }
      : {
          queryKey: ["transfers", "live"],
          staleTime: Number.POSITIVE_INFINITY,
          refetchOnMount: true,
          placeholderData: keepPreviousData,
          refetchOnReconnect: true,
          refetchInterval: () => 5_000,
          queryFn: async () => await latestTransfers({ limit: QUERY_LIMIT * 2 })
        }
  )
)

let queryStatus: "pending" | "done" =
  $liveTransfers.status === "pending" || $liveTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"

let transfersDataStore = derived([liveTransfers], ([$liveTransfers]) => {
  return $liveTransfers?.data?.transfers ?? []
})

let timestamps = derived([liveTransfers], ([$liveTransfers]) => ({
  oldestTimestamp: $liveTransfers?.data?.oldestTimestamp ?? "",
  latestTimestamp: $liveTransfers?.data?.latestTimestamp ?? ""
}))

/**
 * this can be removed if desired
 * it is only used to clear the cache when navigating away from the page `/explorer/transfers`
 */
onNavigate(navigation => {
  if (navigation.to?.route.id !== "/explorer/transfers") {
    queryClient.removeQueries({ queryKey: ["transfers"] })
  }
})
</script>


<ChainsGate let:chains>
  <TableTransfers
    {chains}
    pageSize={QUERY_LIMIT}
    {timestamp}
    {timestamps}
    {queryStatus}
    {transfersDataStore}
  />
</ChainsGate>
