<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { Chain } from "@unionlabs/sdk/schema"
import type { AddressCanonicalBytes } from "@unionlabs/sdk/schema"
import { cn } from "$lib/utils"
import { truncate } from "$lib/utils/format"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import { Effect, Option } from "effect"

type Props = HTMLAttributes<HTMLDivElement> & {
  address: AddressCanonicalBytes
  chain: Chain
  class?: string
  truncate?: boolean
  truncateChars?: number
  truncatePosition?: "start" | "middle" | "end"
}

const {
  address,
  chain,
  class: className = "",
  truncate: shouldTruncate = false,
  truncateChars = 12,
  truncatePosition = "middle",
  ...rest
}: Props = $props()

const fullDisplayAddress = $derived(Effect.runSync(chain.getDisplayAddress(address)))
const displayAddress = $derived(
  shouldTruncate
    ? truncate(fullDisplayAddress, truncateChars, truncatePosition)
    : fullDisplayAddress
)

// Find the explorer URL for this address
const getExplorerUrl = () => {
  if (chain.explorers.length === 0) {
    return null
  }

  // Use the first explorer by default
  const explorer = chain.explorers[0]
  // Replace {address} placeholder if it exists, otherwise append the address
  const addressUrl = explorer.address_url.toString()
  return addressUrl.includes("{address}")
    ? addressUrl.replace("{address}", displayAddress)
    : `${addressUrl}${displayAddress}`
}

const explorerUrl = $derived(getExplorerUrl())
const explorerName = $derived(chain.explorers.length > 0 ? chain.explorers[0].display_name : null)
</script>

<Tooltip>
  {#snippet trigger()}
    <LongMonoWord class={className} {...rest}>
      {displayAddress}
    </LongMonoWord>
  {/snippet}

  {#snippet content()}
    <div class="text-sm flex flex-col gap-4 text-neutral-400">
      <section class="flex justify-between items-center">
        <h2 class="text-white font-bold text-lg">Address Details</h2>
        <div class="bg-sky-400 text-black font-bold rounded px-1">
          {chain.rpc_type.toUpperCase()}
        </div>
      </section>

      <section>
        <h3 class="text-white">Chain</h3>
        <div>{chain.display_name}</div>
        <div class="text-xs">{chain.universal_chain_id}</div>
      </section>

      <section>
        <h3 class="text-white">Formats</h3>
        <div>
          <span class="text-white">Display:</span>
          <LongMonoWord class="inline">
            {fullDisplayAddress}
          </LongMonoWord>
        </div>
        <div>
          <span class="text-white">Canonical:</span>
          <LongMonoWord class="inline">
            {address}
          </LongMonoWord>
        </div>
      </section>

      {#if explorerUrl}
        <section>
          <h3 class="text-white">Explorer</h3>
          <div>
            <a 
              href={explorerUrl} 
              class="text-sky-400 hover:text-sky-300 underline" 
              target="_blank" 
              rel="noopener noreferrer"
            >
              View on {explorerName || "Explorer"}
            </a>
          </div>
        </section>
      {/if}
    </div>
  {/snippet}
</Tooltip>
