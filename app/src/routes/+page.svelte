<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.ts"
  import { derived, type Readable } from 'svelte/store';

  import { cosmosBalancesQuery, evmBalancesQuery } from '$lib/queries/balance'
  import { chainsQuery } from '$lib/queries/chains'
  import { sepoliaStore } from "$lib/wallet/evm/config.ts"
  import { cosmosStore } from "$lib/wallet/cosmos"
  import { summarizeString } from '$lib/utilities/format'; 
  import { bech32 } from 'bech32';

  let evmBalances: null | ReturnType<typeof evmBalancesQuery>;
  $: if($sepoliaStore.address) evmBalances = evmBalancesQuery({
      chainId: '11155111',
      address: $sepoliaStore.address,
      tokenSpecification: 'erc20',
  })

  let cosmosBalances: null | ReturnType<typeof cosmosBalancesQuery>;
  $: if ($cosmosStore.address) cosmosBalances = cosmosBalancesQuery({
    chainIds: ['union-testnet-8', 'osmo-test-5'],
    address: $cosmosStore.address
  })


  let chains = chainsQuery();

  let derivedAddress: Readable<null | string> = derived(cosmosStore, ($cosmosStore) => {
    if (!$cosmosStore.rawAddress) return null;
    const words = bech32.toWords($cosmosStore.rawAddress);
    return bech32.encode('union', words)
  });

</script>

<main class="flex flex-col items-center w-full p-4 mt-16 gap-6">
  <Card.Root class="max-w-lg size-full">
    <Card.Header>
      <Card.Title>Welcome to Union</Card.Title>
      <pre>{JSON.stringify($chains, null, 2)}</pre>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2">
      <p>Connect an <b>EVM</b> and <b>Cosmos</b> wallet to begin bridging.</p>

      <div>
        {#if $sepoliaStore.address }
          ✅ EVM wallet <span class="font-mono">{summarizeString($sepoliaStore.address, 6)}</span> connected
        {:else}
          Connect EVM wallet
        {/if}
      </div>

      <div>
        {#if $cosmosStore.address && $cosmosStore.rawAddress }
          ✅ Cosmos wallet <span class="font-mono">{summarizeString($cosmosStore.address, 6)}</span> connected
        {:else}
          Connect cosmos wallet
        {/if}
      </div>
    </Card.Content>
  </Card.Root>

  <Card.Root class="max-w-lg w-full">
    <Card.Header>
      <Card.Title>Balances</Card.Title>
    </Card.Header>
    <Card.Content class="flex flex-col gap-2">
    <h2 class="font-bold">EVM</h2>
    {#if $evmBalances}
      {#if $evmBalances.isLoading}
        Loading...
      {:else if $evmBalances.isError}
        Error: {$evmBalances.error.message}
      {:else if $evmBalances.isSuccess}
        <div>
          {#each $evmBalances.data as asset}
            <div>{summarizeString(asset.symbol, 8)} | {asset.balance}</div>
          {/each}
        </div>
      {/if}
    {:else}
      <p>Connect your EVM wallet to continue</p>
    {/if}

    <h2 class="font-bold">Cosmos</h2>
    <pre>{JSON.stringify($cosmosBalances, null, 2)}</pre>
    {#if $cosmosBalances}
      {#if $cosmosBalances.isLoading}
        Loading...
      {:else if $cosmosBalances.isError}
        {$cosmosBalances.error.message}
      {:else if $cosmosBalances.isSuccess}
        <div>
          {#each $cosmosBalances.data as asset}
            <div>{summarizeString(asset.symbol, 8)} | {asset.balance}</div>
          {/each}
        </div>
      {/if}
    {:else}
      <p>Connect your cosmos wallet to show cosmos balance</p>
    {/if}
    </Card.Content>
  </Card.Root>




  
</main>



