<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"

console.info("transfers")
</script>

<ChainsGate {chains}>
  {#snippet children({ chains })}
    <TableTransfers
      {chains}
      pageSize={24}
    />
  {/snippet}
</ChainsGate>
