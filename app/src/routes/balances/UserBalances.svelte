<script lang="ts">
import { writable, derived, type Writable } from "svelte/store"
import type { Chain } from "$lib/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { userAddress, balanceStore } from "$lib/components/TransferFrom/transfer/balances.ts"

import { balances, queryBalances, updateBalance } from "$lib/stores/balances.ts"
import { onMount } from "svelte"

export let chains: Array<Chain>

balances.subscribe(x => console.log("updated", x))
userAddress.subscribe(addr => {
  chains
    .filter(chain => addr[chain.rpc_type])
    .forEach(chain => queryBalances(chain, addr[chain.rpc_type]?.canonical as string))
})
</script>

<div class="space-y-6">
  <pre>{JSON.stringify($userAddress, null, 2)}</pre>
  <pre>{JSON.stringify($balances, null, 2)}</pre>
</div>
