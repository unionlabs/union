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
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { fromHex } from "viem"

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
  Option.flatMap(packetDetails.data, data =>
    Option.flatMap(chains.data, chainsData => getChain(chainsData, data.source_universal_chain_id))
  )
)

const destinationChain = $derived(
  Option.flatMap(packetDetails.data, data =>
    Option.flatMap(chains.data, chainsData =>
      getChain(chainsData, data.destination_universal_chain_id)
    )
  )
)

const goBack = () => {
  goto("/explorer/packets")
}
</script>

<Sections>
  {#if Option.isSome(packetDetails.error)}
    <Card>
      <ErrorComponent error={packetDetails.error.value} />
    </Card>
  {:else if Option.isSome(packetDetails.data)}
    <Card divided>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
        <div>
          <Label>Packet Hash</Label>
          <div class="text-sm break-all">{packetDetails.data.value.packet_hash}</div>
        </div>
        <div>
          <Label>Channel Version</Label>
          <div class="text-sm">{packetDetails.data.value.channel_version}</div>
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
          <Label>Source Client ID</Label>
          <div class="text-sm">{packetDetails.data.value.source_client_id}</div>
        </div>
        
        <div>
          <Label>Destination Client ID</Label>
          <div class="text-sm">{packetDetails.data.value.destination_client_id}</div>
        </div>
        
        <div>
          <Label>Source Connection ID</Label>
          <div class="text-sm">{packetDetails.data.value.source_connection_id}</div>
        </div>
        
        <div>
          <Label>Destination Connection ID</Label>
          <div class="text-sm">{packetDetails.data.value.destination_connection_id}</div>
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
          <Label>Source Port ID</Label>
          <div class="text-sm font-mono break-all">{fromHex(packetDetails.data.value.source_port_id, "string")}</div>
        </div>
        
        <div>
          <Label>Destination Port ID</Label>
          <div class="text-sm font-mono break-all">{fromHex(packetDetails.data.value.destination_port_id, "string")}</div>
        </div>
        <div>
          <Label>Send Timestamp</Label>
          <DateTimeComponent value={packetDetails.data.value.packet_send_timestamp} />
        </div>
        
        {#if Option.isSome(packetDetails.data.value.packet_recv_timestamp)}
          <div>
            <Label>Receive Timestamp</Label>
            <DateTimeComponent value={packetDetails.data.value.packet_recv_timestamp.value} />
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_recv_maker)}
          <div>
            <Label>Receive Maker</Label>
            <div class="text-sm break-all">
              <div>{packetDetails.data.value.packet_recv_maker.value}</div>
              <div class="text-xs text-zinc-500 mt-1">
                {fromHex(packetDetails.data.value.packet_recv_maker.value, "string")}
              </div>
            </div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_ack_timestamp)}
          <div>
            <Label>Ack Timestamp</Label>
            <DateTimeComponent value={packetDetails.data.value.packet_ack_timestamp.value} />
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_ack_maker)}
          <div>
            <Label>Ack Maker</Label>
            <div class="text-sm break-all">
              <div>{packetDetails.data.value.packet_ack_maker.value}</div>
              <div class="text-xs text-zinc-500 mt-1">
                {fromHex(packetDetails.data.value.packet_ack_maker.value, "string")}
              </div>
            </div>
          </div>
        {/if}
        
        <div>
          <Label>Status</Label>
          <div class="text-sm">{packetDetails.data.value.status}</div>
        </div>
      </div>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
        <!-- Send information -->
        {#if Option.isSome(packetDetails.data.value.packet_send_transaction_hash)}
          <div>
            <Label>Send Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_send_transaction_hash.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_send_height)}
          <div>
            <Label>Send Height</Label>
            <div class="text-sm">{packetDetails.data.value.packet_send_height.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_send_block_hash)}
          <div>
            <Label>Send Block Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_send_block_hash.value}</div>
          </div>
        {/if}
        
        <!-- Receive information -->
        {#if Option.isSome(packetDetails.data.value.packet_recv_transaction_hash)}
          <div>
            <Label>Receive Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_recv_transaction_hash.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_recv_height)}
          <div>
            <Label>Receive Height</Label>
            <div class="text-sm">{packetDetails.data.value.packet_recv_height.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_recv_block_hash)}
          <div>
            <Label>Receive Block Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_recv_block_hash.value}</div>
          </div>
        {/if}
        
        <!-- Ack information -->
        {#if Option.isSome(packetDetails.data.value.packet_ack_transaction_hash)}
          <div>
            <Label>Ack Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_ack_transaction_hash.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_ack_height)}
          <div>
            <Label>Ack Height</Label>
            <div class="text-sm">{packetDetails.data.value.packet_ack_height.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.packet_ack_block_hash)}
          <div>
            <Label>Ack Block Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.packet_ack_block_hash.value}</div>
          </div>
        {/if}
        
        <!-- Write Ack information -->
        {#if Option.isSome(packetDetails.data.value.write_ack_transaction_hash)}
          <div>
            <Label>Write Ack Transaction Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.write_ack_transaction_hash.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.write_ack_height)}
          <div>
            <Label>Write Ack Height</Label>
            <div class="text-sm">{packetDetails.data.value.write_ack_height.value}</div>
          </div>
        {/if}
        
        {#if Option.isSome(packetDetails.data.value.write_ack_block_hash)}
          <div>
            <Label>Write Ack Block Hash</Label>
            <div class="text-sm break-all">{packetDetails.data.value.write_ack_block_hash.value}</div>
          </div>
        {/if}
      </div>
      
      <div class="p-4">
        <Label>Packet Data</Label>
        <pre class="overflow-auto text-xs mt-2">{JSON.stringify(packetDetails.data.value.data, null, 2)}</pre>
      </div>
      
      <div class="p-4">
        <Label>Timeout Height</Label>
        <div class="text-sm">{packetDetails.data.value.timeout_height}</div>
      </div>
      
      {#if Option.isSome(Option.fromNullable(packetDetails.data.value.decoded))}
        <div class="p-4">
          <Label>Decoded Data</Label>
          <pre class="overflow-auto text-xs mt-2">{JSON.stringify(packetDetails.data.value.decoded, null, 2)}</pre>
        </div>
      {/if}
      
      {#if Option.isSome(packetDetails.data.value.acknowledgement)}
        <div class="p-4">
          <Label>Acknowledgement</Label>
          <pre class="overflow-auto text-xs mt-2">{JSON.stringify(packetDetails.data.value.acknowledgement.value, null, 2)}</pre>
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
