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
import { PACKET_TRACE_DISPLAY_NAMES } from "$lib/constants/packet-trace-names"

type Props = HTMLAttributes<HTMLDivElement> & {
  packetTraces: ReadonlyArray<PacketTrace>
}

const { packetTraces }: Props = $props()

const toTraceName = (type: string) =>
  type in PACKET_TRACE_DISPLAY_NAMES ? PACKET_TRACE_DISPLAY_NAMES[type] : type
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
            class="flex-1 px-4"
          >
            <div class="flex items-center">
              <span
                class="font-bold text-zinc-900 dark:text-zinc-100"
              >
                {toTraceName(trace.type)}  
              </span>
              &nbspon&nbsp
              {#if Option.isSome(chain)}
                <ChainComponent class="font-normal" chain={chain.value} />
              {:else}
                <span class="font-mono text-sm"
                  >{trace.chain.universal_chain_id}</span
                >
              {/if}
            </div>

            {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(trace.block_hash) &&  Option.isSome(chain)}
              <div
                class="flex flex-col text-sm text-zinc-600 dark:text-zinc-400"
              >
                <p><DateTimeComponent value={trace.timestamp.value} /> <span>in block {trace.height.value}</span></p>
                <TransactionHashComponent
                  hash={trace.transaction_hash.value}
                />
                <!-- uncomment for block hash, im unsure if it provides real value 
                <div class="flex gap-2">
                  <span class="font-medium">Block:</span>
                  <BlockHashComponent chain={chain.value} hash={trace.block_hash.value} />
                </div>
                !-->
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}
