<script lang="ts">
import { balancesStore } from "$lib/stores/balances.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { Option } from "effect"
import Button from "$lib/components/ui/Button.svelte"
import { RpcType, UniversalChainId } from "$lib/schema/chain"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { AddressEvmCanonical } from "$lib/schema/address"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Sections from "$lib/components/ui/Sections.svelte"

// Example wallet address - this would come from wallet connection in real app
const testAddress = AddressEvmCanonical.make("0xe6831e169d77a861a0e71326afa6d80bcc8bc6aa")

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

<Sections>
  <div class="flex justify-between items-center">
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
    {#each Option.getOrNull(chains.data) ?? [] as chain}
      <Card>
      {#if chain.rpc_type === "evm"}
        {@const tokens = Option.getOrNull(tokensStore.getData(chain.universal_chain_id))}
        <div class="flex flex-col">
          <h3 class="text-lg font-medium mt-4">{chain.universal_chain_id}</h3>
          
          {#if !tokens}
            <Card>
              <div class=" text-zinc-500">Loading tokens...</div>
            </Card>
          {:else if tokens.length === 0}
            <Card>
              <div class="text-zinc-500">No tokens found</div>
            </Card>
          {:else}
            {#each tokens as token}
              {@const balance = balancesStore.getBalance(chain.universal_chain_id, testAddress, token.denom)}
              {@const error = balancesStore.getError(chain.universal_chain_id, testAddress, token.denom)}
                <div class="flex flex-col gap-2 mb-8 ">
                  {#if Option.isSome(balance)}
                    <TokenComponent 
                      chain={chain} 
                      denom={token.denom} 
                      amount={balance.value} 
                    />
                  {:else}
                  <div class="text-red-500 font-bold">
                    NO BALANCE FOR: <TokenComponent 
                      chain={chain} 
                      denom={token.denom} 
                    />
                  </div>
                  {/if}
                  {#if Option.isSome(error)}
                    <ErrorComponent error={error.value} />
                  {/if}
                </div>
            {/each}
          {/if}
        </div>
      {/if}
      </Card>
    {/each}
  {/if}
</Sections>

