<script lang="ts">
import { BalancesStore, balancesStore } from "$lib/stores/balances.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Card from "$lib/components/ui/Card.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import { Option } from "effect"
import Button from "$lib/components/ui/Button.svelte"
import { Chain, RpcType, UniversalChainId } from "$lib/schema/chain"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { AddressCanonicalBytes, AddressEvmCanonical } from "$lib/schema/address"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import type { Tokens } from "$lib/schema/token"
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte"
import { wallets } from "$lib/stores/wallets.svelte"

function fetchAllBalances() {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    const address = Option.getOrNull(wallets.getAddressForChain(chain))

    if (!address) continue

    // Get tokens for this chain
    const tokens = Option.getOrNull(tokensStore.getData(chain.universal_chain_id))
    if (!tokens) {
      // If we don't have tokens yet, trigger a fetch
      tokensStore.fetchTokens(chain.universal_chain_id)
      continue
    }

    // For each token, fetch its balance
    for (const token of tokens) {
      balancesStore.fetchBalance(chain, address, token.denom)
    }
  }
}

// Start fetching tokens when the page loads
$effect(() => {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    if (chain.rpc_type === "evm" || chain.rpc_type === "cosmos") {
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
      <h3 class="text-lg font-medium mt-4">{chain.universal_chain_id}</h3>
      {#if chain.rpc_type !== "aptos"}
        <div class="flex flex-col">
          
          {#if Option.isNone(sortedBalancesStore.sortedBalances)}
            <Card>
              <div class="text-zinc-500">Loading balances...</div>
            </Card>
          {:else}
            {@const tokensForChain = Option.fromNullable(sortedBalancesStore.sortedBalances.value.find(v => v.chain.universal_chain_id === chain.universal_chain_id)).pipe(Option.flatMap(c => c.tokens))}
            {#if Option.isNone(tokensForChain)}
              <Card>
                <div class="text-zinc-500">No balances found</div>
              </Card>
            {:else}
              {#each tokensForChain.value as { token, balance, error }}
                <div class="flex flex-col gap-2 mb-8">
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
                    ERROR FOR: <TokenComponent 
                        chain={chain} 
                        denom={token.denom} 
                      />
                    <ErrorComponent error={error.value} />
                  {/if}
                </div>
              {/each}
            {/if}
          {/if}
        </div>
      {/if}
      </Card>
    {/each}
  {/if}
</Sections>

