<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TablePackets from "$lib/components/tables/packets.svelte"
</script>

<ChainsGate let:chains>
  <TablePackets {chains} />
</ChainsGate>
