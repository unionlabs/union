<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import type { Chain } from "$lib/types"
import { toDisplayName } from "$lib/utilities/chains"
import MoveRightIcon from "virtual:icons/lucide/move-right"
import ChainDetails from "$lib/chain-details.svelte"
export let chains: Array<Chain>
export let packet: {
  source_chain_id: string
  source_connection_id?: number | null
  source_channel_id?: number | null
  destination_chain_id: string
  destination_connection_id?: number | null
  destination_channel_id?: number | null
}
</script>

<section class="flex flex-col sm:flex-row">
  <div class="flex-1 lex-col text-muted-foreground">
    <h2
      class="font-supermolot capitalize md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap"
    >
      <ChainDetails highlightEnabled={false} {chains} chainId={packet.source_chain_id}/>
    </h2>
    <div class="flex divide-x-2 divide-muted-background">
    <a 
      href={`/explorer/packets/${packet.source_chain_id}`}
      class="pr-2 block text-sm underline text-muted-foreground">
    
      {packet.source_chain_id}
    </a>
    {#if packet.source_connection_id}
      <a
        href={`/explorer/packets/${packet.source_chain_id}/${packet.source_connection_id}`}
        class="px-2 black text-sm underline text-muted-foreground"
      >
        {packet.source_connection_id}
      </a>
    
      {#if packet.source_channel_id}
        <a
          href={`/explorer/packets/${packet.source_chain_id}/${packet.source_connection_id}/${packet.source_channel_id}`}
          class="pl-2 text-sm block underline text-muted-foreground"
        >
          {packet.source_channel_id}
        </a>
      {/if}
    {/if}
    </div>
  </div>
  <div class="flex items-center justify-center px-8">
    <MoveRightIcon class="text-foreground size-8" />
  </div>
  <div class="flex-1 sm:text-right flex-col text-muted-foreground">
    <h2
      class="font-supermolot capitalize md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap"
    >
      <ChainDetails highlightEnabled={false} {chains} chainId={packet.destination_chain_id}/>
    </h2>
    <div class="flex justify-end divide-x-2 divide-muted-background">
    <a 
      href={`/explorer/packets/${packet.destination_chain_id}`}
      class="pr-2 block text-sm underline text-muted-foreground">
    
      {packet.destination_chain_id}
    </a>
    {#if packet.destination_connection_id}
      <a
        href={`/explorer/packets/${packet.destination_chain_id}/${packet.destination_connection_id}`}
        class="px-2 text-sm block underline text-muted-foreground"
      >
        {packet.destination_connection_id}
      </a>
      {#if packet.destination_channel_id}
        <a
          href={`/explorer/packets/${packet.destination_chain_id}/${packet.destination_connection_id}/${packet.destination_channel_id}`}
          class="pl-2 text-sm block underline text-muted-foreground"
        >
        {packet.destination_channel_id}
      </a>
      {/if}
    {/if}
    </div>
  </div>
</section>
