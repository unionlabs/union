<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import type { Chain } from "$lib/types"
import { toDisplayName } from "$lib/utilities/chains"
import MoveRightIcon from "virtual:icons/lucide/move-right"
export let chains: Array<Chain>
export let packet: {
  source_chain_id: string
  source_connection_id: number
  source_channel_id: number
  destination_chain_id: string
  destination_connection_id: number
  destination_channel_id: number
}
</script>

<section class="flex flex-col sm:flex-row">
  <div class="flex-1 lex-col text-muted-foreground">
    <h2
      class="font-supermolot uppercase md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap"
    >
      {toDisplayName(packet.source_chain_id, chains)}
    </h2>
    <div class="flex divide-x-2 divide-muted-background">
    <a 
      href={`/explorer/packets/${packet.source_chain_id}`}
      class="pr-2 block text-sm underline text-muted-foreground">
    
      {packet.source_chain_id}
    </a>
     
    <a
      href={`/explorer/packets/${packet.source_chain_id}/${packet.source_connection_id}`}
      class={cn(
        "px-2 black text-sm underline",
        packet.source_connection_id
          ? "text-muted-foreground"
          : "text-transparent"
      )}
    >
      {packet.source_connection_id}
    </a>
    
    <a
      href={`/explorer/packets/${packet.source_chain_id}/${packet.source_connection_id}/${packet.source_channel_id}`}
      class={cn(
        "pl-2 text-sm block underline",
        packet.source_channel_id
          ? "text-muted-foreground"
          : "text-transparent"
      )}
    >
      {packet.source_channel_id}
    </a>
    </div>
  </div>
  <div class="flex items-center justify-center px-8">
    <MoveRightIcon class="text-foreground size-8" />
  </div>
  <div class="flex-1 sm:text-right flex-col text-muted-foreground">
    <h2
      class="font-supermolot uppercase md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap"
    >
      {toDisplayName(packet.destination_chain_id, chains)}
    </h2>
    <div class="flex justify-end divide-x-2 divide-muted-background">
    <a 
      href={`/explorer/packets/${packet.destination_chain_id}`}
      class="pr-2 block text-sm underline text-muted-foreground">
    
      {packet.destination_chain_id}
    </a>
    <a
      href={`/explorer/packets/${packet.destination_chain_id}/${packet.destination_connection_id}`}
      class={cn(
        "px-2 text-sm block underline",
        packet.destination_connection_id
          ? "text-muted-foreground"
          : "text-transparent"
      )}
    >
      {packet.destination_connection_id}
    </a>
    <a
      href={`/explorer/packets/${packet.destination_chain_id}/${packet.destination_connection_id}/${packet.destination_channel_id}`}
      class={cn(
        "pl-2 text-sm block underline",
        packet.destination_channel_id
          ? "text-muted-foreground"
          : "text-transparent"
      )}
    >
      {packet.destination_channel_id}
    </a>
    </div>
  </div>
</section>
