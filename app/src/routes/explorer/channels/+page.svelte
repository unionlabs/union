<script lang="ts">
import ChainsGate from "$lib/components/chains-gate.svelte"
import TableChannels from "../(components)/table-channels.svelte";
</script>

<ChainsGate let:chains>
  <TableChannels {chains}/>
</ChainsGate>
