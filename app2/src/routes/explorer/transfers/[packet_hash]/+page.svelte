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
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import TransactionHashComponent from "$lib/components/model/TransactionHashComponent.svelte"
import BlockHashComponent from "$lib/components/model/BlockHashComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "$lib/schema/chain"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

// Store for the transfer details
import { transferDetails } from "$lib/stores/transfer-details.svelte"

let fiber: Fiber.Fiber<any, any>
const packetHash = $page.params.packet_hash

onMount(() => {
  fiber = Effect.runFork(transferByPacketHashQuery(packetHash))
  return async () => {
    await Effect.runPromise(Fiber.interrupt(fiber))
    transferDetails.data = Option.none()
    transferDetails.error = Option.none()
  }
})
</script>

<Sections>
  <Card class="overflow-auto p-6" divided>
    {#if Option.isSome(transferDetails.error)}
      <ErrorComponent error={transferDetails.error.value} />
    {:else if Option.isSome(transferDetails.data) && Option.isSome(chains.data)}
      {@const transfer = transferDetails.data.value}
      {@const chainsList = chains.data.value}
      {@const sourceChain = getChain(
        chainsList,
        transfer.source_chain.chain_id,
      )}
      {@const destChain = getChain(
        chainsList,
        transfer.destination_chain.chain_id,
      )}

      <div class="space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label>Source Chain</Label>
            {#if Option.isSome(sourceChain)}
              <ChainComponent chain={sourceChain.value} />
            {:else}
              <div>{transfer.source_chain.chain_id}</div>
            {/if}
          </div>

          <div>
            <Label>Destination Chain</Label>
            {#if Option.isSome(destChain)}
              <ChainComponent chain={destChain.value} />
            {:else}
              <div>{transfer.destination_chain.chain_id}</div>
            {/if}
          </div>

          <!-- <div>
            <Label>Source Connection ID</Label>
            <div class="font-mono text-sm">{transfer.source_connection_id}</div>
          </div>

          <div>
            <Label>Destination Connection ID</Label>
            <div class="font-mono text-sm">{transfer.destination_connection_id}</div>
          </div>

          <div>
            <Label>Source Channel ID</Label>
            <div class="font-mono text-sm">{transfer.source_channel_id}</div>
          </div>

          <div>
            <Label>Destination Channel ID</Label>
            <div class="font-mono text-sm">{transfer.destination_channel_id}</div>
          </div> -->
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label>Sender</Label>
            <div>
              {transfer.sender_canonical}
            </div>
          </div>

          <div>
            <Label>Receiver</Label>
            <div>
              {transfer.receiver_canonical}
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label>Base Token</Label>
            {#if Option.isSome(sourceChain)}
              <TokenComponent
                chain={sourceChain.value}
                denom={transfer.base_token}
                amount={transfer.base_amount}
              />
            {/if}
          </div>

          <div>
            <Label>Quote Token</Label>
            {#if Option.isSome(destChain)}
              <TokenComponent
                chain={destChain.value}
                denom={transfer.quote_token}
                amount={transfer.quote_amount}
              />
            {/if}
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <Label>Send Transaction Hash</Label>
            <TransactionHashComponent
              hash={transfer.transfer_send_transaction_hash}
            />
          </div>

          <div>
            <Label>Send Timestamp</Label>
            <div>{DateTime.formatIso(transfer.transfer_send_timestamp)}</div>
          </div>

          <div>
            <Label>Receive Timestamp</Label>
            {#if Option.isSome(transfer.transfer_recv_timestamp)}
              <div>
                {DateTime.formatIso(transfer.transfer_recv_timestamp.value)}
              </div>
            {:else}
              <div class="text-yellow-500">Pending</div>
            {/if}
          </div>
        </div>

        {#if transfer.traces.length > 0}
          <div>
            <Label>Traces</Label>
            <div class="space-y-2">
              {#each transfer.traces as trace}
                {@const chain = getChain(chainsList, trace.chain.chain_id)}
                <div class="bg-zinc-100 dark:bg-zinc-800 p-4 rounded">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <Label>Type</Label>
                      <div class="font-mono text-sm">{trace.type}</div>
                    </div>
                    <div>
                      <Label>Chain</Label>
                      {#if Option.isSome(chain)}
                        <ChainComponent chain={chain.value} />
                      {:else}
                        <div class="font-mono text-sm">
                          {trace.chain.chain_id}
                        </div>
                      {/if}
                    </div>
                    {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(trace.block_hash)}
                      <div>
                        <Label>Height</Label>
                        <div class="font-mono text-sm">
                          {trace.height.value}
                        </div>
                      </div>
                      <div>
                        <Label>Timestamp</Label>
                        <div class="font-mono text-sm">
                          {DateTime.formatIso(trace.timestamp.value)}
                        </div>
                      </div>
                      <div class="col-span-2">
                        <Label>Transaction Hash</Label>
                        <TransactionHashComponent
                          hash={trace.transaction_hash.value}
                        />
                      </div>
                      <div class="col-span-2">
                        <Label>Block Hash</Label>
                        <BlockHashComponent hash={trace.block_hash.value} />
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {#each Array(6) as _}
            <Skeleton class="h-10" />
          {/each}
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {#each Array(2) as _}
            <Skeleton class="h-16" />
          {/each}
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {#each Array(4) as _}
            <Skeleton class="h-10" />
          {/each}
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {#each Array(3) as _}
            <Skeleton class="h-10" />
          {/each}
        </div>

        <div>
          <Skeleton class="h-40" />
        </div>
      </div>
    {/if}
  </Card>
</Sections>
