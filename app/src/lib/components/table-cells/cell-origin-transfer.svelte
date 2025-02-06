<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types"
import Address from "../address.svelte"
import ChainDetails from "$lib/chain-details.svelte"

export let chains: Array<Chain>
export let value: {
  chainId: string
  address: string
}
</script>

<div class={cn("flex flex-col items-start")} {...$$restProps}>
    <div class="font-bold">
      <ChainDetails {chains} chainId={value.chainId}/>
    </div>
    <span class="text-muted-foreground"><Address address={value.address} chainId={value.chainId} {chains}/><span>
</div>
