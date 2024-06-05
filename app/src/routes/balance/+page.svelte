<script lang="ts">
  import { cosmosBalancesQuery, evmBalancesQuery } from '$lib/queries/balance'
  import { sepoliaStore } from "$lib/wallet/evm/config.ts"
  import { cosmosStore } from "$lib/wallet/cosmos"

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


<div>
  <h2>Sepolia</h2>
  {#if $evmBalances}
    {#if $evmBalances.isLoading}
      Loading...
    {:else if $evmBalances.isError}
      {$evmBalances.error.message}
    {:else if $evmBalances.isSuccess}
      <pre>{JSON.stringify($evmBalances.data, null, 2)}</pre>
    {/if}
  {:else}
    <p>Connect your EVM wallet to continue</p>
  {/if}

</div>


<div>
  <h2>Cosmos</h2>
  {#if $cosmosBalances}
    {#if $cosmosBalances.isLoading}
      Loading...
    {:else if $cosmosBalances.isError}
      {$cosmosBalances.error.message}
    {:else if $cosmosBalances.isSuccess}
      <pre>{JSON.stringify($cosmosBalances.data)}</pre>
    {/if}
  {:else}
    <p>Connect your cosmos wallet to continue</p>
  {/if}
</div>

