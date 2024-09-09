<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"
</script>

<ChainsGate let:chains>
  <TableTransfers
    {chains}
    pageSize={24}
  />
</ChainsGate>
