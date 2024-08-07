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
  getContext<Readable<Array<{ address: string; normalizedAddress: string }>>>("addresses")

let normalizedAddresses = derived(addresses, $addresses =>
  $addresses.map(addr => addr.normalizedAddress)
)
</script>


<ChainsGate let:chains>
  {#each $addresses as address }
    <AddressMultichain {address} {chains}/>
  {/each}
  <TableTransfers
    {chains}
    {timestamp}
    pageSize={24}
    normalizedAddresses={$normalizedAddresses}
  />
</ChainsGate>
