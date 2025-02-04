<script lang="ts">
import { writable, derived } from "svelte/store"
import type { Chain } from "$lib/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { userAddress, balanceStore } from "$lib/components/TransferFrom/transfer/balances.ts"

export let chains: Array<Chain>

let userBalancesQueries = userBalancesQuery({ chains, userAddr: $userAddress, connected: true })
</script>

<div>
  <div class="px-4">
  {#each $userBalancesQueries as userBalance}
    {#if userBalance.data}
      <h2 class="font-bold pt-8">{userBalance.data.chain_id}</h2>
      {#if userBalance.data.balances.isOk()}
        <pre class="bg-green-800 px-4">{JSON.stringify(userBalance.data.balances.value, null, 2)}</pre>
      {:else}
        <div class="bg-red-800 px-4">{userBalance.data.balances.error.message}</div>
      {/if}
    {:else}
      <pre class="bg-orange-800 px-4">{JSON.stringify(userBalance,null,2)}</pre>
    {/if}
  {/each}
  </div>
</div>

