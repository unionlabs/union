<script lang="ts">
import { page } from "$app/stores"
import { getContext } from "svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { derived, writable, type Readable } from "svelte/store"
import { decodeTimestampSearchParam } from "$lib/timestamps.ts"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"
import AddressMultichain from "$lib/components/address-multichain.svelte"
import StatsBar from "../../(components)/stats-bar.svelte"

let addresses =
  getContext<Readable<Array<{ address: string; normalizedAddress: string }>>>("addresses")

let normalizedAddresses = derived(addresses, $addresses =>
  $addresses.map(addr => addr.normalizedAddress)
)
</script>


<ChainsGate let:chains>
  <div class="-m-6 mb-6">
    <StatsBar addresses={$normalizedAddresses} />
  </div>
  <section class="flex gap-4 flex-col items-center pb-6">
    {#each $addresses as address }
      <AddressMultichain {address} {chains}/>
    {/each}
  </section>
  <TableTransfers
    {chains}
    pageSize={24}
    normalizedAddresses={$normalizedAddresses}
  />
</ChainsGate>
