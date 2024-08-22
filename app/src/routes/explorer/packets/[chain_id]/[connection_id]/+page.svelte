<script lang="ts">
import { page } from "$app/stores"
import TablePackets from "$lib/components/tables/packets.svelte"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
</script>

<TablePackets {chain_id} {connection_id}/>
