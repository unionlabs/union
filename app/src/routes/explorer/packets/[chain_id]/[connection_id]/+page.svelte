<script lang="ts">
import { page } from "$app/stores"
import TablePackets from "$lib/components/tables/packets.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"

const chain_id = $page.params.chain_id
const connection_id = $page.params.connection_id
</script>

<ChainsGate let:chains>
  <TablePackets {chains} {chain_id} {connection_id}/>
</ChainsGate>
