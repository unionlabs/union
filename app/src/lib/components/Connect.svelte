<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { onMount } from 'svelte'
  import { sepolia } from 'viem/chains'
  import { injected } from '@wagmi/connectors'
  import { config } from '$lib/wallet/config.ts'
  import { connect, reconnect, getAccount } from '@wagmi/core'

  const account = getAccount(config)
  let buttonText = account.status === 'connected' ? account.address : 'Connect Wallet'

  async function connectWallet(_event: MouseEvent) {
    const { accounts, chainId } = await connect(config, {
      chainId: sepolia.id,
      connector: injected()
    })
  }

  onMount(async () => {
    const [
      {
        accounts: [address]
      }
    ] = await reconnect(config, {
      connectors: [injected()]
    })

    if (account.status === 'connected') buttonText = account.address
    else if (address.length > 0) buttonText = address
  })
</script>

<Button.Root
  on:click={connectWallet}
  disabled={buttonText !== 'Connect Wallet'}
  class={clsx([
    'shadow-mini hover:bg-dark/95 active:scale-98 rounded-lg bg-stone-50 text-black',
    'inline-flex h-12 items-center justify-center px-[21px]',
    'text-[15px] font-semibold active:transition-all'
  ])}
>
  {buttonText}
</Button.Root>
