<script lang="ts">
import { page } from "$app/stores"
import { getContext } from "svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { derived, writable, type Readable } from "svelte/store"
import { decodeTimestampSearchParam } from "../../timestamps.ts"
import TableTransfers from "../../(components)/table-transfers.svelte"

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : null
)

let addressArray =
  getContext<Readable<{ nonNormalized: Array<string>; normalized: Array<string> }>>("addressArray")

let normalizedAddresses = derived(addressArray, $addressArray => $addressArray.normalized)
</script>


<ChainsGate let:chains>
  <TableTransfers
    {chains}
    {timestamp}
    pageSize={24}
    normalizedAddresses={$normalizedAddresses}
  />
</ChainsGate>
