<script lang="ts">
import { page } from "$app/stores"
import { derived, writable, type Writable } from "svelte/store"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { decodeTimestampSearchParam } from "./timestamps.ts"
import TableTransfers from "$lib/components/transfers-table/transfers-table.svelte"
</script>

<ChainsGate let:chains>
  <TableTransfers
    {chains}
    pageSize={24}
  />
</ChainsGate>
