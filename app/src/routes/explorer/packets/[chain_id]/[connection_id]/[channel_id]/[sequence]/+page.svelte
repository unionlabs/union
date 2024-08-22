<script lang="ts">
import { page } from "$app/stores"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"
import { packetDetailsQuery } from "$lib/queries/packets"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
const sequence = $page.params.sequence

const packetDetails = packetDetailsQuery(chain_id, connection_id, channel_id, Number(sequence))
</script>

<h2 class="text-2xl mb-6">{chain_id}/{connection_id}/{channel_id}/{sequence}</h2>

{#if $packetDetails.data}
  <pre class="text-sm">{JSON.stringify($packetDetails.data.v0_packets, null, 2)}</pre>
{:else}
  <LoadingLogo/>
{/if}

