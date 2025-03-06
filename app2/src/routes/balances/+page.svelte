<script lang="ts">
  import { balancesStore } from "$lib/stores/balances.svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
  import { Option } from "effect"

  // Get all entries from the store
  let entries = $derived([...balancesStore.data.entries()])

  // Parse the composite key back into its components
  function parseKey(key: string) {
    const [universalChainId, address, denom] = key.split(":")
    return { universalChainId, address, denom }
  }
</script>

<div class="flex flex-col gap-4 p-4">
  <SectionTitle>Balances</SectionTitle>

  {#if entries.length === 0}
    <Card>
      <div class="p-4 text-zinc-500">No balances found</div>
    </Card>
  {:else}
    {#each entries as [key, balance]}
      {@const { universalChainId, address, denom } = parseKey(key)}
      <Card>
        <div class="p-4 flex flex-col gap-2">
          <div class="text-sm text-zinc-500">Chain: {universalChainId}</div>
          <div class="text-sm text-zinc-500">Address: {address}</div>
          <div class="text-sm text-zinc-500">Token: {denom}</div>
          <div class="font-medium">
            Balance: {Option.getOrNull(balance)}
          </div>
          
          {#if Option.isSome(balancesStore.getError(universalChainId, address, denom))}
            <div class="text-red-500 text-sm">
              Error loading balance
            </div>
          {/if}
        </div>
      </Card>
    {/each}
  {/if}
</div>
