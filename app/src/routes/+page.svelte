<script lang="ts">
import * as Card from "$lib/components/ui/card/index.ts"
import { derived, type Readable } from "svelte/store"
import { rawToBech32, rawToHex } from "$lib/utilities/address"

import { cosmosBalancesQuery, evmBalancesQuery } from "$lib/queries/balance"
import { chainsQuery } from "$lib/queries/chains"
import { sepoliaStore } from "$lib/wallet/evm/config.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import { truncate } from "$lib/utilities/format"

let evmBalances: null | ReturnType<typeof evmBalancesQuery>
$: if ($sepoliaStore.address)
  evmBalances = evmBalancesQuery({
    chainId: "11155111",
    address: $sepoliaStore.address,
    tokenSpecification: "erc20"
  })

let chains = chainsQuery()
let cosmosBalances: null | ReturnType<typeof cosmosBalancesQuery>
let cosmosChains = derived(chains, $chains => {
  if (!$chains?.isSuccess) {
    return null
  }
  return $chains.data.filter(
    (c: (typeof $chains.data)[number]) =>
      c.rpc_type === "cosmos" && c.addr_prefix !== null && c.rpcs && c.chain_id
  )
})

$: if (
  $cosmosChains &&
  $cosmosStore.rawAddress?.length !== undefined &&
  $cosmosStore.rawAddress?.length > 0
) {
  cosmosBalances = cosmosBalancesQuery({
    // https://stackoverflow.com/questions/77206461/type-guard-function-is-not-narrowing-the-type-in-array-filter
    //@ts-ignore
    chains: $cosmosChains,
    address: $cosmosStore.rawAddress
  })
}

$: if ($cosmosStore.connectionStatus === "disconnected") cosmosBalances = null
</script>

<main class="flex flex-col items-center w-full p-4 mt-16 gap-6">
  <Card.Root class="max-w-xl size-full">
    <Card.Header>
      <Card.Title>Welcome to Union</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2">
      <p>Connect an <b>EVM</b> and <b>Cosmos</b> wallet to begin bridging.</p>
      <div>
        {#if $sepoliaStore.address }
          ✅ EVM wallet <span class="font-mono">{truncate($sepoliaStore.address, 6)}</span> connected
        {:else}
          Connect EVM wallet
        {/if}
      </div>

      <div>
        {#if $cosmosStore.address && $cosmosStore.rawAddress }
          ✅ Cosmos wallet <span class="font-mono">{truncate($cosmosStore.address, 6)}</span> connected
          <div class="text-xs font-mono text-muted-foreground">RAW: {rawToHex($cosmosStore.rawAddress)}</div>
        {:else}
          Connect cosmos wallet
        {/if}
      </div>
    </Card.Content>
  </Card.Root>


  <Card.Root class="max-w-xl w-full">
    <Card.Header>
      <Card.Title>Balances</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-6">
    <div>
      <h2 class="font-bold">Sepolia</h2>
      {#if $evmBalances}
        {#if $evmBalances.isLoading}
          Loading...
        {:else if $evmBalances.isError}
          Error: {$evmBalances.error.message}
        {:else if $evmBalances.isSuccess}
          <div>
            {#each $evmBalances.data as asset}
              <div>{truncate(asset.symbol, 8)} | {asset.balance}</div>
            {/each}
          </div>
        {/if}
      {:else}
        <p>Connect your EVM wallet to continue</p>
      {/if}
    </div>

    {#if $cosmosChains && $cosmosBalances}
      {#each $cosmosBalances as balance, index}
        <div>
        <h3 class="font-bold">{$cosmosChains[index].display_name}</h3>
          {#if $cosmosChains[index]?.addr_prefix && $cosmosStore.rawAddress && $cosmosStore.rawAddress.length > 0}
            <div class="text-xs font-mono text-muted-foreground">
              {rawToBech32($cosmosChains[index].addr_prefix, $cosmosStore.rawAddress)}
            </div>
          {/if}
          {#if balance.isLoading}
            <p class="text-muted-foreground">Loading...</p>
          {:else if balance.isError}
            <p class="text-red-500">{balance.error}</p>
          {:else if balance.isSuccess}
          <div>
            {#if !(balance.data instanceof Error)}
              {#each balance.data as asset}
                <div>{truncate(asset.symbol, 8)} | {asset.balance}</div>
              {/each}
            {/if}
          </div>
          {/if}
        </div>
      {/each}
    {:else}
      <p>Connect your cosmos wallet to show cosmos balance</p>
    {/if}
    </Card.Content>
  </Card.Root>
</main>



