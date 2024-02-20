<script lang="ts">
  import clsx from 'clsx'
  import { Button } from 'bits-ui'
  import { fetcher } from '$/lib/utilities'
  import { snapAddress } from '$/lib/snap.ts'
  import { createMutation, useQueryClient } from '@tanstack/svelte-query'

  const queryClient = useQueryClient()

  const unoFromFaucetMutation = createMutation({
    mutationFn: () => fetcher(`/api/faucet?address=${$snapAddress}`),
    onSuccess: async () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ['balance-sepolia-uno'] }),
        queryClient.invalidateQueries({ queryKey: ['balance-union-uno'] })
      ])
  })
</script>

<Button.Root
  class={clsx('rounded-md border-[1px] px-4 py-2')}
  on:click={() => {
    if (!$snapAddress) return
    $unoFromFaucetMutation.mutate()
  }}
>
  get UNO from faucet
</Button.Root>
