<script lang="ts">
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types.ts"
import Precise from "$lib/components/precise.svelte"
import { findAsset } from "$lib/utilities/helpers.ts"

export let chains: Array<Chain>
export let userAddr: UserAddresses

let chain = chains.filter(c => c.chain_id === "union-testnet-8")
$: userBalances = userBalancesQuery({
  userAddr: userAddr,
  chains: chain
})
$: balance = $userBalances.at(0)?.data ?? []
$: console.log(balance)
$: asset = balance.find(balance => balance.symbol.toLowerCase() === "muno")
$: supportedAsset = findAsset(chain[0], "muno")
</script>

{#if asset && chain}
  <Precise {supportedAsset} {asset} toolTip showSymbol />
{/if}
