<script lang="ts">
import type { Chain } from "$lib/types"
import { rawToBech32 } from "$lib/utilities/address"
import { Badge } from "$lib/components/ui/badge/index.ts"

export let address: { address: string; normalizedAddress: string }
export let chains: Array<Chain>

// let addressChain = chains.find(c => c.chain_id === chainId)
// let otherChains: Array<Chain> = chains.filter(c => c.chain_id !== chainId)
let cosmosChains: Array<Chain> = chains.filter(c => c.rpc_type === "cosmos" && c.enabled_staging)

// @ts-ignore
const fromHexString = hexString =>
  Uint8Array.from(hexString.match(/.{1,2}/g).map(byte => Number.parseInt(byte, 16)))
</script>

<div>
{#if address.address.startsWith("0x")}
  <div class="text-lg font-bold flex items-center gap-2">{address.address}<Badge>EVM</Badge></div>
{:else}
  <div class="text-lg font-bold flex items-center gap-2">{address.address}<Badge>Cosmos</Badge></div>
  {#each cosmosChains as cosmosChain}
    {#if !address.address.startsWith(cosmosChain.addr_prefix)}
    <div>
      {rawToBech32(cosmosChain.addr_prefix, fromHexString(address.normalizedAddress))}
    </div>
    {/if}
  {/each}
{/if}
</div>

