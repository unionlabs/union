<script lang="ts">
  import type {HTMLAttributes} from "svelte/elements"
  import type {Chain} from "@unionlabs/sdk/schema"
  import {Array as Arr, Option, pipe, String as Str, Struct} from "effect"
  import Tooltip from "$lib/components/ui/Tooltip.svelte"
  import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
  import ChainComponent from "$lib/components/model/ChainComponent.svelte"
  import Label from "../ui/Label.svelte"
  import A from "../ui/A.svelte"

  type Props = HTMLAttributes<HTMLDivElement> & {
  hash: string
  chain: Chain
  class?: string
}

const { hash, chain, class: className = "", ...rest }: Props = $props()

// For Cosmos chains: remove 0x prefix and convert to uppercase
const formattedHash = $derived(
  chain.rpc_type === "cosmos" && hash.startsWith("0x") ? hash.slice(2).toUpperCase() : hash
)

const explorerUrl = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("block_url")),
    Option.map(Str.concat(formattedHash))
  )
)

const explorerName = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("display_name")),
    Option.getOrElse(() => "Explorer")
  )
)
</script>

<Tooltip title={`Block on ${chain.display_name}`}>
  {#snippet trigger()}
    <LongMonoWord class={className} {...rest}>
      {formattedHash}
    </LongMonoWord>
  {/snippet}

  {#snippet content()}
    <section>
      <Label>Chain</Label>
      <ChainComponent {chain} />
    </section>

    <section>
      <Label>Block Hash</Label>
      <LongMonoWord>
        {formattedHash}
      </LongMonoWord>
    </section>

    {#if Option.isSome(explorerUrl)}
      <section>
        <Label>Explorer</Label>
        <A href={explorerUrl.value}>View on {explorerName}</A>
      </section>
    {/if}
  {/snippet}
</Tooltip>
