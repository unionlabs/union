<script lang="ts">
import type { Chain } from "$lib/types"
import { toDisplayName } from "$lib/utilities/chains"
import { hexAddressToBech32 } from "@unionlabs/client"
import { isHex } from "viem"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"

export let chains: Array<Chain>
export let chainId: string
export let address: string | null
export let showChain = false
export let showRaw = false

const chain = chains.find(c => c.chain_id === chainId) ?? null
const parsedAddress =
  chain?.rpc_type === "cosmos" && isHex(address)
    ? hexAddressToBech32({ address, bech32Prefix: chain.addr_prefix })
    : address
const explorer = chain?.explorers?.at(0)?.address_url ?? null
</script>

<div class="flex flex-col text-xs">
  <div class="flex gap-1 items-center">
  {#if parsedAddress}
    {#if !chain}
      invalid chain {chainId}
    {:else}
      {#if !explorer}
        {parsedAddress}
      {:else}
        <a class="underline" href={`${explorer}/${parsedAddress}`}>{parsedAddress}</a>
      {/if}{#if showChain}<ArrowLeftIcon />{toDisplayName(
            chainId,
            chains,
          )}{/if}
    {/if}
  {/if}
  </div>
    {#if address && showRaw}
    <div class="text-muted-foreground">
        RAW: {address}
    </div>
    {/if}
</div>
