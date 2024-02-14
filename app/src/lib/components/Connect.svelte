<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { onMount } from 'svelte'
  import { sepolia } from 'viem/chains'
  import {
    config,
    wallet,
    connect,
    switchChain,
    addUnoERC20,
    unionAddress,
    snapInstalled,
    connectToUnion,
    connectLeapSnap,
    connectedToUnion,
    type ConnectorType,
    updateSnapInstalled,
    checkConnectedToUnion,
    unoTokenAddedToMetaMask
  } from '$lib/wallet/config.ts'
  import {
    cosmjsSigner,
    erc20balanceStore,
    getUnoERC20Balance,
    unionBalanceStore,
    getUnoUnionBalance,
    _getOfflineSigner,
    sendUnoFromUnionToSepolia
  } from '$/lib/union-actions'

  let error: any

  $: if ($wallet.isConnected) error = undefined

  // @ts-ignore
  $: $unoTokenAddedToMetaMask = unoTokenAddedToMetaMask

  async function connectWallet(type: ConnectorType) {
    error = undefined
    try {
      const connection = await connect(type, 11_155_111)
      if (!connection) throw new Error(`No matching connector found: ${type}`)
    } catch (error) {
      console.error(error)
      error = error
    }
  }

  onMount(async () => {
    await updateSnapInstalled()
    await checkConnectedToUnion()

    const unoERC20Balance = $wallet.address ? await getUnoERC20Balance($wallet.address) : null
    if (unoERC20Balance) erc20balanceStore.set(unoERC20Balance)

    // TODO: CORS issue
    // const unoUnionBalance = $unionAddress ? await getUnoUnionBalance($unionAddress) : null
    // if (unoUnionBalance) unionBalanceStore.set(unoUnionBalance)

    await _getOfflineSigner()
  })
</script>

<div>
  <p class="mb-12">Status: {$wallet.status}</p>
  {#if $wallet.isConnected}
    <div>
      <p>EVM Address: {$wallet.address}</p>
      <p>EVM Chain ID: {$wallet.chainId}</p>
      <p>UNO ERC20 Balance: {$erc20balanceStore}</p>
      <br />
      <p>Union Address: {$unionAddress}</p>
      <p>Union Chain ID: union-testnet-6</p>
      <p>UNO Union Balance: {$unionBalanceStore}</p>
      <div>
        {#if !$snapInstalled}
          <div class="mt-4">
            <button on:click={connectLeapSnap}>Add Leap Cosmos Wallet to Metamask 🌌</button>
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
        <div class="my-3">
          {#if $unoTokenAddedToMetaMask}
            <div>✅ UNO Token Added to Metamask</div>
          {:else}
            <button on:click={addUnoERC20}>OPTIONAL: Track UNO to Metamask</button>
          {/if}
        </div>
        <div class="my-4">
          {#if $connectedToUnion}
            <div>✅ Connected to Union</div>
          {:else}
            <button on:click={connectToUnion}>Connect to Union</button>
          {/if}
        </div>
      </div>
      <br />
      <div>
        {#if $cosmjsSigner}
          <button on:click={() => sendUnoFromUnionToSepolia($cosmjsSigner, $wallet.address)}
            >Send UNO from Union to Sepolia</button
          >
        {/if}
      </div>
      <button>
        Send UNO from Sepolia to Union
      </button>
    </div>
  {:else}
    <Button.Root
      on:click={() => connectWallet('injected')}
      disabled={error}
      class={clsx([
        'shadow-mini hover:bg-dark/95 active:scale-98 rounded-lg bg-stone-50 text-black',
        'inline-flex h-12 items-center justify-center px-[21px]',
        'text-[15px] font-semibold active:transition-all'
      ])}
    >
      Connect Wallet
    </Button.Root>
  {/if}
</div>
