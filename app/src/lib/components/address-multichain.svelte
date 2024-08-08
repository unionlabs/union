<script lang="ts">
import type { Chain } from "$lib/types"
import { rawToBech32 } from "$lib/utilities/address"
import { Badge } from "$lib/components/ui/badge/index.ts"

export let address: { address: string; normalizedAddress: string }
export let chains: Array<Chain>

const addressChain = chains.find(c => address.address.startsWith(c.addr_prefix)) as Chain
const cosmosChains: Array<Chain> = chains.filter(c => c.rpc_type === "cosmos" && c.enabled_staging)

const rpc_type = address.address.startsWith("0x") ? "evm" : "cosmos"

// @ts-ignore
const fromHexString = hexString =>
  Uint8Array.from(hexString.match(/.{1,2}/g).map(byte => Number.parseInt(byte, 16)))

const otherCosmosAddresses: Array<{ address: string; chain: Chain }> = chains
  .filter(c => c.rpc_type === "cosmos")
  .map(c => ({
    address: rawToBech32(c.addr_prefix, fromHexString(address.normalizedAddress)),
    chain: c
  }))
  .filter(pair => pair.address !== address.address)

const allCosmosAddresses = [
  { address: address.address, chain: addressChain },
  ...otherCosmosAddresses
].map(pair => {
  let prefix = pair.chain.addr_prefix
  let body = pair.address.replace(prefix, "").slice(0, -6)
  let checksum = pair.address.slice(-6)

  return {
    prefix,
    body,
    checksum,
    ...pair
  }
})
</script>

<div>
{#if addressChain?.rpc_type === "evm"}
  <div class="text-lg font-bold flex items-center gap-2">{address.address}<Badge>EVM</Badge></div>
{:else}
<Badge>Cosmos</Badge>
<ul>
  {#each allCosmosAddresses as cosmosAddress}
    <li class="text-lg first:font-bold">
      <span class="text-muted-foreground mr-1">{cosmosAddress.prefix}</span>{cosmosAddress.body}<span class="ml-1 text-muted-foreground">{cosmosAddress.checksum}</span>
    </li>
  {/each}
</ul>

  
{/if}
</div>

