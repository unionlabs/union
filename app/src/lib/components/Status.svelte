<script lang="ts">
  import { wallet, switchChain, config } from '$lib/wallet/config.ts'
  import {
    snapAddress,
    snapInstalled,
    snapConnected,
    getSnapAddress,
    suggestSnapChain,
    unionTransactions,
    snapChainConnected,
    ensureSnapInstalled,
    ensureSnapConnected,
    snapChainInitialized,
    ensureSnapChainInitialized,
    sendAssetFromUnionToEthereum
  } from '$/lib/snap.ts'
</script>
      

<div class="p-4 font-mono border-neutral-900 border border-solid border-2">
  <p>Wallet status: {$wallet.status}</p>
  <p>EVM Address: {$wallet.address}</p>
  <p>EVM Chain ID: {$wallet.chainId}</p>
  <p>Union Address: {$snapAddress}</p>
  <p>Union Chain ID: union-testnet-6</p>
</div>
