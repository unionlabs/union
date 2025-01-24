<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types"
import Address from "../address.svelte"

export let chains: Array<Chain>
export let value: {
  chainId: string
  address: string
}

const chainDisplayName =
  chains.find(chain => chain.chain_id === value.chainId)?.display_name ??
  value.chainId ??
  "unknown chain"
</script>

<div class={cn("flex flex-col items-start")} {...$$restProps}>
    <div class="font-bold">
        {chainDisplayName}
    </div>
    <Address address={value.address} chainId={value.chainId} {chains}/>
</div>
