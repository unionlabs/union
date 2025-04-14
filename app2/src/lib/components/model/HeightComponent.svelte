<script lang="ts">
  import type {HTMLAttributes} from "svelte/elements"
  import type {Chain, Height} from "@unionlabs/sdk/schema"
  import Tooltip from "$lib/components/ui/Tooltip.svelte"
  import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"

  type Props = HTMLAttributes<HTMLDivElement> & {
  height: Height
  chain: Chain
  class?: string
}

const { height, chain, class: className = "", ...rest }: Props = $props()

// Find the explorer URL for this block height
const getExplorerUrl = () => {
  if (chain.explorers.length === 0) {
    return null
  }

  // Use the first explorer by default
  const explorer = chain.explorers[0]
  // Replace {block} placeholder if it exists, otherwise append the height
  const blockUrl = explorer.block_url.toString()
  return blockUrl.includes("{block}")
    ? blockUrl.replace("{block}", height.toString())
    : `${blockUrl}${height}`
}

const explorerUrl = $derived(getExplorerUrl())
const explorerName = $derived(chain.explorers.length > 0 ? chain.explorers[0].display_name : null)
</script>

<Tooltip>
  {#snippet trigger()}
    <LongMonoWord class={className} {...rest}>
      {height}
    </LongMonoWord>
  {/snippet}

  {#snippet content()}
    <div class="text-sm flex flex-col gap-4 text-neutral-400">
      <section class="flex justify-between items-center">
        <h2 class="text-white font-bold text-lg">Block Details</h2>
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
        <h3 class="text-white">Block Height</h3>
        <LongMonoWord>
          {height}
        </LongMonoWord>
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
