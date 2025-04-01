<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { Chain } from "@unionlabs/sdk/schema"
import { cn } from "$lib/utils"
import Tooltip from "$lib/components/ui/Tooltip.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  chain: Chain
  class?: string
}

const { chain, class: className = "", ...rest }: Props = $props()

const classes = cn("flex flex-col text-md font-semibold", className)
</script>

<Tooltip>
  {#snippet trigger()}
    <div class={classes} {...rest}>
      <p>{chain.display_name}</p>
    </div>
  {/snippet}

  {#snippet content()}
    <div class="text-sm flex flex-col gap-4 text-neutral-400">
      <section class="flex justify-between items-center">
        <h2 class="text-white font-bold text-lg">{chain.display_name}</h2>
        <div class="bg-sky-400 text-black font-bold rounded px-1">
          {chain.rpc_type.toUpperCase()}
        </div>
      </section>

      <section>
        <h3 class="text-white">Chain Details</h3>
        <div>Chain ID: {chain.chain_id}</div>
        <div>Universal ID: {chain.universal_chain_id}</div>
        <div>Address Prefix: {chain.addr_prefix}</div>
        <div>Network: {chain.testnet ? 'Testnet' : 'Mainnet'}</div>
      </section>

      <section>
        <h3 class="text-white">RPC Endpoints</h3>
        <div class="flex flex-col gap-2">
          {#each chain.rpcs as rpc}
            <div>
              <span class="text-white">{rpc.type}:</span>
              <a href={rpc.url} class="underline ml-2" target="_blank" rel="noopener noreferrer">
                {rpc.url}
              </a>
            </div>
          {/each}
        </div>
      </section>

      {#if chain.explorers.length > 0}
        <section>
          <h3 class="text-white">Explorers</h3>
          <div class="flex flex-col gap-2">
            {#each chain.explorers as explorer}
              <div class="flex flex-col">
                <div class="flex items-center gap-2">
                  <span class="text-white">{explorer.display_name}</span>
                  <a href={explorer.home_url} class="underline" target="_blank" rel="noopener noreferrer">
                    {explorer.home_url}
                  </a>
                </div>
                <div class="text-xs">{explorer.description}</div>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <section>
        <h3 class="text-white">Features</h3>
        <div class="grid grid-cols-2 gap-x-4">
          {#each Object.entries(chain.features[0] || {}) as [key, enabled]}
            <div class="flex items-center gap-2">
              <div class={cn(
                "w-2 h-2 rounded-full",
                enabled ? "bg-green-500" : "bg-red-500"
              )} />
              <span>{key.replace(/_/g, ' ')}</span>
            </div>
          {/each}
        </div>
      </section>
    </div>
  {/snippet}
</Tooltip>
