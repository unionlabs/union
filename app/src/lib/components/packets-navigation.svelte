<script lang="ts">
import { page } from "$app/stores"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"
import { packetDetailsQuery } from "$lib/queries/packets"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
const sequence = $page.params.sequence
</script>

<h2 class="text-2xl mb-6"><a href={`/explorer/packets/${chain_id}`}>{chain_id}</a>/{#if connection_id}<a href={`/explorer/packets/${chain_id}/${connection_id}`}>{connection_id}</a>/{/if}{#if channel_id}<a href={`/explorer/packets/${chain_id}/${connection_id}/${channel_id}`}>{channel_id}</a>/{/if}{#if sequence}<a href={`/explorer/packets/${chain_id}/${connection_id}/${channel_id}/${sequence}`}>{sequence}</a>{/if}</h2>
