<script lang="ts">
import { page } from "$app/stores"
import TablePackets from "$lib/components/tables/packets.svelte"

const chain_id = $page.params.chain_id
</script>

<h2 class="text-2xl mb-6">{chain_id}</h2>

<TablePackets {chain_id}/>
