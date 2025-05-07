<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state"
import PacketComponent from "$lib/components/model/PacketComponent.svelte"
import PacketHashComponent from "$lib/components/model/PacketHashComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { packetDetails } from "$lib/stores/packets.svelte"
import { PacketHash } from "@unionlabs/sdk/schema"
import { Either, flow, pipe, Schema as S } from "effect"
import { onMount } from "svelte"
import type { PageData } from "./$types"

type Props = {
  data: PageData
}

const { data }: Props = $props()

onMount(() => {
  packetDetails.runEffect(packetDetailsQuery(data.packetHash))

  return () => {
    packetDetails.interruptFiber()
  }
})
</script>

<Sections>
  <Card divided>
    <PacketComponent />
  </Card>
</Sections>
