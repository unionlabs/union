<script lang="ts">
import { page } from "$app/state"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import JsonPreview from "$lib/components/ui/JsonPreview.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import * as AppRuntime from "$lib/runtime"
import { chains } from "$lib/stores/chains.svelte"
import { Indexer } from "@unionlabs/sdk"
import { getChain, PacketHash } from "@unionlabs/sdk/schema"
import { ConfigProvider, Effect, Layer, Option, pipe } from "effect"
import * as O from "effect/Option"
import { graphql } from "gql.tada"

const packetHash = PacketHash.make(page.params.packet_hash)

// GraphQL config for development endpoint
const QlpConfigProvider = Layer.setConfigProvider(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://development.graphql.union.build/v1/graphql"],
    ]),
  ),
)

const unbondData = AppRuntime.runPromiseExit$(() => {
  void page.params.packet_hash

  return Effect.gen(function*() {
    const currentPacketHash = PacketHash.make(page.params.packet_hash)
    const indexer = yield* Indexer.Indexer
    const result = yield* indexer.fetch({
      document: graphql(`
        query GetUnbondByPacketHash($packet_hash: String!) @cached(ttl: 10) {
          v2_unbonds(args: { p_packet_hash: $packet_hash }) {
            packet_hash
            success
            source_universal_chain_id
            destination_universal_chain_id
            sender_display
            base_token
            base_amount
            unbond_send_timestamp
            unbond_send_transaction_hash
            sort_order
          }
        }
      `),
      variables: { packet_hash: currentPacketHash }
    })
    
    const unbonds = result.v2_unbonds as Array<any>
    
    if (unbonds.length === 0) {
      return yield* Effect.fail(new Error("Unbond not found"))
    }
    
    return unbonds[0]
  }).pipe(
    Effect.provide(Indexer.Indexer.Default),
    Effect.provide(QlpConfigProvider),
  )
})

const sourceChain = $derived(
  pipe(
    O.all([chains.data, unbondData.current]),
    O.flatMap(([chainsData, unbond]) => 
      unbond._tag === "Success" 
        ? getChain(chainsData, unbond.value.source_universal_chain_id)
        : O.none()
    )
  )
)

const destinationChain = $derived(
  pipe(
    O.all([chains.data, unbondData.current]),
    O.flatMap(([chainsData, unbond]) => 
      unbond._tag === "Success" 
        ? getChain(chainsData, unbond.value.destination_universal_chain_id)
        : O.none()
    )
  )
)
</script>

<Sections>
  <Card divided>
    {#if O.isSome(unbondData.current)}
      {#if unbondData.current.value._tag === "Success"}
        {@const unbond = unbondData.current.value.value}
        {@const status = unbond.success === true ? "success" : unbond.success === false ? "failure" : "pending"}
        
        <div class="p-6">
          <h1 class="text-2xl font-bold mb-4">Unbond Transaction</h1>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Unbond Details -->
            <div class="space-y-4">
              <div>
                <Label>Packet Hash</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {unbond.packet_hash}
                </div>
              </div>
              
              <div>
                <Label>Amount</Label>
                {#if O.isSome(sourceChain)}
                  <TokenComponent
                    chain={sourceChain.value}
                    denom={unbond.base_token}
                    amount={BigInt(unbond.base_amount)}
                    showIcon={true}
                  />
                {:else}
                  <div class="font-semibold font-mono">
                    {unbond.base_amount} {unbond.base_token}
                  </div>
                {/if}
              </div>
              
              <div>
                <Label>Status</Label>
                <div class="text-sm">
                  <span class="inline-flex items-center px-2 py-1 rounded text-xs font-medium {
                    status === 'success' 
                      ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/20 dark:text-emerald-400'
                      : status === 'failure'
                        ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
                        : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
                  }">
                    {status}
                  </span>
                </div>
              </div>
              
              <div>
                <Label>Timestamp</Label>
                {#if unbond.unbond_send_timestamp}
                  <div class="text-sm text-zinc-400">
                    {(() => {
                      try {
                        const date = new Date(unbond.unbond_send_timestamp)
                        return isNaN(date.getTime()) ? "Invalid date" : date.toLocaleString()
                      } catch {
                        return "Invalid date"
                      }
                    })()}
                  </div>
                {:else}
                  <div class="text-sm text-zinc-500">No timestamp</div>
                {/if}
              </div>
            </div>
            
            <!-- Chain Information -->
            <div class="space-y-4">
              <div>
                <Label>Source Chain</Label>
                {#if O.isSome(sourceChain)}
                  <ChainComponent chain={sourceChain.value} withToken={unbond.base_token} />
                {:else}
                  <div class="text-sm text-zinc-500">{unbond.source_universal_chain_id}</div>
                {/if}
              </div>
              
              <div>
                <Label>Destination Chain</Label>
                {#if O.isSome(destinationChain)}
                  <ChainComponent chain={destinationChain.value} />
                {:else}
                  <div class="text-sm text-zinc-500">{unbond.destination_universal_chain_id}</div>
                {/if}
              </div>
              
              <div>
                <Label>Sender</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {unbond.sender_display}
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Raw Data Section -->
        <details class="group border-t border-zinc-800">
          <summary class="flex cursor-pointer items-center justify-between px-6 py-4 text-sm text-zinc-400 hover:text-zinc-200">
            <span class="font-medium">Raw Unbond Data</span>
            <span class="transition-transform group-open:rotate-180">↓</span>
          </summary>
          <div class="px-6 pb-6">
            <JsonPreview value={unbond} />
          </div>
        </details>
        
      {:else if unbondData.current.value._tag === "Failure"}
        <div class="p-6">
          <ErrorComponent error={unbondData.current.value.error} />
        </div>
      {/if}
    {:else}
      <div class="p-6">
        <div class="animate-pulse space-y-4">
          <div class="h-8 bg-zinc-700 rounded w-1/3"></div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-4">
              {#each Array(4) as _}
                <div>
                  <div class="h-4 bg-zinc-700 rounded w-1/4 mb-2"></div>
                  <div class="h-6 bg-zinc-800 rounded"></div>
                </div>
              {/each}
            </div>
            <div class="space-y-4">
              {#each Array(4) as _}
                <div>
                  <div class="h-4 bg-zinc-700 rounded w-1/4 mb-2"></div>
                  <div class="h-6 bg-zinc-800 rounded"></div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </Card>
</Sections>
