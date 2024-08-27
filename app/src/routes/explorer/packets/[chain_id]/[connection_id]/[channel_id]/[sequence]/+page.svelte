<script lang="ts">
import { page } from "$app/stores"
import ChainsGate from "$lib/components/chains-gate.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import PacketDetails from "$lib/components/packet-details.svelte"
import { packetDetailsQuery } from "$lib/queries/packets"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
const sequence = $page.params.sequence

const packetDetails = packetDetailsQuery(chain_id, connection_id, channel_id, Number(sequence))
</script>

<ChainsGate let:chains>
  {#if $packetDetails.data}
    {#each $packetDetails.data.v0_packets as packetDetails}
      <PacketDetails {chains} {packetDetails}/>
    {/each}
  {:else}
    <LoadingLogo/>
  {/if}
</ChainsGate>
