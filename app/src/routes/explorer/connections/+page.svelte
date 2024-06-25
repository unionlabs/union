<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TableConnections from "../(components)/table-connections.svelte";
</script>

<ChainsGate let:chains>
  <TableConnections {chains}/>
</ChainsGate>
