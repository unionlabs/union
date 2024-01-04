<script lang="ts">
  import { readable } from 'svelte/store'
  import {config} from '$lib/wallet/config'
  import { getAccount,getEnsName, watchAccount, connect } from '@wagmi/core'
  import { injected } from '@wagmi/connectors'

  // const ensName = await getEnsName(config, { address: '0xf4212614C7Fe0B3feef75057E88b2E77a7E23e83'})
</script>

<main class="flex h-full w-full max-w-[500px] justify-center flex-col mx-auto items-center">
  <p class="text-4xl font-extrabold my-3">Transfer</p>
  <div class="container">
    <!-- <p>ENS Name: {ensName}</p> -->
    <!-- <div>
      <div>user address: {$account.address}</div>
      <div>Connected chain: {$network.chain?.name}</div>
    </div> -->
    <w3m-button />
  </div>
</main>
