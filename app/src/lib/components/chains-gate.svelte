<script lang="ts">
import type { Chain } from "$lib/types"
import LoadingLogo from "./loading-logo.svelte"
import { chainsQuery } from "$lib/queries/chains"
import { tokensQuery } from "$lib/queries/tokens"
import { type Readable, derived } from "svelte/store"

let chains = chainsQuery()
// let tokens = tokensQuery()

const EMPTY_CHAINS: Array<Chain> = []

let checkedChains: Readable<Array<Chain>> = derived(chains, $chains => {
  // this will never happen, but is needed to satisfy svelte's prop type checker
  if (
    !$chains?.data ||
    $chains.data === null ||
    $chains.data === undefined ||
    $chains.data.length === 0
  ) {
    return EMPTY_CHAINS
  }
  return $chains.data.map(chain => {
    let display_name = ""

    if (chain.display_name === null) {
      console.error("no display_name for chain", chain)
    } else {
      display_name = chain.display_name
    }

    let rpcType = chain.rpc_type
    if (!rpcType) console.error("no rpc type found")

    let addr_prefix = ""
    if (chain.addr_prefix === null) {
      console.error("no addr_prefix for chain", chain)
    } else {
      addr_prefix = chain.addr_prefix
    }
    return {
      chain_id: chain.chain_id,
      enabled: chain.enabled,
      enabled_staging: chain.enabled_staging,
      display_name,
      rpc_type: rpcType,
      rpcs: chain.rpcs,
      addr_prefix,
      testnet: !!chain.testnet,
      explorers: chain.explorers,
      // this as statement should no longer be required in the next typescript release
      tokens: chain.tokens,
      // @deprecated
      assets: chain.assets.filter(
        asset => asset.display_symbol !== null && asset.decimals !== null && asset.denom !== null
      ) as Chain["assets"]
    } as Chain
  })
})
</script>

{#if !!$chains.data}
  <slot chains={$checkedChains} />
{:else if $chains.isLoading}
  <LoadingLogo class="size-16" />
{:else if $chains.isError}
  Error loading chains.
{/if}
