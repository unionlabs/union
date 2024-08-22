<script lang="ts">
import { page } from "$app/stores"
import PacketsNavigation from "$lib/components/packets-navigation.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
</script>

<PacketsNavigation/>

<TablePackets {chain_id} {connection_id}/>
