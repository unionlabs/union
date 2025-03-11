<script lang="ts">
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { Effect, Option } from "effect"
import { onMount } from "svelte"
import { packetDetails } from "$lib/stores/packets.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import { page } from "$app/state"
import { goto } from "$app/navigation"

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

const sourceChain = $derived(
  Option.flatMap(
    Option.flatMap(packetDetails.data, data => 
      Option.flatMap(chains.data, chainsData =>
        getChain(chainsData, data.source_universal_chain_id)
      )
    )
  )
)

const destinationChain = $derived(
  Option.flatMap(
    Option.flatMap(packetDetails.data, data => 
      Option.flatMap(chains.data, chainsData =>
        getChain(chainsData, data.destination_universal_chain_id)
      )
    )
  )
)

const goBack = () => {
  goto("/explorer/packets")
}
</script>

<Sections>
  <Button onclick={goBack} class="mb-4">← Back to Packets</Button>
  
  {#if Option.isSome(packetDetails.error)}
    <Card>
      <ErrorComponent error={packetDetails.error.value} />
    </Card>
  {:else if Option.isSome(packetDetails.data)}
    <Card>
      <SectionTitle>Packet Details</SectionTitle>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
        <div>
          <Label>Packet Hash</Label>
          <div class="text-sm break-all">{packetDetails.data.value.packet_hash}</div>
        </div>
        
        <div>
          <Label>Status</Label>
          <div class="text-sm">{packetDetails.data.value.status}</div>
        </div>
        
        <div>
          <Label>Source Chain</Label>
          {#if Option.isSome(sourceChain)}
            <ChainComponent chain={sourceChain.value} />
          {:else}
            <div class="text-sm">{packetDetails.data.value.source_universal_chain_id}</div>
          {/if}
        </div>
        
        <div>
          <Label>Destination Chain</Label>
          {#if Option.isSome(destinationChain)}
            <ChainComponent chain={destinationChain.value} />
          {:else}
            <div class="text-sm">{packetDetails.data.value.destination_universal_chain_id}</div>
          {/if}
        </div>
        
        <div>
          <Label>Channel Version</Label>
          <div class="text-sm">{packetDetails.data.value.channel_version}</div>
        </div>
        
        <div>
          <Label>Source Channel ID</Label>
          <div class="text-sm">{packetDetails.data.value.source_channel_id}</div>
        </div>
        
        <div>
          <Label>Destination Channel ID</Label>
          <div class="text-sm">{packetDetails.data.value.destination_channel_id}</div>
        </div>
        
        <div>
          <Label>Send Timestamp</Label>
          <DateTimeComponent value={packetDetails.data.value.packet_send_timestamp} />
        </div>
        
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.packet_recv_timestamp))}
          <div>
            <Label>Receive Timestamp</Label>
            <DateTimeComponent value={packetDetails.data.value.packet_recv_timestamp} />
          </div>
        {/if}
        
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.packet_ack_timestamp))}
          <div>
            <Label>Ack Timestamp</Label>
            <DateTimeComponent value={packetDetails.data.value.packet_ack_timestamp} />
          </div>
        {/if}
      </div>
    </Card>
    
    <Card>
      <SectionTitle>Transaction Details</SectionTitle>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.packet_send_transaction_hash))}
          <div>
            <Label>Send Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_send_transaction_hash}</div>
          </div>
        {/if}
        
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.packet_recv_transaction_hash))}
          <div>
            <Label>Receive Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_recv_transaction_hash}</div>
          </div>
        {/if}
        
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.packet_ack_transaction_hash))}
          <div>
            <Label>Ack Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_ack_transaction_hash}</div>
          </div>
        {/if}
        
        {#if Option.isSome(Option.fromNullable(packetDetails.data.value.write_ack_transaction_hash))}
          <div>
            <Label>Write Ack Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.write_ack_transaction_hash}</div>
          </div>
        {/if}
      </div>
    </Card>
    
    <Card>
      <SectionTitle>Packet Data</SectionTitle>
      
      <div class="p-4">
        <pre class="bg-zinc-900 p-4 rounded overflow-auto text-xs">{JSON.stringify(packetDetails.data.value.data, null, 2)}</pre>
      </div>
      
      {#if Option.isSome(Option.fromNullable(packetDetails.data.value.decoded))}
        <SectionTitle>Decoded Data</SectionTitle>
        <div class="p-4">
          <pre class="bg-zinc-900 p-4 rounded overflow-auto text-xs">{JSON.stringify(packetDetails.data.value.decoded, null, 2)}</pre>
        </div>
      {/if}
      
      {#if Option.isSome(Option.fromNullable(packetDetails.data.value.acknowledgement))}
        <SectionTitle>Acknowledgement</SectionTitle>
        <div class="p-4">
          <pre class="bg-zinc-900 p-4 rounded overflow-auto text-xs">{JSON.stringify(packetDetails.data.value.acknowledgement, null, 2)}</pre>
        </div>
      {/if}
    </Card>
  {:else}
    <Card>
      <div class="p-4">
        <Skeleton class="h-8 w-full mb-4" />
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          {#each Array(8).fill(0)}
            <div>
              <Skeleton class="h-4 w-24 mb-2" />
              <Skeleton class="h-6 w-full" />
            </div>
          {/each}
        </div>
      </div>
    </Card>
  {/if}
</Sections>
