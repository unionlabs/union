<script lang="ts">
import type { Chain } from "$lib/types"
import { rawToBech32 } from "$lib/utilities/address"

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
  <h3>EVM address</h3>
  <div>{address.address}</div>
{:else}
  <h3>Cosmos address</h3>
  {#each cosmosChains as cosmosChain}
    <div>
      {rawToBech32(cosmosChain.addr_prefix, fromHexString(address.normalizedAddress))}
    </div>
  {/each}
{/if}
</div>

