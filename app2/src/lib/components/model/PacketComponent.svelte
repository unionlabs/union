<script lang="ts">
import { Option } from "effect"
import { packetDetails } from "$lib/stores/packets.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "@unionlabs/sdk/schema"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import { fromHex } from "viem"
import LongMonoWord from "$lib/components/ui/LongMonoWord.svelte"
import TransactionHashComponent from "$lib/components/model/TransactionHashComponent.svelte"
import HeightComponent from "$lib/components/model/HeightComponent.svelte"
import BlockHashComponent from "$lib/components/model/BlockHashComponent.svelte"
import PacketTracesComponent from "$lib/components/model/PacketTracesComponent.svelte"
import { pipe } from "effect/Function"

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
</script>

{#if Option.isSome(packetDetails.error)}
  <ErrorComponent error={packetDetails.error.value} />
{:else if Option.isSome(packetDetails.data)}
  <div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
      <div>
        <Label>Packet Hash</Label>
        <div class="text-sm font-mono break-all">{packetDetails.data.value.packet_hash}</div>
      </div>
      <div>
        <Label>Channel Version</Label>
        <div class="">{packetDetails.data.value.channel_version}</div>
      </div>
    </div>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 gap-y-8 p-4">
      <div>
        <div class="flex gap-4">
          <div class="flex-1 ">
            <Label>Source Chain</Label>
            {#if Option.isSome(sourceChain)}
              <ChainComponent chain={sourceChain.value} />
            {:else}
              <div class="">{packetDetails.data.value.source_universal_chain_id}</div>
            {/if}
          </div>
          <div>
            <Label>Client</Label>
            <div class="">{packetDetails.data.value.source_client_id}</div>
          </div>
          <div>
            <Label>Conn</Label>
            <div class="">{packetDetails.data.value.source_connection_id}</div>
          </div>
          <div>
            <Label>Chan</Label>
            <div class="">{packetDetails.data.value.source_channel_id}</div>
          </div>
          
        </div>

        <div class="mt-2">
          <Label>Port</Label>
          <LongMonoWord>{ Option.isSome(sourceChain) && sourceChain.value.rpc_type === "cosmos" ? fromHex(packetDetails.data.value.source_port_id, "string") : packetDetails.data.value.source_port_id}</LongMonoWord>
        </div>        
      </div>
      <div>
        <div class="flex gap-4">
          <div class="flex-1">
            <Label>Destination Chain</Label>
            {#if Option.isSome(destinationChain)}
              <ChainComponent chain={destinationChain.value} />
            {:else}
              <div class="">{packetDetails.data.value.destination_universal_chain_id}</div>
            {/if}
          </div>
          <div>
            <Label>Client</Label>
            <div class="">{packetDetails.data.value.destination_client_id}</div>
          </div>
          <div>
            <Label>Conn</Label>
            <div class="">{packetDetails.data.value.destination_connection_id}</div>
          </div>
          <div>
            <Label>Chan</Label>
            <div class="">{packetDetails.data.value.destination_channel_id}</div>
          </div>
        </div>

        <div  class="mt-2">
          <Label>Port</Label>
          <LongMonoWord>{ Option.isSome(destinationChain) && destinationChain.value.rpc_type === "cosmos" ? fromHex(packetDetails.data.value.destination_port_id, "string") : packetDetails.data.value.destination_port_id}</LongMonoWord>
        </div>        
      </div>
    </div>

    <div class="flex md:grid md:grid-cols-2 gap-4 p-4">
      <div class="flex-1">
        <Label>Timeout Timestamp</Label>
        <div class="">{packetDetails.data.value.timeout_timestamp}</div>
      </div>
      <div>
        <Label>Height</Label>
        {#if Option.isSome(sourceChain)}
          <HeightComponent 
            height={packetDetails.data.value.timeout_height} 
            chain={sourceChain.value} 
          />
        {:else}
          <div class="">{packetDetails.data.value.timeout_height}</div>
        {/if}
      </div>
    </div>
    
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4">
      {#if Option.isSome(packetDetails.data.value.packet_ack_maker)}
        <section>
          <Label>Ack Maker</Label>
          {#if pipe(sourceChain, Option.map(c => c.rpc_type === "cosmos"), Option.getOrElse(() => false))}
            <LongMonoWord>{fromHex(packetDetails.data.value.packet_ack_maker.value, "string")}</LongMonoWord>
          {:else}
            <LongMonoWord>{packetDetails.data.value.packet_ack_maker.value}</LongMonoWord>
          {/if}
        </section>
      {/if}
      {#if Option.isSome(packetDetails.data.value.packet_recv_maker)}
        <section>
          <Label>Receive Maker</Label>
          {#if pipe(destinationChain, Option.map(c => c.rpc_type === "cosmos"), Option.getOrElse(() => false))}
            <LongMonoWord>{fromHex(packetDetails.data.value.packet_recv_maker.value, "string")}</LongMonoWord>
          {:else}
            <LongMonoWord>{packetDetails.data.value.packet_recv_maker.value}</LongMonoWord>
          {/if}
        </section>
      {/if}
    </div>
    
     <div class="p-4">
      <Label>Packet Data</Label>
      {#if Option.isSome(packetDetails.data.value.decoded)}
        <pre class="overflow-auto text-sm mt-2">{JSON.stringify(packetDetails.data.value.decoded.value, null, 2)}</pre>
      {:else}
        <div class=" text-zinc-500 mt-2">No data decoding available for this packet</div>
      {/if}
    </div>

    <div class="p-4">
      <Label>Raw Packet Data</Label>
      <LongMonoWord class="mt-2">{packetDetails.data.value.data}</LongMonoWord>
    </div>
    
    
    {#if Option.isSome(packetDetails.data.value.acknowledgement)}
      <div class="p-4">
        <Label>Acknowledgement</Label>
        <LongMonoWord class="mt-2">{packetDetails.data.value.acknowledgement.value}</LongMonoWord>
      </div>
    {/if}
    <PacketTracesComponent packetTraces={packetDetails.data.value.traces}/>
  </div>
{:else}
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
{/if}
