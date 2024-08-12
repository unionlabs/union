<script lang="ts">
import type { Chain } from "$lib/types"
import LoadingLogo from "./loading-logo.svelte"
import { chainsQuery } from "$lib/queries/chains"
import { type Readable, derived } from "svelte/store"

let chains = chainsQuery()

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

    let ucs1_configurations = chain.ucs1_configurations?.reduce<Chain["ucs1_configurations"]>(
      (acc, item) => {
        let forward = item.forward.reduce<Record<string, (typeof item.forward)[number]>>(
          (acc2, item2) => {
            acc2[item2.destination_chain.chain_id] = item2
            return acc2
          },
          {}
        )

        let item_with_fwd = {
          ...item,
          forward
        }

        acc[item.destination_chain.chain_id] = item_with_fwd

        return acc
      },
      {}
    ) as Chain["ucs1_configurations"]

    return {
      chain_id: chain.chain_id,
      enabled: chain.enabled,
      enabled_staging: chain.enabled_staging,
      ucs1_configurations,
      display_name,
      rpc_type,
      rpcs: chain.rpcs as Array<{ url: string; type: string }>,
      addr_prefix,
      testnet: !!chain.testnet,
      explorers: chain.explorers,
      // this as statement should no longer be required in the next typescript release
      assets: chain.assets?.filter(
        asset => asset.display_symbol !== null && asset.decimals !== null && asset.denom !== null
      ) as Chain["assets"]
    }
  })
})
</script>

{#if !!$chains.data}
  <slot chains={$checkedChains} rawChains={$chains?.data ?? []} />
{:else if $chains.isLoading}
  <LoadingLogo class="size-16" />
{:else if $chains.isError}
  Error loading chains.
{/if}
