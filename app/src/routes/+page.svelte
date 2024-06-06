<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.ts"

  import { cosmosBalancesQuery, evmBalancesQuery } from '$lib/queries/balance'
  import { sepoliaStore } from "$lib/wallet/evm/config.ts"
  import { cosmosStore } from "$lib/wallet/cosmos"
  import { summarizeString } from '$lib/utilities/format'; 

  let evmBalances: null | ReturnType<typeof evmBalancesQuery>;
  $: if($sepoliaStore.address) evmBalances = evmBalancesQuery({
      chainId: '11155111',
      address: $sepoliaStore.address,
      tokenSpecification: 'erc20',
  })

  let cosmosBalances: null | ReturnType<typeof cosmosBalancesQuery>;
  $: if ($cosmosStore.address) cosmosBalances = cosmosBalancesQuery({
    chainId: 'union-testnet-8',
    address: $cosmosStore.address
  })

</script>

<main class="flex flex-col items-center w-full p-4 mt-16 gap-6">
  <Card.Root class="max-w-lg size-full">
    <Card.Header>
      <Card.Title>Welcome to Union</Card.Title>
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
        {#if $cosmosStore.address }
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



