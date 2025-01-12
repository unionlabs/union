<script lang="ts">
import { writable, derived } from "svelte/store"
import type { Chain } from "$lib/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { userAddress, balanceStore } from "$lib/components/TransferFrom/transfer/balances.ts"

export let chains: Array<Chain>

$: userBalancesQueries = userBalancesQuery({ chains, userAddr: $userAddress, connected: true })
$: balanceStore.set($userBalancesQueries.map(query => query.data || []))

let hideZeroBalances = writable(true)

$: filteredBalanceStore = derived(
  [balanceStore, hideZeroBalances],
  ([$balanceStore, $hideZeroBalances]) => {
    return $balanceStore.map(chainAssets =>
      $hideZeroBalances ? chainAssets.filter(asset => BigInt(asset.balance) > 0n) : chainAssets
    )
  }
)

function formatBalance(balance: string, decimals: number | null): string {
  if (!decimals) return balance
  const num = Number(balance) / 10 ** decimals
  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6
  }).format(num)
}
</script>

<div class="space-y-6">
  <div class="flex items-center space-x-2">
    <input type="checkbox" id="hideZeroBalances" bind:checked={$hideZeroBalances}>
    <label for="hideZeroBalances">Hide zero balances</label>
  </div>

  {#each $filteredBalanceStore as chainAssets, chainIndex}
    {#if chainAssets?.length}
      <div class="rounded-lg border p-4">
        <h2 class="text-lg font-semibold mb-3">
          {chains[chainIndex].display_name}
        </h2>
        <div class="space-y-2">
          {#each chainAssets as asset}
            <div class="flex items-center justify-between p-2 hover:bg-gray-50 rounded"
                 class:opacity-50={BigInt(asset.balance) === 0n}>
              <div class="flex items-center space-x-2">
                <span class="font-medium">{asset.metadata.display_symbol ?? asset.metadata.denom}</span>
                {#if asset.metadata.display_name}
                  <span class="text-gray-500 text-sm">
                    ({asset.metadata.display_name})
                  </span>
                {/if}
                {#if asset.metadata.gasToken}
                  <span class="bg-blue-100 text-blue-800 text-xs px-2 py-0.5 rounded">
                    Gas Token
                  </span>
                {/if}
              </div>
              <div class="text-right">
                <div class="font-medium">
                  {formatBalance(asset.balance, asset.metadata.decimals)}
                </div>
                <div class="text-xs text-gray-500">
                  {asset.metadata.metadata_level}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/each}
</div>