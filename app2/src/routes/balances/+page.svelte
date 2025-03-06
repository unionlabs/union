<script lang="ts">
import { balancesStore } from "$lib/stores/balances.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { Option } from "effect"
import Button from "$lib/components/ui/Button.svelte"
import { RpcType, UniversalChainId } from "$lib/schema/chain"
import { AddressEvmCanonical } from "$lib/schema/address"

// Get all entries from the store
let entries = $derived([...balancesStore.data.entries()])

// Parse the composite key back into its components
function parseKey(key: string) {
  const [universalChainId, address, denom] = key.split(":")
  return { universalChainId, address, denom }
}

// Example wallet address - this would come from wallet connection in real app
const testAddress = AddressEvmCanonical.make(
  "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA".toLowerCase()
)

function fetchAllBalances() {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    // Only fetch for EVM chains for now
    if (chain.rpc_type !== "evm") continue

    // Get tokens for this chain
    const tokens = Option.getOrNull(tokensStore.getData(chain.universal_chain_id))
    if (!tokens) {
      // If we don't have tokens yet, trigger a fetch
      tokensStore.fetchTokens(chain.universal_chain_id)
      continue
    }

    // For each token, fetch its balance
    for (const token of tokens) {
      balancesStore.fetchBalance(chain, testAddress, token.denom)
    }
  }
}

// Start fetching tokens when the page loads
$effect(() => {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    if (chain.rpc_type === "evm") {
      tokensStore.fetchTokens(chain.universal_chain_id)
    }
  }
})
</script>

<div class="flex flex-col gap-4 p-4">
  <div class="flex justify-between items-center">
    <SectionTitle>Balances</SectionTitle>
    <Button onclick={fetchAllBalances}>Fetch All Balances</Button>
  </div>

  {#if Option.isNone(chains.data)}
    <Card>
      <div class="p-4 text-zinc-500">Loading chains...</div>
    </Card>
  {:else if Option.isSome(chains.error)}
    <Card>
      <div class="p-4 text-red-500">Error loading chains</div>
    </Card>
  {:else}
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
  {/if}
</div>

