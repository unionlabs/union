<script lang="ts">
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { Effect } from "effect"
import { onMount } from "svelte"
import { packetDetails } from "$lib/stores/packets.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { page } from "$app/state"
import { goto } from "$app/navigation"
import PacketComponent from "$lib/components/model/PacketComponent.svelte"

onMount(() => {
  const packetHash = page.params.packet_hash
  if (!packetHash) {
    goto("/explorer/packets")
    return
  }

  packetDetails.runEffect(packetDetailsQuery(packetHash))

  return () => {
    packetDetails.interruptFiber()
  }
})
</script>

<Sections>
  <Card divided>
    <PacketComponent/>
  </Card>
</Sections>
