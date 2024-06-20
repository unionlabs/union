<script lang="ts">
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types.ts"
import Precise from "$lib/components/precise.svelte"

export let chains: Array<Chain>
export let userAddr: UserAddresses

$: userBalances = userBalancesQuery({
  userAddr: userAddr,
  chains: chains.filter(c => c.chain_id === "union-testnet-8")
})
$: chain = chains.find(c => c.chain_id === "union-testnet-8")
$: unionBalances = $userBalances.at(0)?.data ?? []
$: asset = unionBalances.find(balance => balance.symbol.toLowerCase() === "muno")
</script>

{#if asset && chain}
  <Precise {asset} {chain} toolTip showSymbol />
{/if}
