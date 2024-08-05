<script lang="ts">
import { page } from "$app/stores"
import { onNavigate } from "$app/navigation"
import { derived, writable } from "svelte/store"
import DevTools from "$lib/components/dev-tools.svelte"
import { decodeTimestampSearchParam } from "./timestamps.ts"
import TableTransfers from "./(components)/table-transfers.svelte"
import { currentUtcTimestampWithBuffer } from "$lib/utilities/date.ts"
import { createQuery, useQueryClient, keepPreviousData } from "@tanstack/svelte-query"
import { latestTransfers, paginatedAddressesTransfers } from "./paginated-transfers.ts"

/**
 * the timestamp is the source of truth, used as query key and url search param
 */

const QUERY_LIMIT = 6
const REFRESH_INTERVAL = 5_000 // 5 seconds

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : currentUtcTimestampWithBuffer()
)

let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

const queryClient = useQueryClient()

/**
 * only happens when:
 *  1. it is the first query on initial page load with no timestamp search param,
 *  2. the user clicks on the `current` button which resets to current and live data
 */
let REFETCH_ENABLED = writable($page.url.searchParams.has("timestamp") ? false : true)

let liveTransfers = createQuery(
  derived(REFETCH_ENABLED, $REFETCH_ENABLED => ({
    queryKey: ["transfers", "live"],
    staleTime: Number.POSITIVE_INFINITY,
    enabled: $REFETCH_ENABLED,
    refetchOnMount: $REFETCH_ENABLED,
    placeholderData: keepPreviousData,
    refetchOnReconnect: $REFETCH_ENABLED,
    refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
    queryFn: async () => await latestTransfers({ limit: QUERY_LIMIT * 2 })
  }))
)

let transfers = createQuery(
  derived([timestamp, REFETCH_ENABLED], ([$timestamp, $REFETCH_ENABLED]) => ({
    queryKey: ["transfers", $timestamp],
    refetchOnMount: false,
    refetchOnReconnect: false,
    placeholderData: keepPreviousData,
    staleTime: Number.POSITIVE_INFINITY,
    enabled: () => $REFETCH_ENABLED === false,
    queryFn: async () =>
      await paginatedAddressesTransfers({
        timestamp: $timestamp,
        limit: QUERY_LIMIT
      })
  }))
)

let queryStatus: "pending" | "done" = $REFETCH_ENABLED
  ? $liveTransfers.status === "pending" || $liveTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $transfers.status === "pending" || $transfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
$: queryStatus = $REFETCH_ENABLED
  ? $liveTransfers.status === "pending" || $liveTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $transfers.status === "pending" || $transfers.fetchStatus === "fetching"
    ? "pending"
    : "done"

let transfersDataStore = derived(
  [liveTransfers, transfers, REFETCH_ENABLED],
  ([$liveTransfers, $transfers, $REFETCH_ENABLED]) => {
    if ($REFETCH_ENABLED) return $liveTransfers?.data?.transfers ?? []
    return $transfers?.data?.transfers ?? []
  }
)

let timestamps = derived(
  [liveTransfers, transfers, REFETCH_ENABLED],
  ([$liveTransfers, $transfers, $REFETCH_ENABLED]) =>
    $REFETCH_ENABLED
      ? {
          oldestTimestamp: $liveTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $liveTransfers?.data?.latestTimestamp ?? ""
        }
      : {
          oldestTimestamp: $transfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $transfers?.data?.latestTimestamp ?? ""
        }
)

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

<DevTools>
  {JSON.stringify(
    { idx: $pagination.pageIndex, $REFETCH_ENABLED },
    undefined,
    2
  )}
</DevTools>

<TableTransfers
  {timestamp}
  {timestamps}
  {pagination}
  {queryStatus}
  {REFETCH_ENABLED}
  {transfersDataStore}
/>
