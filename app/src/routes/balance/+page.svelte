<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query'
  import { cosmosBalancesQuery, evmBalancesQuery } from '$lib/queries/balance'
  import { CHAIN_URLS } from '$lib/constants';
  import { sepoliaStore } from "$lib/wallet/evm/config.ts"
  import { cosmosStore } from "$lib/wallet/cosmos"

$: evmBalances = evmBalancesQuery({
    chainId: '11155111',
    address: $sepoliaStore.address,
    tokenSpecification: 'erc20',
})

$: cosmosBalances = cosmosBalancesQuery({
    chainId: 'union-testnet-8',
      address: $cosmosStore.address
  })

$: data1 = $cosmosBalances?.data || []
$: data2 = $evmBalances?.data || []

</script>

<main>
    <pre>{JSON.stringify(data1, undefined, 2)}</pre>
    <pre>{JSON.stringify(data2, undefined, 2)}</pre>
</main>
