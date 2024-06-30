<script lang="ts">
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types.ts"
import Precise from "$lib/components/precise.svelte"

export let chains: Array<Chain>
export let userAddr: UserAddresses
export let connected: boolean

let chain = chains.filter(c => c.chain_id === "union-testnet-8")
$: userBalances = userBalancesQuery({
  userAddr: userAddr,
  chains: chain,
  connected
})
$: unionBalances = $userBalances.at(0)?.data ?? []
$: asset = unionBalances.find(balance => balance.symbol.toLowerCase() === "muno")
</script>

{#if asset}
  <span class=""><Precise chain={chain[0]} {asset} showSymbol displayDecimals={6} /></span>
{/if}
