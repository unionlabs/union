<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { snapAddress } from '$/lib/snap.ts'
  import { getUnoFromFaucet } from '$/lib/faucet.ts'
</script>

<Button.Root
  class={clsx('rounded-md border-[1px] px-4 py-2')}
  on:click={() => {
    if (!$snapAddress) return
    return getUnoFromFaucet({ address: $snapAddress })
  }}
>
  get UNO from faucet
</Button.Root>
