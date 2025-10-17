<script lang="ts">
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import JsonPreview from "$lib/components/ui/JsonPreview.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { Indexer } from "@unionlabs/sdk"
import { TokenRawAmount } from "@unionlabs/sdk/schema"
import { ConfigProvider, Effect, Layer, pipe } from "effect"
import { graphql } from "gql.tada"
import type { PageData } from "./$types"
import * as AppRuntime from "$lib/runtime"

interface Props {
  data: PageData
}

const { data }: Props = $props()

const packetHash = $derived(data.packetHash)

const QlpConfigProvider = Layer.setConfigProvider(
  ConfigProvider.fromMap(
    new Map([
      ["GRAPHQL_ENDPOINT", "https://graphql.union.build/v1/graphql"],
    ]),
  ),
)

const dustWithdrawalData = $derived(pipe(
  Effect.gen(function*() {
    const indexer = yield* Indexer.Indexer
    const result = yield* indexer.fetch({
      document: graphql(`
        query GetDustWithdrawalByPacketHash($packet_hash: String!) @cached(ttl: 10) {
          v2_dust_withdraws(args: { p_packet_hash: $packet_hash }) {
            packet_hash
            delivery_packet_hash
            dust_withdraw_success
            delivery_success
            packet_shape
            source_universal_chain_id
            destination_universal_chain_id
            staker_canonical
            staker_display
            staker_zkgm
            quote_token
            quote_amount
            dust_withdraw_send_timestamp
            dust_withdraw_send_transaction_hash
            dust_withdraw_recv_timestamp
            dust_withdraw_recv_transaction_hash
            dust_withdraw_timeout_timestamp
            dust_withdraw_timeout_transaction_hash
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
      variables: { packet_hash: packetHash },
    })

    const dustWithdrawals = result.v2_dust_withdraws as Array<any>

    if (dustWithdrawals.length === 0) {
      return yield* Effect.fail(new Error("Dust withdrawal not found"))
    }

    return dustWithdrawals[0]
  }),
  Effect.provide(Layer.fresh(Indexer.Indexer.Default)),
  Effect.provide(QlpConfigProvider),
  AppRuntime.runPromise,
))
</script>

<Sections>
  <Card divided>
    {#await dustWithdrawalData}
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
    {:then dw}
      {@const status = dw.dust_withdraw_success === true && dw.delivery_success === true
        ? "success"
        : dw.dust_withdraw_success === false || dw.delivery_success === false
        ? "failure"
        : dw.dust_withdraw_recv_timestamp && dw.delivery_recv_timestamp
        ? "success"
        : dw.dust_withdraw_timeout_timestamp || dw.delivery_timeout_timestamp
        ? "failure"
        : "pending"}

      <div class="p-6">
        <h1 class="text-2xl font-bold mb-4">Dust Withdrawal</h1>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Dust Withdrawal Details -->
          <div class="space-y-4">
            <div>
              <Label>Packet Hash</Label>
              <div class="font-mono text-sm text-zinc-400 break-all">
                {dw.packet_hash}
              </div>
            </div>

            <div>
              <Label>Amount</Label>
              {#if dw.source_chain}
                <TokenComponent
                  chain={dw.source_chain}
                  denom={dw.quote_token}
                  amount={BigInt(dw.quote_amount) as TokenRawAmount}
                  showIcon={true}
                />
              {:else}
                <div class="font-semibold font-mono">
                  {dw.quote_amount} {dw.quote_token}
                </div>
              {/if}
            </div>

            <div>
              <Label>Status</Label>
              <div class="text-sm">
                <span
                  class="
                    inline-flex items-center px-2 py-1 rounded text-xs font-medium {
                    status === 'success'
                    ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/20 dark:text-emerald-400'
                    : status === 'failure'
                    ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
                    : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
                    }
                  "
                >
                  {status}
                </span>
              </div>
            </div>

            <div>
              <Label>Dust Withdraw Send Timestamp</Label>
              {#if dw.dust_withdraw_send_timestamp}
                <div class="text-sm text-zinc-400">
                  {
                    (() => {
                      try {
                        const date = new Date(dw.dust_withdraw_send_timestamp)
                        return isNaN(date.getTime())
                          ? "Invalid date"
                          : date.toLocaleString()
                      } catch {
                        return "Invalid date"
                      }
                    })()
                  }
                </div>
              {:else}
                <div class="text-sm text-zinc-500">No timestamp</div>
              {/if}
            </div>

            {#if dw.dust_withdraw_recv_timestamp}
              <div>
                <Label>Dust Withdraw Receive Timestamp</Label>
                <div class="text-sm text-zinc-400">
                  {
                    (() => {
                      try {
                        const date = new Date(dw.dust_withdraw_recv_timestamp)
                        return isNaN(date.getTime())
                          ? "Invalid date"
                          : date.toLocaleString()
                      } catch {
                        return "Invalid date"
                      }
                    })()
                  }
                </div>
              </div>
            {/if}

            {#if dw.delivery_recv_timestamp}
              <div>
                <Label>Delivery Receive Timestamp</Label>
                <div class="text-sm text-zinc-400">
                  {
                    (() => {
                      try {
                        const date = new Date(dw.delivery_recv_timestamp)
                        return isNaN(date.getTime())
                          ? "Invalid date"
                          : date.toLocaleString()
                      } catch {
                        return "Invalid date"
                      }
                    })()
                  }
                </div>
              </div>
            {/if}
          </div>

          <!-- Chain Information -->
          <div class="space-y-4">
            <div>
              <Label>Source Chain</Label>
              {#if dw.source_chain}
                <ChainComponent
                  chain={dw.source_chain}
                  withToken={dw.quote_token}
                />
              {:else}
                <div class="text-sm text-zinc-500">{dw.source_universal_chain_id}</div>
              {/if}
            </div>

            <div>
              <Label>Destination Chain</Label>
              {#if dw.destination_chain}
                <ChainComponent
                  chain={dw.destination_chain}
                  withToken={dw.quote_token}
                />
              {:else}
                <div class="text-sm text-zinc-500">{dw.destination_universal_chain_id}</div>
              {/if}
            </div>

            {#if dw.delivery_packet_hash}
              <div>
                <Label>Delivery Packet Hash</Label>
                <div class="font-mono text-sm text-zinc-400 break-all">
                  {dw.delivery_packet_hash}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Raw Data Section -->
      <details class="group border-t border-zinc-800">
        <summary class="flex cursor-pointer items-center justify-between px-6 py-4 text-sm text-zinc-400 hover:text-zinc-200">
          <span class="font-medium">Raw Dust Withdrawal Data</span>
          <span class="transition-transform group-open:rotate-180">↓</span>
        </summary>
        <div class="px-6 pb-6">
          <JsonPreview value={dw} />
        </div>
      </details>
    {:catch error}
      <div class="p-6">
        <ErrorComponent {error} />
      </div>
    {/await}
  </Card>
</Sections>

