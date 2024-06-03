<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query'
  import { evmBalancesQuery } from '$lib/queries/balance'
  import { CHAINS } from '$lib/constants'

  $: evmBalances = evmBalancesQuery({
    address: '0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd',
    chainId: '11155111',
    tokenSpecification: 'erc20',
  })

  $: data = $evmBalances?.data || []
</script>

<main>
  <pre>{JSON.stringify(data, undefined, 2)}</pre>
</main>
