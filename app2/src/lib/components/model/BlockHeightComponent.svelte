<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import Truncate from "$lib/components/ui/Truncate.svelte"
import { cn } from "$lib/utils"
import type { Chain, Height } from "@unionlabs/sdk/schema"
import { Array as Arr, Option, pipe, String as Str, Struct } from "effect"
import type { HTMLAttributes } from "svelte/elements"
import A from "../ui/A.svelte"
import Label from "../ui/Label.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  height: Height
  chain: Chain
  class?: string
}

const { height, chain, class: className = "", ...rest }: Props = $props()

// Find the explorer URL for this block height
const explorerUrl = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("block_url")),
    Option.map(Str.concat(`${height}`)),
  ),
)

const explorerName = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("display_name")),
    Option.getOrElse(() => "explorer"),
  ),
)
</script>

<Tooltip title={chain ? `Transaction on ${chain.display_name}` : "Transaction"}>
  {#snippet trigger()}
    <Truncate
      class={cn("font-mono break-all", className)}
      {...rest}
      showCopy={false}
      value={`${height}`}
      maxLength={12}
    />
  {/snippet}

  {#snippet content()}
    {#if chain}
      <section>
        <Label>Chain</Label>
        <ChainComponent {chain} />
      </section>
    {/if}

    <section>
      <Label>Block Height</Label>
      <LongMonoWord>
        {height}
      </LongMonoWord>
    </section>

    {#if Option.isSome(explorerUrl)}
      <section>
        <Label>Explorer</Label>
        <div>
          <A href={explorerUrl.value}>
            View on {explorerName}
          </A>
        </div>
      </section>
    {/if}
  {/snippet}
</Tooltip>
