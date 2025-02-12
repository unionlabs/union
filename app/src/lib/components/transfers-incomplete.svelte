<script lang="ts">
import type { Chain } from "$lib/types"
import { transfersIncomplete } from "$lib/queries/transfers.ts"
import { createQuery } from "@tanstack/svelte-query"
import LoadingLogo from "./loading-logo.svelte"
import CellOriginTransfer from "$lib/components/table-cells/cell-origin-transfer.svelte"
import CellTimestamp from "$lib/components/table-cells/cell-timestamp.svelte"
import Button from "./ui/button/button.svelte"

export const transfers = createQuery({
  queryKey: ["transfers-incomplete"],
  enabled: true,
  refetchOnMount: true,
  refetchOnReconnect: true,
  refetchInterval: () => 5_000,
  queryFn: async () => await transfersIncomplete()
})
export let chains: Array<Chain>
</script>


{#if $transfers.data}
  <div class="flex flex-col gap-4">
  {#each $transfers.data as transfer}
    <div class="flex flex-col gap-2">
      <div class="flex gap-4">
        <CellOriginTransfer class="w-[180px]" {chains} value={transfer.source}/>
        <CellOriginTransfer class="w-[180px]"  {chains} value={transfer.destination}/>
        <CellTimestamp value={transfer.timestamp}/>
        <Button href={`/explorer/transfers/${transfer.source.hash}`}>Open Transfer</Button>
      </div>
    </div>
  {/each}
  </div>
{:else}
  <LoadingLogo/>
{/if}
