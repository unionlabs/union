<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TableTransfers from "../(components)/table-transfers.svelte";
</script>

<ChainsGate let:chains>
  <TableTransfers {chains}/>
</ChainsGate>
