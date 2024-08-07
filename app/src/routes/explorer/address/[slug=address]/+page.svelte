<script lang="ts">
import { page } from "$app/stores"
import { getContext } from "svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { derived, writable, type Readable } from "svelte/store"
import { decodeTimestampSearchParam } from "$lib/timestamps.ts"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"
import AddressMultichain from "$lib/components/address-multichain.svelte"

let timestamp = writable(
  $page.url.searchParams.has("timestamp")
    ? decodeTimestampSearchParam(`${$page.url.searchParams.get("timestamp")}`)
    : null
)

let addresses =
  getContext<Readable<{ nonNormalized: Array<string>; normalized: Array<string> }>>("addressArray")

let normalizedAddresses = derived(addresses, $addresses => $addresses.normalized)
</script>


<ChainsGate let:chains>
  {#each $normalizedAddresses as normalizedAddress}
    <AddressMultichain {chains} {normalizedAddress}/>
  {/each}
  <TableTransfers
    {chains}
    {timestamp}
    pageSize={24}
    normalizedAddresses={$normalizedAddresses}
  />
</ChainsGate>
