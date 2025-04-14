<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { AddressCanonicalBytes, Chain } from "@unionlabs/sdk/schema"
import { truncate } from "$lib/utils/format"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import { Effect } from "effect"
import Label from "../ui/Label.svelte"
import A from "../ui/A.svelte"

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
// const fullDisplayAddress = address
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
        <Label>Chain</Label>
        <div>{chain.display_name}</div>
      </section>
      <section>
        <Label>Display</Label>
        <LongMonoWord>
          {fullDisplayAddress}
        </LongMonoWord>
      </section>

      <section>
        <Label>Canonical</Label>
        <LongMonoWord>
          {address}
        </LongMonoWord>
      </section>

      {#if explorerUrl}
        <section>
        <Label>Explorer</Label>
          <div>
            <A class="underline"  href={explorerUrl}>
              View on {explorerName || "Explorer"}
            </A>
          </div>
        </section>
      {/if}
    </div>
  {/snippet}
</Tooltip>
