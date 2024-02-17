<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { wallet, connect, type ConnectorType } from '$lib/wallet/config.ts'

  let error: any

  $: if ($wallet.isConnected) error = undefined

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
</script>

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
