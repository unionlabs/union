<script lang="ts">
  import { wallet, switchChain } from '$lib/wallet/config.ts'
  import {
    erc20balanceStore,
    unionBalanceStore,
    getUnoERC20Balance,
    getUnoUnionBalance,
    sendAssetFromEthereumToUnion
  } from '$/lib/union-actions'
  import clsx from 'clsx'
  import { onMount } from 'svelte'
  import { sepolia } from 'viem/chains'
  import Faucet from '$/lib/components/Faucet.svelte'
  import Connect from '$lib/components/Connect.svelte'
  import {
    snapAddress,
    snapInstalled,
    getSnapAddress,
    snapChainConnected,
    suggestSnapChain,
    ensureSnapInstalled,
    snapConnected,
    ensureSnapConnected,
    sendSnapTransaction,
    ensureSnapChainInitialized
  } from '$/lib/snap.ts'

  let error: any

  onMount(async () => {
    // await initializeStargateClient()
    await ensureSnapInstalled()
    await ensureSnapConnected()
    await getSnapAddress()
    await ensureSnapChainInitialized()

    const unoUnionBalance = $snapAddress ? await getUnoUnionBalance($snapAddress) : '0'
    unionBalanceStore.set(unoUnionBalance)
    const unoERC20Balance = $wallet.address ? await getUnoERC20Balance($wallet.address) : 0n
    erc20balanceStore.set(unoERC20Balance)
  })

  onMount(async () => {})
</script>

<main
  class="mx-auto mt-12 flex min-h-full min-w-full flex-col items-center justify-center space-y-6"
>
  <p class="mb-12">Status: {$wallet.status}</p>
  {#if $wallet.isConnected}
    <div>
      <p>EVM Address: {$wallet.address}</p>
      <p>EVM Chain ID: {$wallet.chainId}</p>
      <p>UNO ERC20 Balance: {$erc20balanceStore}</p>
      <br />
      <p>Union Address: {$snapAddress}</p>
      <p>Union Chain ID: union-testnet-6</p>
      <p>UNO Union Balance: {$unionBalanceStore}</p>
      <div>
        <p>SNAP INSTALLED: {$snapInstalled}</p>
        <p>SNAP CONNECTED: {$snapConnected}</p>

        <!-- {#if !$snapChainInitialized} -->
        <button on:click={() => suggestSnapChain()}>add union chain</button>
        <!-- {/if} -->
        {#if !$snapInstalled}
          <div class="mt-4">
            <button on:click={ensureSnapInstalled}>Add Leap Cosmos Wallet to Metamask 🌌</button>
          </div>
        {:else}
          <div class="my-4">✅ Leap Cosmos Wallet Installed</div>
        {/if}
        <button
          on:click={() => switchChain(sepolia.id)}
          class={clsx([
            'my-5',
            'shadow-mini hover:bg-dark/95 active:scale-98 rounded-lg bg-stone-50 text-black',
            'inline-flex h-12 items-center justify-center px-[21px]',
            'text-[15px] font-semibold active:transition-all',
            $wallet.chainId === sepolia.id ? 'hidden' : ''
          ])}
        >
          Switch Chain
        </button>
        <div class="my-4">
          {#if $snapChainConnected}
            <div>✅ Connected to Union</div>
          {:else}
            <button on:click={suggestSnapChain}>Connect to Union</button>
          {/if}
        </div>
      </div>
      <br />
      <!-- <div>
        {#if $cosmosOfflineSigner}
          <button on:click={sendUnoFromUnionToSepolia}>Send UNO from Union to Sepolia</button>
        {/if}
      </div> -->
      <div>
        <button on:click={() => sendSnapTransaction()}>Send UNO from Union to Sepolia</button>
      </div>
      <button on:click={() => sendAssetFromEthereumToUnion({ amount: 3n })}>
        Send UNO from Sepolia to Union
      </button>
    </div>
    <Faucet />
  {:else}
    <Connect />
  {/if}
</main>
