<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { sepolia } from 'viem/chains'
  import { wallet, connect, type ConnectorType } from '$lib/wallet/config.ts'

  let error: any

  $: if ($wallet.isConnected) error = undefined

  async function connectWallet(type: ConnectorType) {
    error = undefined
    try {
      const connection = await connect(type, sepolia.id)
      if (!connection) throw new Error(`No matching connector found: ${type}`)
    } catch (error) {
      var _error = error instanceof Error ? error.message : error
      console.error(_error)
      error = _error
    }
  }
</script>

<Button.Root
  on:click={() => connectWallet('injected')}
  disabled={error}
  class={clsx([
    'rounded-lg bg-stone-50 text-black shadow-mini hover:bg-dark/95 active:scale-98',
    'inline-flex h-12 items-center justify-center px-[21px]',
    'text-[15px] font-semibold active:transition-all'
  ])}
>
  Connect Wallet
</Button.Root>
