<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { onMount } from 'svelte'
  import { sepolia } from 'viem/chains'
  import { fade } from 'svelte/transition'
  import { injected } from '@wagmi/connectors'
  import { config } from '$lib/wallet/config.ts'
  import { connect, reconnect, getAccount, disconnect } from '@wagmi/core'

  const account = getAccount(config)
  let connectionStatus = account.status
  $: connectionStatus = account.status

  let buttonText = account.status === 'connected' ? account.address : 'Connect Wallet'

  async function connectWallet(_event: MouseEvent) {
    console.log(account.status)
    const isConnected = account.status === 'connected'
    if (isConnected) return disconnect(config, { connector: account.connector })
    const { accounts, chainId } = await connect(config, {
      chainId: sepolia.id,
      connector: injected()
    })
  }

  onMount(async () => {
    console.log(account.status, connectionStatus)
    const [
      {
        accounts: [address]
      }
    ] = await reconnect(config, {
      connectors: [injected()]
    })

    if (account.status === 'connected') buttonText = account.address
    // else if (address.length > 0) buttonText = address
  })

  $: buttonText = account.status === 'connected' ? account.address : 'Connect Wallet'
</script>

<Button.Root
  on:click={connectWallet}
  disabled={account.status === 'reconnecting' || account.status === 'connecting'}
  class={clsx([
    'shadow-mini hover:bg-dark/95 active:scale-98 rounded-lg bg-stone-50 text-black',
    'inline-flex h-12 items-center justify-center px-[21px]',
    'text-[15px] font-semibold active:transition-all'
  ])}
>
  {buttonText}
</Button.Root>
