<script lang="ts">
import { page } from "$app/stores"
import { onMount } from "svelte"
import { DateTime, Effect, Fiber, Option } from "effect"
import { transferByPacketHashQuery } from "$lib/queries/transfer-by-hash.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import Label from "$lib/components/ui/Label.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

// Store for the transfer details
import { transferDetails } from "$lib/stores/transfer-details.svelte"

let fiber: Fiber.Fiber<any, any>
const packetHash = $page.params.packet_hash

onMount(() => {
  fiber = Effect.runFork(transferByPacketHashQuery(packetHash))
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})
</script>

<Sections>
  <div class="flex items-center gap-4">
    <a href="/explorer/transfers" class="text-blue-500 hover:underline">
      &larr; Back to Transfers
    </a>
    <h1 class="font-bold text-2xl">Transfer Details</h1>
  </div>

  <Card class="overflow-auto p-6" divided>
    {#if Option.isSome(transferDetails.error)}
      <ErrorComponent error={transferDetails.error.value}/>
    {:else if Option.isSome(transferDetails.data) && Option.isSome(chains.data)}
      {@const transfer = transferDetails.data.value}
      {@const chainsList = chains.data.value}
      {@const sourceChain = getChain(chainsList, transfer.source_chain_id)}
      {@const destChain = getChain(chainsList, transfer.destination_chain_id)}

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <Label>Packet Hash</Label>
          <div class="font-mono text-sm break-all bg-gray-100 dark:bg-gray-800 p-2 rounded">
            {transfer.packet_hash}
          </div>
        </div>

        <div>
          <Label>Sort Order</Label>
          <div class="font-mono text-sm break-all bg-gray-100 dark:bg-gray-800 p-2 rounded">
            {transfer.sort_order}
          </div>
        </div>

        <div>
          <Label>Source Chain</Label>
          {#if Option.isSome(sourceChain)}
            <ChainComponent chain={sourceChain.value} />
          {:else}
            <div>{transfer.source_chain_id}</div>
          {/if}
        </div>

        <div>
          <Label>Destination Chain</Label>
          {#if Option.isSome(destChain)}
            <ChainComponent chain={destChain.value} />
          {:else}
            <div>{transfer.destination_chain_id}</div>
          {/if}
        </div>

        <div>
          <Label>Sender</Label>
          <div class="font-mono text-sm break-all bg-gray-100 dark:bg-gray-800 p-2 rounded">
            {transfer.sender_normalized}
          </div>
        </div>

        <div>
          <Label>Receiver</Label>
          <div class="font-mono text-sm break-all bg-gray-100 dark:bg-gray-800 p-2 rounded">
            {transfer.receiver_normalized}
          </div>
        </div>

        <div>
          <Label>Send Timestamp</Label>
          <div>{DateTime.formatIso(transfer.packet_send_timestamp)}</div>
        </div>

        <div>
          <Label>Receive Timestamp</Label>
          {#if Option.isSome(transfer.packet_recv_timestamp)}
            <div>{DateTime.formatIso(transfer.packet_recv_timestamp.value)}</div>
          {:else}
            <div class="text-yellow-500">Pending</div>
          {/if}
        </div>
      </div>

    {:else}
      <div class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
          <Skeleton class="h-10" />
        </div>
      </div>
    {/if}
  </Card>
</Sections>
