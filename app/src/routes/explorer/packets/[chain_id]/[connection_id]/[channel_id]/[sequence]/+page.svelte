<script lang="ts">
import { page } from "$app/stores"
import ChainsGate from "$lib/components/chains-gate.svelte"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import PacketDetails from "$lib/components/packet-details.svelte"
import { packetDetailsQuery } from "$lib/queries/packets"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
const packet_send_transaction_hash = $page.params.sequence

const packetDetails = packetDetailsQuery(
  chain_id,
  Number(connection_id),
  Number(channel_id),
  packet_send_transaction_hash
)
</script>

<ChainsGate let:chains>
  {#if $packetDetails.data}
    {#each $packetDetails.data.v1_ibc_union_packets as packetDetails}
      <PacketDetails {chains} {packetDetails}/>
    {/each}
  {:else}
    <LoadingLogo/>
  {/if}
</ChainsGate>
