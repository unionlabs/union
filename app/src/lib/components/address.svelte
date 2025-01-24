<script lang="ts">
import type { Chain } from "$lib/types"
import { toDisplayName } from "$lib/utilities/chains"
import { hexAddressToBech32 } from "@unionlabs/client"
import { isHex } from "viem"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"

import { highlightItem } from "$lib/stores/highlight"
import { cn } from "$lib/utilities/shadcn"
import Truncate from "./truncate.svelte"

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

<!-- svelte-ignore a11y-interactive-supports-focus -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class={cn("flex flex-col text-xs transition-colors", $highlightItem?.kind === "address" && $highlightItem.address === address  ? "bg-union-accent-300 dark:bg-union-accent-950" : "")}
  on:mouseleave={() => highlightItem.set(null)}
  on:mouseenter={() => {
  highlightItem.set(address ? { kind: "address", address} : null)
  }}>
  <div class="flex gap-1 items-center">
  {#if parsedAddress}
    {#if !chain}
      invalid chain {chainId}
    {:else}
      {#if !explorer}
        {parsedAddress}
      {:else}
        <a class="underline" on:click={(e) => e.stopPropagation()} href={`${explorer}${parsedAddress}`}><Truncate class="underline" value={parsedAddress} type="address"/></a>
      {/if}{#if showChain}<span class="text-muted-foreground flex gap-1"><ArrowLeftIcon />{toDisplayName(
            chainId,
            chains,
          )}</span>{/if}
    {/if}
  {/if}
  </div>
    {#if address && showRaw}
    <div class="text-muted-foreground">
        <Truncate value={address} type="address"/>
    </div>
    {/if}
</div>
