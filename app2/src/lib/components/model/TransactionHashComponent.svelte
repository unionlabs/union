<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { Chain } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import Truncate from "$lib/components/ui/Truncate.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "../ui/Label.svelte"
import A from "../ui/A.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  hash: string
  chain?: Chain
  class?: string
}

const { hash, chain, class: className = "", ...rest }: Props = $props()

// Format hash based on chain type
const formattedHash = $derived.by(() => {
  if (chain?.rpc_type === "cosmos" && hash.startsWith("0x")) {
    // For Cosmos chains: remove 0x prefix and convert to uppercase
    return hash.slice(2).toUpperCase()
  }
  return hash
})

// Find the explorer URL for this transaction hash
const explorerUrl = $derived(
  Option.flatMap(Option.fromNullable(chain), c =>
    Option.liftPredicate(c.explorers, explorers => explorers.length > 0).pipe(
      Option.map(explorers => {
        // Use the first explorer by default
        const explorer = explorers[0]
        // Replace {hash} placeholder if it exists, otherwise append the hash
        const txUrl = explorer.tx_url.toString()
        return txUrl.includes("{hash}") ? txUrl.replace("{hash}", formattedHash) : `${txUrl}${formattedHash}`
      })
    )
  )
)
const explorerName = $derived(
  Option.flatMap(Option.fromNullable(chain), c =>
    Option.liftPredicate(c.explorers, explorers => explorers.length > 0).pipe(
      Option.map(explorers => explorers[0].display_name)
    )
  )
)
</script>

<Tooltip title="Transaction">
  {#snippet trigger()}
    <Truncate class="font-mono text-xs break-all {className}" {...rest} showCopy={false} value={formattedHash} maxLength={12} />
  {/snippet}

  {#snippet content()}
    {#if chain}
      <section>
        <Label>Chain</Label>
        <ChainComponent chain={chain} />
      </section>
    {/if}

    <section>
      <Label>Transaction Hash</Label>
      <LongMonoWord>
        {formattedHash}
      </LongMonoWord>
    </section>

    {#if Option.isSome(explorerUrl)}
      <section>
        <Label>Explorer</Label>
        <div>
          <A href={explorerUrl.value}>
            View on {Option.getOrElse(explorerName, () => "Explorer")}
          </A>
        </div>
      </section>
    {/if}
  {/snippet}
</Tooltip>
