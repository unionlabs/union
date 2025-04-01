<script lang="ts">
import { page } from "$app/state"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { transferByPacketHashQuery } from "$lib/queries/transfer-by-hash.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import Label from "$lib/components/ui/Label.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import TransactionHashComponent from "$lib/components/model/TransactionHashComponent.svelte"
import BlockHashComponent from "$lib/components/model/BlockHashComponent.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { getChain } from "@unionlabs/sdk/schema"
import PacketComponent from "$lib/components/model/PacketComponent.svelte"
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { packetDetails } from "$lib/stores/packets.svelte"

// Store for the transfer details
import { transferDetails } from "$lib/stores/transfer-details.svelte"
import SharpRightArrowIcon from "$lib/components/icons/SharpRightArrowIcon.svelte"

let fiber: Fiber.Fiber<any, any>
const packetHash = page.params.packet_hash

onMount(() => {
  fiber = Effect.runFork(transferByPacketHashQuery(packetHash))
  packetDetails.runEffect(packetDetailsQuery(packetHash))

  return async () => {
    await Effect.runPromise(Fiber.interrupt(fiber))
    transferDetails.data = Option.none()
    transferDetails.error = Option.none()

    // Clean up packet details
    packetDetails.interruptFiber()
  }
})
</script>

<Sections>
  <Card class="overflow-auto p-6 transition-size" divided>
    {#if Option.isSome(transferDetails.error)}
      <ErrorComponent error={transferDetails.error.value} />
    {:else if Option.isSome(transferDetails.data) && Option.isSome(chains.data)}
      {@const transfer = transferDetails.data.value}
      {@const chainsList = chains.data.value}
      {@const sourceChain = getChain(
        chainsList,
        transfer.source_chain.universal_chain_id,
      )}
      {@const destChain = getChain(
        chainsList,
        transfer.destination_chain.universal_chain_id,
      )}

      <div class="space-y-8">
        <!-- Chain and Token Transfer Display -->
        <div class="flex flex-col items-center gap-2">
          <div class="text-2xl">
            {#if !settingsStore.showQuoteTokens}
              <TokenComponent
                chain={sourceChain.value}
                denom={transfer.base_token}
                amount={transfer.base_amount}
              />
            {/if}
          </div>
          <div class="flex items-center gap-4">
            {#if Option.isSome(sourceChain)}
              <div class="flex flex-col items-end">
                {#if settingsStore.showQuoteTokens}
                  <TokenComponent
                    chain={sourceChain.value}
                    denom={transfer.base_token}
                    amount={transfer.base_amount}
                  />
                {/if}
                <div class="flex flex-col items-end">
                  {#if Option.isSome(sourceChain)}
                    <ChainComponent chain={sourceChain.value} />
                    <AddressComponent
                      address={transfer.sender_canonical}
                      chain={sourceChain.value}
                      class="text-zinc-400"
                    />
                  {:else}
                    <div>{transfer.source_chain.chain_id}</div>
                    <div class="font-mono text-sm text-zinc-500">{transfer.sender_canonical}</div>
                  {/if}
                </div>
              </div>
              
              <div class="flex flex-col items-center gap-1">
                <SharpRightArrowIcon class="w-8 h-8 text-zinc-400" />
              </div>
              
              <div class="flex flex-col items-start">
                {#if settingsStore.showQuoteTokens && Option.isSome(destChain)}
                  <TokenComponent
                    chain={destChain.value}
                    denom={transfer.quote_token}
                    amount={transfer.quote_amount}
                  />
                {/if}
                {#if Option.isSome(destChain)}
                  <ChainComponent chain={destChain.value} />
                  <AddressComponent
                    address={transfer.receiver_canonical}
                    chain={destChain.value}
                    class="text-zinc-400"
                  />
                {:else}
                  <div>{transfer.destination_chain.chain_id}</div>
                  <div class="font-mono text-sm text-zinc-500">{transfer.receiver_canonical}</div>
                {/if}
              </div>
            {/if}
            </div>
              <div class="text-sm text-zinc-500 flex flex-col items-center">
                <DateTimeComponent value={transfer.transfer_send_timestamp} showSeconds={false} />
                {#if Option.isSome(transfer.transfer_recv_timestamp)}
                  <DateTimeComponent value={transfer.transfer_recv_timestamp.value} showSeconds={false} />
                {/if}
              </div>
          </div>



        {#if transfer.traces.length > 0}
          <div class="relative">
            <Label>Transfer Timeline</Label>
            <div class="mt-4 space-y-8">
              {#each transfer.traces as trace, i}
                {@const chain = getChain(chainsList, trace.chain.universal_chain_id)}
                <!-- Timeline line -->
                <div class="absolute left-2 top-8 z-10 bottom-0 w-0.5 bg-zinc-200 dark:bg-zinc-700" ></div>
                
                <!-- Timeline item -->
                <div class="relative flex items-start gap-4 ml-6">
                  <!-- Timeline dot -->
                  <div class="absolute z-20 -left-[1.5rem]">
                    <div class="h-4 w-4 rounded-full bg-zinc-300 dark:bg-zinc-600 ring-4 ring-white dark:ring-zinc-900" ></div>
                  </div>
                  
                  <!-- Content -->
                  <div class="flex-1 bg-zinc-50 dark:bg-zinc-900 px-4 rounded-lg">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-zinc-900 dark:text-zinc-100">
                        {trace.type}
                      </span>
                      {#if Option.isSome(chain)}
                        <ChainComponent chain={chain.value} />
                      {:else}
                        <span class="font-mono text-sm">{trace.chain.universal_chain_id}</span>
                      {/if}
                    </div>
                    
                    {#if Option.isSome(trace.height) && Option.isSome(trace.timestamp) && Option.isSome(trace.transaction_hash) && Option.isSome(trace.block_hash)}
                      <div class="flex flex-col gap-1 text-sm text-zinc-600 dark:text-zinc-400">
                        <div>
                          <DateTimeComponent value={trace.timestamp.value} />
                          <span>at height {trace.height.value}</span>
                        </div>
                        <div class="flex gap-2">
                          <span class="font-medium">Tx:</span> <TransactionHashComponent hash={trace.transaction_hash.value} />
                        </div>
                        <div class="flex gap-2">
                          <span class="font-medium">Block:</span>
                          <BlockHashComponent hash={trace.block_hash.value} />
                        </div>
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </Card>
  
  <!-- Packet Details Card -->
  <Card divided>
    <div class="p-4">
      Packet Details
    </div>
    <PacketComponent/>
  </Card>
</Sections>
