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

// Example wallet address - this would come from wallet connection in real app
const testAddress = AddressEvmCanonical.make("0xe6831e169d77a861a0e71326afa6d80bcc8bc6aa")

const getSortedTokens = (
  tokens: Tokens,
  chain: Chain,
  bs: BalancesStore,
  address: AddressCanonicalBytes
) =>
  tokens
    .map(token => {
      const balance = bs.getBalance(chain.universal_chain_id, address, token.denom)
      const error = bs.getError(chain.universal_chain_id, address, token.denom)
      const tokenInfo = tokensStore
        .getData(chain.universal_chain_id)
        .pipe(
          Option.flatMap(tokens => Option.fromNullable(tokens.find(t => t.denom === token.denom)))
        )

      // Get decimals from token info
      const decimals =
        Option.getOrNull(
          Option.flatMap(tokenInfo, t => Option.fromNullable(t.representations[0]?.decimals))
        ) ?? 18 // Default to 18 if not found

      // Calculate numeric value for sorting
      const numericValue = Option.match(balance, {
        onNone: () => -1n,
        onSome: bal =>
          Option.match(Option.fromNullable(bal), {
            onNone: () => 0n,
            onSome: val => val
          })
      })

      return {
        token,
        balance,
        error,
        numericValue,
        decimals
      }
    })
    .sort((a, b) => {
      // First, separate by status
      if (Option.isSome(a.error) && !Option.isSome(b.error)) return 1
      if (!Option.isSome(a.error) && Option.isSome(b.error)) return -1

      if (Option.isNone(a.balance) && Option.isSome(b.balance)) return 1
      if (Option.isSome(a.balance) && Option.isNone(b.balance)) return -1

      // Then sort by value
      if (a.numericValue === -1n && b.numericValue !== -1n) return 1
      if (a.numericValue !== -1n && b.numericValue === -1n) return -1

      if (a.numericValue === 0n && b.numericValue > 0n) return 1
      if (a.numericValue > 0n && b.numericValue === 0n) return -1

      // Sort by actual value if both have balances
      if (a.numericValue > 0n && b.numericValue > 0n) {
        // Adjust for decimals
        const aAdjusted = a.numericValue / 10n ** BigInt(a.decimals)
        const bAdjusted = b.numericValue / 10n ** BigInt(b.decimals)
        return aAdjusted < bAdjusted ? 1 : -1
      }

      return 0
    })

const sortedBalances = $derived(
  chains.data.pipe(
    Option.map(d =>
      d.map(chain => ({
        chain,
        tokens: tokensStore
          .getData(chain.universal_chain_id)
          .pipe(Option.map(ts => getSortedTokens(ts, chain, balancesStore, testAddress)))
      }))
    )
  )
)

function fetchAllBalances() {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  for (const chain of chainsData) {
    // Only fetch for EVM chains for now
    if (chain.rpc_type === "aptos") continue

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
          
          {#if Option.isNone(sortedBalances)}
            <Card>
              <div class="text-zinc-500">Loading balances...</div>
            </Card>
          {:else}
            {@const tokensForChain = Option.fromNullable(sortedBalances.value.find(v => v.chain.universal_chain_id === chain.universal_chain_id)).pipe(Option.flatMap(c => c.tokens))}
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

