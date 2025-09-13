<script lang="ts">
import { page } from "$app/state"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import JsonPreview from "$lib/components/ui/JsonPreview.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import * as AppRuntime from "$lib/runtime"
import { Indexer } from "@unionlabs/sdk"
import { PacketHash, TokenRawAmount } from "@unionlabs/sdk/schema"
import { ConfigProvider, Effect, Layer, pipe } from "effect"
import * as O from "effect/Option"
import { graphql } from "gql.tada"

const packetHash = $derived(PacketHash.make(page.params.packet_hash))

const QlpConfigProvider = Layer.setConfigProvider(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://development.graphql.union.build/v1/graphql"],
    ]),
  ),
)

const bondData = $derived(pipe(
    Effect.gen(function*() {
      const indexer = yield* Indexer.Indexer
      const result = yield* indexer.fetch({
        document: graphql(`
          query GetBondByPacketHash($packet_hash: String!) @cached(ttl: 10) {
            v2_bonds(args: { p_packet_hash: $packet_hash }) {
              packet_hash
              delivery_packet_hash
              bond_success
              delivery_success
              packet_shape
              source_universal_chain_id
              remote_universal_chain_id
              destination_universal_chain_id
              sender_canonical
              sender_display
              sender_zkgm
              receiver_canonical
              receiver_display
              receiver_zkgm
              base_token
              base_amount
              quote_token
              quote_amount
              remote_base_token
              remote_base_amount
              remote_quote_token
              remote_quote_amount
              bond_send_timestamp
              bond_send_transaction_hash
              bond_recv_timestamp
              bond_recv_transaction_hash
              bond_timeout_timestamp
              bond_timeout_transaction_hash
              delivery_send_timestamp
              delivery_send_transaction_hash
              delivery_recv_timestamp
              delivery_recv_transaction_hash
              delivery_timeout_timestamp
              delivery_timeout_transaction_hash
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
      
      const bonds = result.v2_bonds as Array<any>
      
      if (bonds.length === 0) {
        return yield* Effect.fail(new Error("Bond not found"))
      }
      
      return bonds[0]
    }),
    Effect.provide(Indexer.Indexer.Default),
    Effect.provide(QlpConfigProvider),
    Effect.runPromise
  )
)

</script>

<Sections>
  <Card divided>
    {#await bondData}
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
    {:then bond}
      {@const status = bond.bond_success === true ? "success" : bond.bond_success === false ? "failure" : "pending"}
        
        <div class="p-6">
          <h1 class="text-2xl font-bold mb-4">Bond</h1>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Bond Details -->
            <div class="space-y-4">
              <div>
                <Label>Packet Hash</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {bond.packet_hash}
                </div>
              </div>
              
              <div>
                <Label>Amount</Label>
                <!--TODO, check amount type-->
                {#if bond.source_chain}
                  <TokenComponent
                    chain={bond.source_chain}
                    denom={bond.base_token}
                    amount={BigInt(bond.base_amount) as TokenRawAmount}
                    showIcon={true}
                  />
                {:else}
                  <div class="font-semibold font-mono">
                    {bond.base_amount} {bond.base_token}
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
                {#if bond.bond_send_timestamp}
                  <div class="text-sm text-zinc-400">
                    {(() => {
                      try {
                        const date = new Date(bond.bond_send_timestamp)
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
                {#if bond.source_chain}
                  <ChainComponent chain={bond.source_chain} withToken={bond.base_token} />
                {:else}
                  <div class="text-sm text-zinc-500">{bond.source_universal_chain_id}</div>
                {/if}
              </div>
              
              <div>
                <Label>Destination Chain</Label>
                {#if bond.destination_chain}
                  <ChainComponent chain={bond.destination_chain} withToken={bond.quote_token} />
                {:else}
                  <div class="text-sm text-zinc-500">{bond.destination_universal_chain_id}</div>
                {/if}
              </div>
              
              <div>
                <Label>Sender</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {bond.sender_display}
                </div>
              </div>
              
              <div>
                <Label>Receiver</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {bond.receiver_display}
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Raw Data Section -->
        <details class="group border-t border-zinc-800">
          <summary class="flex cursor-pointer items-center justify-between px-6 py-4 text-sm text-zinc-400 hover:text-zinc-200">
            <span class="font-medium">Raw Bond Data</span>
            <span class="transition-transform group-open:rotate-180">↓</span>
          </summary>
          <div class="px-6 pb-6">
            <JsonPreview value={bond} />
          </div>
        </details>
    {:catch error}
      <div class="p-6">
        <ErrorComponent {error} />
      </div>
    {/await}
  </Card>
</Sections>
