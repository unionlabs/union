<script lang="ts">
import { page } from "$app/stores"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { packetDetailsQuery } from "$lib/queries/packets"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
const sequence = $page.params.sequence

const packetDetails = packetDetailsQuery(chain_id, connection_id, channel_id, Number(sequence))
</script>

{#if $packetDetails.data}
  <pre class="text-sm">{JSON.stringify($packetDetails.data.v0_packets, null, 2)}</pre>
{:else}
  <LoadingLogo/>
{/if}

