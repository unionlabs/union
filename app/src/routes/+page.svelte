<script lang="ts">
  import {
    getUnoERC20Balance,
    sepoliaTransactions,
    sendAssetFromEthereumToUnion
  } from '$/lib/union-actions'
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
  import clsx from 'clsx'
  import { onMount } from 'svelte'
  import { Button } from 'bits-ui'
  import { sepolia } from 'viem/chains'
  import toast from 'svelte-french-toast'
  import { getBalance } from '@wagmi/core'
  import Faucet from '$/lib/components/Faucet.svelte'
  import Connect from '$lib/components/Connect.svelte'
  import { generateRandomInteger } from '$/lib/utilities'
  import { fetchUnionUnoBalance } from '$/lib/fetchers/balance'
  import { wallet, switchChain, config } from '$lib/wallet/config.ts'
  import { useQueryClient, createQuery, createMutation } from '@tanstack/svelte-query'

  let error: any

  /**
   * TODO:-
   *  - turn `send*` functions into mutations,
   *  - invalidate all queries in this page on success
   */

  let pollingIntervalMS = 2500

  onMount(async () => {
    await ensureSnapInstalled()
    await ensureSnapConnected()
    await getSnapAddress()
    await ensureSnapChainInitialized()
  })

  const queryClient = useQueryClient()

  $: unoUnionBalance = createQuery<string>({
    // eslint-disable-next-line @tanstack/query/exhaustive-deps
    queryKey: ['balance-union-uno'],
    queryFn: async () => {
      if (!$snapAddress) return '0'
      return await fetchUnionUnoBalance($snapAddress)
    },
    placeholderData: '0',
    enabled: !!$snapAddress,
    refetchInterval: pollingIntervalMS
  })

  $: unoERC20Balance = createQuery<bigint>({
    queryKey: ['balance-sepolia-uno', $wallet.address],
    queryFn: async () => getUnoERC20Balance(`${$wallet.address}`),
    placeholderData: 0n,
    enabled: !!$wallet.address,
    refetchInterval: pollingIntervalMS
  })

  $: sepoliaEthBalance = createQuery<string>({
    // eslint-disable-next-line @tanstack/query/exhaustive-deps
    queryKey: ['balance-sepolia-eth', $wallet.address],
    queryFn: async () => {
      if (!$wallet.address) return '0'
      const balance = await getBalance(config, { address: $wallet.address, chainId: sepolia.id })
      return balance.formatted
    },
    placeholderData: '0',
    enabled: !!$wallet.address,
    refetchInterval: pollingIntervalMS * 1.5
  })
</script>

<main
  class="mx-auto mt-12 flex min-h-full min-w-full flex-col items-center justify-center space-y-6"
>
  <p>Status: {$wallet.status}</p>
  {#if $wallet.isConnected}
    <div>
      <a
        class={clsx(['rounded-md border-[1px] border-gray-200 px-4 py-2 text-blue-500 underline'])}
        href="https://www.alchemy.com/faucets/ethereum-sepolia"
        target="_blank"
        rel="noopener noreferrer"
      >
        Sepolia ETH Faucet
      </a>
      <p>EVM Address: {$wallet.address}</p>
      <p>EVM Chain ID: {$wallet.chainId}</p>
      <p>UNO ERC20 Balance: {$unoERC20Balance.data}</p>
      <p>Sepolia ETH Balance: {$sepoliaEthBalance.data}</p>
      <br />
      <p>Union Address: {$snapAddress}</p>
      <p>Union Chain ID: union-testnet-6</p>
      <p>UNO Union Balance: {$unoUnionBalance.data}</p>
      <div>
        <p>SNAP INSTALLED: {$snapInstalled}</p>
        <p>SNAP CONNECTED: {$snapConnected}</p>

        {#if !$snapChainInitialized}
          <button on:click={() => suggestSnapChain()}>add union chain</button>
        {/if}
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
            'rounded-lg bg-stone-50 text-black shadow-mini hover:bg-dark/95 active:scale-98',
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

      <section class="my-3 flex max-w-72 flex-col space-y-2">
        <div>
          <Button.Root
            class={clsx(['rounded-md border-[1px] px-4 py-2'])}
            on:click={() => {
              if ($unoUnionBalance?.data === '0') {
                toast.error('$UNO balance on Union is 0\nUse faucet button to get sum', {
                  position: 'bottom-center'
                })
              } else {
                sendAssetFromUnionToEthereum({ amount: BigInt(generateRandomInteger(1, 99)) })
              }
            }}
          >
            Send UNO from Union to Sepolia
          </Button.Root>
          <ol>
            {#each $unionTransactions as transactionHash}
              <li>
                <a
                  href={`https://testnet.union.explorers.guru/transaction/${transactionHash}`}
                  class="text-blue-100 underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {transactionHash}
                </a>
              </li>
            {/each}
          </ol>
        </div>
        <div>
          <Button.Root
            class={clsx(['rounded-md border-[1px] px-4 py-2'])}
            on:click={() => {
              if ($sepoliaEthBalance.data !== '0' && $unoERC20Balance.data !== 0n)
                sendAssetFromEthereumToUnion({ amount: BigInt(generateRandomInteger(1, 99)) })
              else
                toast.error(
                  `Sepolia ETH balance is ${sepoliaEthBalance}\nSepolia $UNO balance is ${unoERC20Balance}`,
                  { position: 'bottom-center' }
                )
            }}
          >
            Send UNO from Sepolia to Union
          </Button.Root>
          <ol>
            {#each $sepoliaTransactions as transactionHash}
              <li>
                <a
                  href={`https://11155111.testnet.routescan.io/tx/${transactionHash}`}
                  class="text-blue-200 underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {transactionHash}
                </a>
              </li>
            {/each}
          </ol>
        </div>
        <div class="w-full">
          <Faucet />
        </div>
      </section>
    </div>
  {:else}
    <Connect />
  {/if}
</main>
