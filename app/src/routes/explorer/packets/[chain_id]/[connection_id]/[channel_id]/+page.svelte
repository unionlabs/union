<script lang="ts">
import { page } from "$app/stores"
import TablePackets from "$lib/components/tables/packets.svelte"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
const channel_id = $page.params.channel_id
</script>

<h2 class="text-2xl mb-6">{chain_id}/{connection_id}/{channel_id}</h2>

<TablePackets {chain_id} {connection_id} {channel_id}/>
