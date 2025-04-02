<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { getChain, PacketTrace } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import { cn } from "$lib/utils"
import Label from "../ui/Label.svelte"
import { Option } from "effect"
import DateTimeComponent from "../ui/DateTimeComponent.svelte"
import TransactionHashComponent from "./TransactionHashComponent.svelte"
import BlockHashComponent from "./BlockHashComponent.svelte"
import ChainComponent from "./ChainComponent.svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  packetTraces: ReadonlyArray<PacketTrace>
}

const { packetTraces }: Props = $props()
</script>


{#if packetTraces.length > 0 && Option.isSome(chains.data)}
{@const chainsList = chains.data.value}
  <div class="relative">
    <Label>Transfer Timeline</Label>
    <div class="mt-4 space-y-8">
      {#each packetTraces as trace, i}
        {@const chain = getChain(
          chainsList,
          trace.chain.universal_chain_id,
        )}
        <!-- Timeline line -->
        <div
          class="absolute left-2 top-8 z-10 bottom-0 w-0.5 bg-zinc-200 dark:bg-zinc-700"
        ></div>

        <!-- Timeline item -->
        <div class="relative flex items-start gap-4 ml-6">
          <!-- Timeline dot -->
          <div class="absolute z-20 -left-[1.5rem]">
            <div
              class="h-4 w-4 rounded-full bg-zinc-300 dark:bg-zinc-600 ring-4 ring-white dark:ring-zinc-900"
            ></div>
          </div>

          <!-- Content -->
          <div
            class="flex-1 bg-zinc-50 dark:bg-zinc-900 px-4 rounded-lg"
          >
            <div class="flex items-center gap-2">
              <span
                class="font-medium text-zinc-900 dark:text-zinc-100"
              >
                {trace.type}
              </span>
              {#if Option.isSome(chain)}
                <ChainComponent chain={chain.value} />
              {:else}
                <span class="font-mono text-sm"
                  >{trace.chain.universal_chain_id}</span
                >
              {/if}
            </div>

            {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(trace.block_hash) &&  Option.isSome(chain)}
              <div
                class="flex flex-col gap-1 text-sm text-zinc-600 dark:text-zinc-400"
              >
                <div>
                  <DateTimeComponent value={trace.timestamp.value} />
                  <span>at height {trace.height.value}</span>
                </div>
                <div class="flex gap-2">
                  <span class="font-medium">Tx:</span>
                  <TransactionHashComponent
                    hash={trace.transaction_hash.value}
                  />
                </div>
                <div class="flex gap-2">
                  <span class="font-medium">Block:</span>
                  <BlockHashComponent chain={chain.value} hash={trace.block_hash.value} />
                </div>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}
