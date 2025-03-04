<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { Chain } from "$lib/schema/chain"
import { cn } from "$lib/utils"

type Props = HTMLAttributes<HTMLDivElement> & {
  chain: Chain
  class?: string
}

const { chain, class: className = "", ...rest }: Props = $props()

const classes = cn("flex flex-col text-md font-semibold", className)
</script>

<div class={classes} {...rest}>
  <p>{chain.display_name}</p>
  <!--
  <div>{chain.chain_id}</div>
  <div>{chain.testnet}</div>
  <div>{chain.rpc_type}</div>
  <div>{JSON.stringify(chain.features)}</div>
  !-->
</div>
