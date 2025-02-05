<script lang="ts">
import { writable, derived, type Writable } from "svelte/store"
import type { Chain } from "$lib/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { userAddress, balanceStore } from "$lib/components/TransferFrom/transfer/balances.ts"

import { balances, queryBalances, updateBalance } from "$lib/stores/balances.ts"
import { onMount } from "svelte"

export let chains: Array<Chain>

onMount(() => {
  queryBalances(
    chains.find(c => c.chain_id === "17000"),
    "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
  )
  // updateBalance("union-testnet-9", "muno", { kind: "balance", amount: "10", timestamp: "" })
  // updateBalance("elgafar-1", "stars", { kind: "balance", amount: "10", timestamp: "" })
})

balances.subscribe(x => console.log("updated", x))
</script>

<div class="space-y-6">
  <pre>{JSON.stringify($balances, null, 2)}</pre>
</div>
