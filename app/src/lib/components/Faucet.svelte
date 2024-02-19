<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { getUnoFromFaucet } from '$/lib/faucet.ts'
  import { unionAddress } from '$/lib/wallet/config'
</script>

<Button.Root
  class={clsx('rounded-md border-[1px] px-4 py-2')}
  on:click={() => getUnoFromFaucet({ address: $unionAddress })}
>
  get UNO from faucet
</Button.Root>
