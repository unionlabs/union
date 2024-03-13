<script lang="ts">
import clsx from "clsx"
import { sepolia } from "viem/chains"
import { Button } from "$lib/components/ui/button"
import { wallet, connect, type ConnectorType } from "$lib/wallet/config.ts"

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

<Button
  on:click={() => connectWallet('injected')}
  disabled={error}
  class={clsx([
    // 'shadow-mini active:scale-98 rounded-lg bg-stone-50 text-black',
    // 'inline-flex h-12 items-center justify-center px-[21px]',
    // 'text-[15px] font-semibold active:transition-all'
  ])}
>
  Connect Wallet
</Button>
