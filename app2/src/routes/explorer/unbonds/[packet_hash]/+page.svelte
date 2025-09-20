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
import { PacketHash, TokenRawAmount } from "@unionlabs/sdk/schema"
import { ConfigProvider, Effect, Layer, Option, pipe } from "effect"
import * as O from "effect/Option"
import { graphql } from "gql.tada"

const packetHash = $derived(PacketHash.make(page.params.packet_hash))

// GraphQL config for development endpoint
const QlpConfigProvider = Layer.setConfigProvider(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://development.graphql.union.build/v1/graphql"],
    ]),
  ),
)

const unbondData = $derived(pipe(
  Effect.gen(function*() {
    const indexer = yield* Indexer.Indexer
    const result = yield* indexer.fetch({
      document: graphql(`
        query GetUnbondByPacketHash($packet_hash: String!) @cached(ttl: 10) {
          v2_unbonds(args: { p_packet_hash: $packet_hash }) {
            packet_hash
            success
            packet_shape
            source_universal_chain_id
            destination_universal_chain_id
            sender_canonical
            sender_display
            sender_zkgm
            base_token
            base_amount
            unbond_send_timestamp
            unbond_send_transaction_hash
            unbond_recv_timestamp
            unbond_recv_transaction_hash
            unbond_timeout_timestamp
            unbond_timeout_transaction_hash
            sort_order
            source_chain {
              chain_id
              universal_chain_id
              display_name
              rpc_type
            }
            destination_chain {
              chain_id
              universal_chain_id
              display_name
              rpc_type
            }
          }
        }
      `),
      variables: { packet_hash: packetHash }
    })
    
    const unbonds = result.v2_unbonds as Array<any>
    
    if (unbonds.length === 0) {
      return yield* Effect.fail(new Error("Unbond not found"))
    }
    
    return unbonds[0]
  }),
  Effect.provide(Indexer.Indexer.Default),
  Effect.provide(QlpConfigProvider),
  Effect.runPromise
))

</script>

<Sections>
  <Card divided>
    {#await unbondData}
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
    {:then unbond}
      {@const status = unbond.success === true ? "success" : unbond.success === false ? "failure" : "pending"}
        
        <div class="p-6">
          <h1 class="text-2xl font-bold mb-4">Unbond</h1>
          
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
                <!--TODO, check amount type-->
                {#if unbond.source_chain}
                  <TokenComponent
                    chain={unbond.source_chain}
                    denom={unbond.base_token}
                    amount={BigInt(unbond.base_amount) as TokenRawAmount}
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

              <!-- Unbond Progress -->
              {#if unbond.unbond_send_timestamp && status === 'pending'}
                {@const sendTime = new Date(unbond.unbond_send_timestamp)}
                {@const now = new Date()}
                {@const unbondPeriodMs = 27 * 24 * 60 * 60 * 1000}
                {@const elapsedMs = now.getTime() - sendTime.getTime()}
                {@const progressPercent = Math.min(100, Math.max(0, (elapsedMs / unbondPeriodMs) * 100))}
                {@const remainingMs = Math.max(0, unbondPeriodMs - elapsedMs)}
                {@const remainingDays = Math.floor(remainingMs / (24 * 60 * 60 * 1000))}
                {@const remainingHours = Math.floor((remainingMs % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000))}
                
                <div>
                  <Label>Unbond Progress</Label>
                  <div class="space-y-2">
                    <div class="flex justify-between text-xs text-zinc-400">
                      <span>
                        {remainingDays > 0 ? `${remainingDays}d ${remainingHours}h remaining ±4h` : remainingHours > 0 ? `${remainingHours}h remaining ±4h` : 'Almost ready ±4h'}
                      </span>
                      <span>{progressPercent.toFixed(1)}%</span>
                    </div>
                    <div class="w-full bg-zinc-700 rounded-full h-2">
                      <div 
                        class="bg-accent h-2 rounded-full transition-all duration-300" 
                        style="width: {progressPercent}%"
                      ></div>
                    </div>
                  </div>
                </div>
              {/if}
              
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
                {#if unbond.source_chain}
                  <ChainComponent chain={unbond.source_chain} withToken={unbond.base_token} />
                {:else}
                  <div class="text-sm text-zinc-500">{unbond.source_universal_chain_id}</div>
                {/if}
              </div>
              
              <div>
                <Label>Destination Chain</Label>
                {#if unbond.destination_chain}
                  <ChainComponent chain={unbond.destination_chain} withToken={unbond.base_token} />
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
        
    {:catch error}
      <div class="p-6">
        <ErrorComponent {error} />
      </div>
    {/await}
  </Card>
</Sections>
