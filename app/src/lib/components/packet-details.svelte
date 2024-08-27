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
  source_chain_id: packet.from_chain_id,
  source_connection_id: packet.from_connection_id,
  source_channel_id: packet.from_channel_id,
  source_sequence: packet.source_sequence,
  destination_chain_id: packet.to_chain_id,
  destination_connection_id: packet.to_connection_id,
  destination_channel_id: packet.to_channel_id,
  destination_sequence: packet.destination_sequence
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
      <div>Source: {packet.from_port_id}</div>
      <div>Destination: {packet.to_port_id}</div>

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
