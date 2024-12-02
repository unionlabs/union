<script lang="ts">
import { packetDetailsFragment } from "$lib/graphql/fragments/packets.ts"
import { readFragment, type FragmentOf } from "gql.tada"
import type { Chain } from "$lib/types"

import * as Card from "$lib/components/ui/card/index.ts"
import PacketPath from "./packet-path.svelte"
import DetailsHeading from "./details-heading.svelte"

export let packetDetails: FragmentOf<typeof packetDetailsFragment>
const packet = readFragment(packetDetailsFragment, packetDetails)
export let chains: Array<Chain>

const packetSourceDestination = {
  source_chain_id: packet.source_chain_id ?? "Undefined",
  source_connection_id: packet.source_connection_id ?? "Undefined",
  source_channel_id: packet.source_channel_id ?? "Undefined",
  source_sequence: packet.source_sequence?.toString() ?? "Undefined",
  destination_chain_id: packet.destination_chain_id ?? "Undefined",
  destination_connection_id: packet.destination_connection_id ?? "Undefined",
  destination_channel_id: packet.destination_channel_id ?? "Undefined",
  destination_sequence: packet.destination_sequence?.toString() ?? "Undefined"
}
</script>  


<Card.Root class="break-words">
  <Card.Header
    class="font-bold text-md text-center break-words text-muted-foreground flex flex-row gap-2 justify-center"
  >
    PACKET {packet.source_sequence}
  </Card.Header>
  <Card.Content class="flex flex-col gap-8">
    <PacketPath {chains} packet={packetSourceDestination}/>

    <section>
      <DetailsHeading>
        Ports
      </DetailsHeading>
      <div>Source: {packet.source_port_id}</div>
      <div>Destination: {packet.destination_port_id}</div>

    </section>
    <section>
      <DetailsHeading>
        Data
      </DetailsHeading>
      <div class="overflow-x-scroll">
      <pre class="text-sm">{JSON.stringify(packet, null, 2)}</pre>
      </div>
    </section>
  </Card.Content>
</Card.Root>
