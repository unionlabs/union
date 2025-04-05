<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { Chain } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import Truncate from "$lib/components/ui/Truncate.svelte"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  hash: string
  chain?: Chain
  class?: string
}

const { hash, chain, class: className = "", ...rest }: Props = $props()

// Find the explorer URL for this transaction hash
const getExplorerUrl = () => {
  const chainOption = Option.fromNullable(chain)

  return Option.flatMap(chainOption, c =>
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
}

const getExplorerName = () => {
  return Option.flatMap(Option.fromNullable(chain), c =>
    Option.liftPredicate(c.explorers, explorers => explorers.length > 0).pipe(
      Option.map(explorers => explorers[0].display_name)
    )
  )
}

const getChainName = () => {
  return Option.map(Option.fromNullable(chain), c => c.display_name)
}

const explorerUrl = $derived(getExplorerUrl())
const explorerName = $derived(getExplorerName())
const chainName = $derived(Option.getOrElse(getChainName(), () => ""))
</script>

<Tooltip>
  {#snippet trigger()}
    <div class="font-mono text-xs break-all {className}" {...rest}>
      <Truncate showCopy={false} value={hash} maxLength={12} />
    </div>
  {/snippet}

  {#snippet content()}
    <div class="text-sm flex flex-col gap-4 text-neutral-400">
      <section class="flex justify-between items-center">
        <h2 class="text-white font-bold text-lg">Transaction Details</h2>
        {#if chain}
          <div class="bg-sky-400 text-black font-bold rounded px-1">
            {chain.rpc_type.toUpperCase()}
          </div>
        {/if}
      </section>

      {#if chain}
        <section>
          <h3 class="text-white">Chain</h3>
          <div>{chain.display_name}</div>
          <div class="text-xs">{chain.universal_chain_id}</div>
        </section>
      {/if}

      <section>
        <h3 class="text-white">Transaction Hash</h3>
        <LongMonoWord>
          {hash}
        </LongMonoWord>
      </section>

      {#if Option.isSome(explorerUrl)}
        <section>
          <h3 class="text-white">Explorer</h3>
          <div>
            <a 
              href={Option.getOrElse(explorerUrl, () => "#")} 
              class="text-sky-400 hover:text-sky-300 underline" 
              target="_blank" 
              rel="noopener noreferrer"
            >
              View on {Option.getOrElse(explorerName, () => "Explorer")}
            </a>
          </div>
        </section>
      {/if}
    </div>
  {/snippet}
</Tooltip>
