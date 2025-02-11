<script lang="ts">
import { page } from "$app/stores"
import ChainsGate from "$lib/components/chains-gate.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"
const chain_id = $page.params.chain_id
</script>

<ChainsGate let:chains>
  <TablePackets {chains}  {chain_id}/>
</ChainsGate>
