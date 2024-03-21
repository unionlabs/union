<script lang="ts">
import clsx from "clsx"

import { snapAddress } from "$/lib/snap.ts"
import { Button } from "$lib/components/ui/button"
import { getUnoFromFaucet } from "$/lib/mutations/faucet"
import { createMutation, useQueryClient } from "@tanstack/svelte-query"

const queryClient = useQueryClient()

const unoFromFaucetMutation = createMutation({
  mutationFn: async () => {
    if (!$snapAddress) return
    return await getUnoFromFaucet({ address: $snapAddress })
  },
  onSuccess: () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["balance-sepolia-uno"] }),
      queryClient.invalidateQueries({ queryKey: ["balance-union-uno"] })
    ])
})
</script>

<Button
  class={clsx('rounded-md px-4 py-2')}
  on:click={() => {
    if (!$snapAddress) return
    $unoFromFaucetMutation.mutate()
  }}
>
  Get UNO from faucet
</Button>
