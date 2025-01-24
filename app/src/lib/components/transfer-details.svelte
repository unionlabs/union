<script lang="ts">
import { page } from "$app/stores"
import request from "graphql-request"
import { transfersBySourceHashBaseQueryDocument } from "$lib/graphql/queries/transfer-details.ts"
import DetailsHeading from "$lib/components/details-heading.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import * as Card from "$lib/components/ui/card/index.ts"
import { toIsoString } from "$lib/utilities/date"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { derived } from "svelte/store"
import { raise } from "$lib/utilities"
import Trace from "$lib/components/trace.svelte"
import type { Chain } from "$lib/types"
import { submittedTransfers } from "$lib/stores/submitted-transfers"
import { cn } from "$lib/utilities/shadcn"
import Truncate from "$lib/components/truncate.svelte"
import { formatUnits } from "viem"
import PacketPath from "./packet-path.svelte"
import Token from "./token.svelte"
import Address from "./address.svelte"

// prefix a source with 0x if not there for cosmos tx hashes
const source = $page.params.source.startsWith("0x")
  ? $page.params.source.toLowerCase()
  : `0x${$page.params.source.toLowerCase()}`

export let chains: Array<Chain>

let transfers = createQuery({
  queryKey: ["transfers-by-source-base", source],
  refetchInterval: query => (query.state.data?.length === 0 ? 1_000 : false), // fetch every second until we have the transaction
  placeholderData: (previousData, _) => previousData,
  queryFn: async () => {
    console.log("querying")
    const response = await request(URLS().GRAPHQL, transfersBySourceHashBaseQueryDocument, {
      source_transaction_hash: source
    })

    if (
      response.v1_ibc_union_fungible_asset_orders === undefined ||
      response.v1_ibc_union_fungible_asset_orders === null
    )
      raise("error fetching transfers")

    return response.v1_ibc_union_fungible_asset_orders
  }
})

let processedTransfers = derived(
  [transfers, submittedTransfers],
  ([$transfers, $submittedTransfers]) => {
    if ($transfers.data === undefined || $transfers.data.length === 0) {
      if ($submittedTransfers[source] === undefined) {
        return null
      }
      return [$submittedTransfers[source]]
    }
    return $transfers.data.map(transfer => {
      let tx = structuredClone(transfer)
      return {
        transfer_day: tx.packet_send_timestamp
          ? toIsoString(new Date(tx.packet_send_timestamp)).split("T")[0]
          : null,
        ...tx
      }
    })
  }
)
</script>

{#if $processedTransfers !== null && $processedTransfers.length > 0}
  <div class="flex flex-col w-full items-center gap-6">
    {#each $processedTransfers as transfer, transferIndex}
      {@const sourceExplorer = chains
        .find((c) => c.chain_id === transfer.source_chain_id)
        ?.explorers?.at(0)}
      {@const destinationExplorer = chains
        .find((c) => c.chain_id === transfer.destination_chain_id)
        ?.explorers?.at(0)}
      {#if transfer.source_chain_id !== null && transfer.destination_chain_id !== null}
        <Card.Root
          class="flex flex-col w-full lg:w-auto max-w-full overflow-y-hidden overflow-x-auto justify-self-center dark:bg-muted"
        >
          <Card.Header
            class="font-bold text-md text-center break-words text-muted-foreground flex flex-row gap-2 justify-center"
          >
            TRANSFER <Truncate
              value={transfer.packet_send_transaction_hash}
              type="hash"
            />
          </Card.Header>
          <Card.Content class="flex flex-col gap-8">
            <section class="flex justify-between">
              <div>
                <div class="flex flex-col gap-6">
                  {#if transfer.base_amount && transfer.base_token}
                    <Token
                      expanded="true"
                      amount={transfer.base_amount}
                      denom={transfer.base_token}
                      chainId={transfer.source_chain_id}
                      {chains}
                    />
                  {/if}
                  {#if "quote_amount" in transfer && transfer.quote_amount && "quote_token" in transfer && transfer.quote_token}
                    <Token
                      expanded="true"
                      amount={transfer.quote_amount}
                      denom={transfer.quote_token}
                      chainId={transfer.destination_chain_id}
                      {chains}
                    />
                  {/if}
                </div>
              </div>
            </section>

            <section>
              <!-- typescript is stupid here, as source_chain_id and destination_chain_id have already established not to be null. !-->
              <PacketPath packet={transfer} {chains} />
            </section>
            <section class="flex flex-col lg:flex-row justify-between gap-8">
              <div class="flex-col text-muted-foreground">
                <DetailsHeading>Sender</DetailsHeading>
                <Address showRaw address={transfer.sender_normalized} {chains} chainId={transfer.source_chain_id}  />
              </div>
              <div class="lg:text-right flex-col text-muted-foreground">
                <DetailsHeading>Receiver</DetailsHeading>
                <Address showRaw address={transfer.receiver_normalized} {chains} chainId={transfer.destination_chain_id}  />
              </div>
            </section>
          </Card.Content>
          <Card.Footer class="items-start flex flex-col w-full gap-4">
            <div class="font-bold text-md">{transfer.transfer_day}</div>
            {#if transfer.traces}
              <Trace traces={transfer.traces} {chains} />
            {:else}
              <LoadingLogo />
            {/if}
          </Card.Footer>
        </Card.Root>
        <!--
      <div class="text-transparent hover:text-muted-foreground transition text-xs overflow-hidden">
        {#if !(source.slice(0, 2) === "0x")}0x{/if}{source.toLowerCase()}
      </div>
      !-->
      {/if}
    {/each}
  </div>
{:else if $transfers.isLoading}
  <LoadingLogo class="size-16" />
{:else if $transfers.isError}
  Error loading transfer data
{/if}
