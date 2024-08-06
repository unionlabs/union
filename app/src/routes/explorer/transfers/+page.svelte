<script lang="ts">
import { page } from "$app/stores"
import { derived, writable, type Writable } from "svelte/store"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { decodeTimestampSearchParam } from "./timestamps.ts"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"

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
    pageSize={24}
  />
</ChainsGate>
