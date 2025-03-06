<script lang="ts">
import { balancesStore } from "$lib/stores/balances.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { Option } from "effect"
import Button from "$lib/components/ui/Button.svelte"
import { RpcType, UniversalChainId } from "$lib/schema/chain"

// Get all entries from the store
let entries = $derived([...balancesStore.data.entries()])

// Parse the composite key back into its components
function parseKey(key: string) {
  const [universalChainId, address, denom] = key.split(":")
  return { universalChainId, address, denom }
}

// Example data for testing
const testData = {
  chain: {
    universal_chain_id: UniversalChainId.make("sepolia.11155111"),
    rpc_type: "evm"
  },
  address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".toLowerCase(),
  denom: "0x779877A7B0D9E8603169DdbD7836e478b4624789" // LINK token on Sepolia
}

function fetchTestBalance() {
  balancesStore.fetchBalance(testData.chain, testData.address, testData.denom)
}
</script>

<div class="flex flex-col gap-4 p-4">
  <div class="flex justify-between items-center">
    <SectionTitle>Balances</SectionTitle>
    <Button onclick={fetchTestBalance}>Fetch Test Balance</Button>
  </div>

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
