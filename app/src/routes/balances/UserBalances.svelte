<script lang="ts">
import { derived, type Readable, writable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types"
import { userAddressAptos } from "$lib/wallet/aptos"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { allChainBalances } from "$lib/queries/balance"

export let chains: Array<Chain>

let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

$: chainBalances = allChainBalances(chains, userAddress)

let hideZeroBalances = writable(true)

$: filteredChainBalances = derived(
  [chainBalances, hideZeroBalances],
  ([$chainBalances, $hideZeroBalances]) => {
    if (!$hideZeroBalances) return $chainBalances
    return $chainBalances.map(chainAssets =>
      chainAssets.filter(asset => BigInt(asset.balance) > 0n)
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

  {#each $filteredChainBalances as chainAssets, chainIndex}
    {#if chainAssets?.length}
      <div class="rounded-lg border p-4">
        <h2 class="text-lg font-semibold mb-3">
          {chains[chainIndex].display_name}
        </h2>
        <div class="space-y-2">
          {#each chainAssets as asset}
            <div class="flex items-center justify-between p-2 hover:bg-gray-50 rounded">
              <div class="flex items-center space-x-2">
                <span class="font-medium">{asset.display_symbol ?? asset.denom}</span>
                {#if asset.display_name}
                  <span class="text-gray-500 text-sm">
                    ({asset.display_name})
                  </span>
                {/if}
                {#if asset.gasToken}
                  <span class="bg-blue-100 text-blue-800 text-xs px-2 py-0.5 rounded">
                    Gas Token
                  </span>
                {/if}
              </div>
              <div class="text-right">
                <div class="font-medium">
                  {formatBalance(asset.balance, asset.decimals)}
                </div>
                <div class="text-xs text-gray-500">
                  {asset.metadata_level}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/each}
</div>