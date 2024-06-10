<script lang="ts">
import { chainsQuery } from "$lib/queries/chains"
import { type Readable, derived } from "svelte/store"
import type { Chain } from "$lib/types"
let chains = chainsQuery()

const EMPTY_CHAINS: Array<Chain> = []

let checkedChains: Readable<Array<Chain>> = derived(chains, $chains => {
  // this will never happen, but is needed to satisfy svelte's prop type checker
  if (!$chains.isSuccess) return EMPTY_CHAINS

  return $chains.data.map(chain => {
    let display_name = ""

    if (chain.display_name === null) {
      console.error("no display_name for chain", chain)
    } else {
      display_name = chain.display_name
    }

    let rpc_type: "evm" | "cosmos" = "cosmos"
    if (chain.rpc_type !== "evm" && chain.rpc_type !== "cosmos") {
      console.error("invalid rpc type for chain", chain)
    } else {
      rpc_type = chain.rpc_type
    }

    let addr_prefix = ""
    if (chain.addr_prefix === null) {
      console.error("no addr_prefix for chain", chain)
    } else {
      addr_prefix = chain.addr_prefix
    }

    return {
      chain_id: chain.chain_id,
      display_name,
      rpc_type,
      rpcs: chain.rpcs,
      addr_prefix
    }
  })
})
</script>

{#if $chains.isLoading}
  Loading chains...
{:else if $chains.isSuccess}
  <slot chains={$checkedChains}/>
{/if}
