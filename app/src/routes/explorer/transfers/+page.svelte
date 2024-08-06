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

const QUERY_LIMIT = 6

let timestamp: Writable<string | null> = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : null
)
</script>


<ChainsGate let:chains>
  <TableTransfers
    {chains}
    {timestamp}
    pageSize={QUERY_LIMIT}
  />
</ChainsGate>
