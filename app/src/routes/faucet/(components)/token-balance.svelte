<script lang="ts">
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddressCosmos } from "$lib/types.ts"
import Precise from "$lib/components/precise.svelte"

  interface Props {
    chains: Array<Chain>;
    userAddrCosmos: UserAddressCosmos;
    symbol: string;
  }

  let { chains, userAddrCosmos, symbol }: Props = $props();

let chain = chains.filter(c => c.chain_id === "union-testnet-8")
let userBalances = $derived(userBalancesQuery({
  userAddr: { cosmos: userAddrCosmos, evm: null },
  chains: chain,
  connected: true
}))
let unionBalances = $derived($userBalances.at(0)?.data ?? [])
let asset = $derived(unionBalances.find(balance => balance.symbol.toLowerCase() === symbol.toLowerCase()))
</script>

{#if asset}
  <span class=""><Precise chain={chain[0]} {asset} showSymbol displayDecimals={6} /></span>
{/if}
