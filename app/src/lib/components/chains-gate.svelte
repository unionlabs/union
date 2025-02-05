<script lang="ts">
import type { Chain, Ucs03Channel, ChainFeature } from "$lib/types"
import LoadingLogo from "./loading-logo.svelte"
import { chainsQuery } from "$lib/queries/chains"
import { recommendedUcs03ChannelsQuery } from "$lib/queries/channels"
import { type Readable, derived } from "svelte/store"
import { isHex } from "viem"
import { page } from "$app/stores"

let chains = chainsQuery($page.data.environment)
let ucs03channels = recommendedUcs03ChannelsQuery()

const EMPTY_CHAINS: Array<Chain> = []
const EMPTY_UCS03_CHANNELS: Array<Ucs03Channel> = []

let checkedChains: Readable<Array<Chain>> = derived([chains, page], ([$chains, $page]) => {
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
      features: chain.features,
      // this as statement should no longer be required in the next typescript release
      tokens: chain.tokens,
      // @deprecated
      assets: chain.assets.filter(
        asset => asset.display_symbol !== null && asset.decimals !== null && asset.denom !== null
      ) as Chain["assets"]
    } as Chain
  })
})

let checkedUcs03Channels: Readable<Array<Ucs03Channel>> = derived(ucs03channels, $ucs03channels => {
  // this will never happen, but is needed to satisfy svelte's prop type checker
  if (
    !$ucs03channels?.data ||
    $ucs03channels.data === null ||
    $ucs03channels.data === undefined ||
    $ucs03channels.data.length === 0
  ) {
    return EMPTY_UCS03_CHANNELS
  }
  return $ucs03channels.data.filter(
    chan =>
      chan.source_chain_id &&
      chan.source_connection_id !== null &&
      chan.source_channel_id !== null &&
      isHex(chan.source_port_id) &&
      chan.destination_chain_id &&
      chan.destination_connection_id !== null &&
      chan.destination_channel_id !== null &&
      isHex(chan.destination_port_id)
  ) as Array<Ucs03Channel>
})
</script>

{#if !!$chains.data && !!$ucs03channels.data}
  <slot chains={$checkedChains} ucs03channels={$checkedUcs03Channels} />
{:else if $chains.isLoading}
  <LoadingLogo class="size-16" />
{:else if $chains.isError}
  Error loading chains.
{/if}
