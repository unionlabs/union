<script lang="ts">
import type { Chain } from "$lib/types"
import { rawToBech32 } from "$lib/utilities/address"
import { Badge } from "$lib/components/ui/badge/index.ts"
import { fade, blur, fly, slide, scale } from "svelte/transition"

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

const allCosmosAddressesDeduplicated = allCosmosAddresses.filter(
  (obj1, i, arr) => arr.findIndex(obj2 => obj2.prefix === obj1.prefix) === i
)

const longestPrefix = Math.max.apply(
  0,
  allCosmosAddresses.map(pair => pair.prefix.length)
)
let addressIndex = 0
setInterval(() => {
  //logic goes here
  addressIndex = (addressIndex + 1) % (allCosmosAddresses.length - 1)
}, 2000)
</script>

{#if addressChain?.rpc_type === "evm"}
  <div class="text-lg font-bold flex items-center gap-2">{address.address}<Badge>EVM</Badge></div>
{:else}
<div class="flex items-center">
  <ul class="py-4">
    {#each allCosmosAddressesDeduplicated as cosmosAddress, i}
      {#if i === addressIndex}
      <li 
        class="text-lg first:font-bold whitespace-pre">
        <span class="select-none">{' '.repeat(longestPrefix - cosmosAddress.prefix.length)}</span><span class="text-muted-foreground mr-1">{cosmosAddress.prefix}</span>{cosmosAddress.body}<span class="ml-1 text-muted-foreground">{cosmosAddress.checksum}</span>
      </li>
      {/if}
    {/each}
  </ul>
  <Badge>Cosmos</Badge>
</div>

{/if}

