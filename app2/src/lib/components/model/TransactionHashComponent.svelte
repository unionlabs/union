<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { Chain } from "@unionlabs/sdk/schema"
import { Array as Arr, Option, pipe, String as Str, Struct } from "effect"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import Truncate from "$lib/components/ui/Truncate.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "../ui/Label.svelte"
import A from "../ui/A.svelte"
import { cn } from "$lib/utils"

type Props = HTMLAttributes<HTMLDivElement> & {
  hash: string
  chain: Chain
  class?: string
}

const { hash, chain, class: className = "", ...rest }: Props = $props()

// For Cosmos chains: remove 0x prefix and convert to uppercase
const formattedHash = $derived(
  chain?.rpc_type === "cosmos" && hash.startsWith("0x") ? hash.slice(2).toUpperCase() : hash
)

// Find the explorer URL for this transaction hash
const explorerUrl = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("tx_url")),
    Option.map(Str.concat(formattedHash))
  )
)

const explorerName = $derived(
  pipe(
    chain.explorers,
    Arr.head,
    Option.map(Struct.get("display_name")),
    Option.getOrElse(() => "explorer")
  )
)
</script>

<Tooltip title={chain ? `Transaction on ${chain.display_name}` : "Transaction"}>
  {#snippet trigger()}
    <Truncate class={cn("font-mono break-all", className)} {...rest} showCopy={false} value={formattedHash} maxLength={12} />
  {/snippet}

  {#snippet content()}
    {#if chain}
      <section>
        <Label>Chain</Label>
        <ChainComponent {chain} />
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
            View on {explorerName}
          </A>
        </div>
      </section>
    {/if}
  {/snippet}
</Tooltip>
