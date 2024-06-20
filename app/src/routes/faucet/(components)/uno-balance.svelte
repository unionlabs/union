<script lang="ts">
import { formatUnits } from "viem"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types.ts"

export let chains: Array<Chain>
export let userAddr: UserAddresses

$: userBalances = userBalancesQuery({
  userAddr: userAddr,
  chains: chains.filter(c => c.chain_id === "union-testnet-8")
})
$: unionBalances = $userBalances.at(0)?.data ?? []
$: munoBalance = unionBalances.find(balance => balance.symbol.toLowerCase() === "muno")
</script>

{#if munoBalance}
  <span class="">{Number(formatUnits(BigInt(munoBalance.balance), 6)).toFixed(2)} UNO</span>
{/if}
