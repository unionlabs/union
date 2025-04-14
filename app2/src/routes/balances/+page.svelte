<script lang="ts">
  import {balancesStore} from "$lib/stores/balances.svelte"
  import {chains} from "$lib/stores/chains.svelte"
  import {tokensStore} from "$lib/stores/tokens.svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import {Option} from "effect"
  import Button from "$lib/components/ui/Button.svelte"
  import TokenComponent from "$lib/components/model/TokenComponent.svelte"
  import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
  import Sections from "$lib/components/ui/Sections.svelte"
  import {sortedBalancesStore} from "$lib/stores/sorted-balances.svelte"
  import {wallets} from "$lib/stores/wallets.svelte"
  import {uiStore} from "$lib/stores/ui.svelte"
  import ChainComponent from "$lib/components/model/ChainComponent.svelte"

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

    // Fetch all token balances for this chain in a batch
    const denoms = tokens.map(token => token.denom)
    balancesStore.fetchBalances(chain, address, denoms)
  }
}

// Start fetching tokens when the page loads
$effect(() => {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    tokensStore.fetchTokens(chain.universal_chain_id)
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
  {:else}
    {#each Option.getOrNull(chains.data) ?? [] as chain}
      <Card divided>
        <section class="p-4">
          <ChainComponent {chain}/>
        </section>
        <section class="p-4">
          <div class="flex flex-col">
            {#if Option.isNone(sortedBalancesStore.sortedBalances)}
              <div class="text-zinc-500">Loading balances...</div>
            {:else}
              {@const tokensForChain = Option.fromNullable(sortedBalancesStore.sortedBalances.value.find(v => v.chain.universal_chain_id === chain.universal_chain_id)).pipe(Option.flatMap(c => c.tokens))}
              {#if Option.isNone(tokensForChain)}
                <div class="text-zinc-500">No balances found</div>
              {:else}
                {#each tokensForChain.value.filter(t => 
                  Option.isSome(t.error) || 
                  Option.isNone(t.balance) || 
                  uiStore.showZeroBalances || 
                  t.numericValue > 0n
                ) as { token, balance, error }}
                  <div class="flex flex-col gap-2">
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
        </section>
      </Card>
    {/each}
  {/if}
</Sections>

