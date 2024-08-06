<script lang="ts">
import { page } from "$app/stores"
import { getContext } from "svelte"
import { onNavigate } from "$app/navigation"
import DevTools from "$lib/components/dev-tools.svelte"
import { derived, writable, type Readable } from "svelte/store"
import { addressTransfersPreference } from "../../preference.ts"
import { decodeTimestampSearchParam } from "../../timestamps.ts"
import { currentUtcTimestampWithBuffer } from "$lib/utilities/date.ts"
import TableTransfers from "../../(components)/table-transfers.svelte"
import { createQuery, useQueryClient, keepPreviousData } from "@tanstack/svelte-query"
import { latestAddressesTransfers, paginatedAddressesTransfers } from "./paginated-transfers.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"

addressTransfersPreference.useLocalStorage()

let QUERY_LIMIT = 10
let REFRESH_INTERVAL = 5_000

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : currentUtcTimestampWithBuffer()
)

let pagination = writable({ pageIndex: 0, pageSize: QUERY_LIMIT })

const queryClient = useQueryClient()

let addressArray =
  getContext<Readable<{ nonNormalized: Array<string>; normalized: Array<string> }>>("addressArray")

let nonNormalizedAddressArray = $addressArray.nonNormalized

let normalizedAddressArray = derived(addressArray, $addressArray => $addressArray.normalized)
/**
 * only happens when:
 *  1. it is the first query on initial page load with no timestamp search param,
 *  2. the user clicks on the `current` button which resets to current and live data
 */
let REFETCH_ENABLED = writable($page.url.searchParams.has("timestamp") ? false : true)

let liveAddressTransfers = createQuery(
  derived(
    [REFETCH_ENABLED, normalizedAddressArray],
    ([$REFETCH_ENABLED, $normalizedAddressArray]) => ({
      queryKey: ["address-transfers", "live", ...$normalizedAddressArray],
      refetchOnMount: $REFETCH_ENABLED,
      placeholderData: keepPreviousData,
      staleTime: Number.POSITIVE_INFINITY,
      refetchOnReconnect: $REFETCH_ENABLED,
      enabled: $REFETCH_ENABLED,
      refetchInterval: () => ($REFETCH_ENABLED ? REFRESH_INTERVAL : false),
      queryFn: async () =>
        await latestAddressesTransfers({
          limit: QUERY_LIMIT * 2,
          addresses: $normalizedAddressArray
        })
    })
  )
)

let addressTransfers = createQuery(
  derived(
    [timestamp, normalizedAddressArray, REFETCH_ENABLED],
    ([$timestamp, $normalizedAddressArray, $REFETCH_ENABLED]) => ({
      queryKey: ["address-transfers", $timestamp, ...$normalizedAddressArray],
      refetchOnMount: false,
      refetchOnReconnect: false,
      placeholderData: keepPreviousData,
      staleTime: Number.POSITIVE_INFINITY,
      enabled: () => $REFETCH_ENABLED === false,
      queryFn: async () =>
        await paginatedAddressesTransfers({
          limit: QUERY_LIMIT,
          timestamp: $timestamp,
          addresses: $normalizedAddressArray
        })
    })
  )
)

let queryStatus: "pending" | "done" = $REFETCH_ENABLED
  ? $liveAddressTransfers.status === "pending" || $liveAddressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $addressTransfers.status === "pending" || $addressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
$: queryStatus = $REFETCH_ENABLED
  ? $liveAddressTransfers.status === "pending" || $liveAddressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"
  : $addressTransfers.status === "pending" || $addressTransfers.fetchStatus === "fetching"
    ? "pending"
    : "done"

let transfersDataStore = derived(
  [liveAddressTransfers, addressTransfers, REFETCH_ENABLED],
  ([$liveAddressTransfers, $addressTransfers, $REFETCH_ENABLED]) => {
    if ($REFETCH_ENABLED) return $liveAddressTransfers?.data?.transfers ?? []
    return $addressTransfers?.data?.transfers ?? []
  }
)

let timestamps = derived(
  [liveAddressTransfers, addressTransfers, REFETCH_ENABLED],
  ([$liveAddressTransfers, $addressTransfers, $REFETCH_ENABLED]) =>
    $REFETCH_ENABLED
      ? {
          oldestTimestamp: $liveAddressTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $liveAddressTransfers?.data?.latestTimestamp ?? ""
        }
      : {
          oldestTimestamp: $addressTransfers?.data?.oldestTimestamp ?? "",
          latestTimestamp: $addressTransfers?.data?.latestTimestamp ?? ""
        }
)

/**
 * this can be removed if desired
 * it is only used to clear the cache when navigating away from the page `/explorer/transfers`
 */
onNavigate(navigation => {
  if (!navigation.to?.route.id?.startsWith("/explorer/transfers/address")) {
    queryClient.removeQueries({ queryKey: ["address-transfers"] })
  }
})
</script>

<DevTools>
  <!-- <pre>
    {JSON.stringify(
      {
        idx: $pagination.pageIndex,
        $REFETCH_ENABLED,
        ...$timestamps,
        ...$addressArray
      },
      undefined,
      2
    )}
  </pre> -->
</DevTools>

<ChainsGate let:chains>
  <TableTransfers
    {chains}
    {timestamp}
    {timestamps}
    {pagination}
    {queryStatus}
    {REFETCH_ENABLED}
    {transfersDataStore}
  />
</ChainsGate>
