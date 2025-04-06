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

// Find the explorer URL for this transaction hash
const explorerUrl = $derived(
  Option.flatMap(Option.fromNullable(chain), c =>
    Option.liftPredicate(c.explorers, explorers => explorers.length > 0).pipe(
      Option.map(explorers => {
        // Use the first explorer by default
        const explorer = explorers[0]
        // Replace {hash} placeholder if it exists, otherwise append the hash
        const txUrl = explorer.tx_url.toString()
        return txUrl.includes("{hash}") ? txUrl.replace("{hash}", hash) : `${txUrl}${hash}`
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
    <Truncate class="font-mono text-xs break-all {className}" {...rest} showCopy={false} value={hash} maxLength={12} />
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
        {hash}
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
