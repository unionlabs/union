<script lang="ts">
import type { Chain } from "$lib/types"
import { rawToBech32 } from "$lib/utilities/address"

export let normalizedAddress: string
export let chains: Array<Chain>
export let chainId: string

let addressChain = chains.find(c => c.chain_id === chainId)
let otherChains: Array<Chain> = chains.filter(c => c.chain_id !== chainId)
let cosmosChains: Array<Chain> = chains.filter(c => c.rpc_type === "cosmos" && c.enabled_staging)

// @ts-ignore
const fromHexString = hexString =>
  Uint8Array.from(hexString.match(/.{1,2}/g).map(byte => parseInt(byte, 16)))
</script>

{#each cosmosChains as cosmosChain}
  <div>
    {rawToBech32(cosmosChain.addr_prefix, fromHexString(normalizedAddress))}
  </div>
{/each}


