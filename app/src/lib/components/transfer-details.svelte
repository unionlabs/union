<script lang="ts">
import { page } from "$app/stores"
import request from "graphql-request"
import {
  transfersBySourceHashBaseQueryDocument
  // transfersBySourceHashTracesAndHopsQueryDocument
} from "$lib/graphql/queries/transfer-details.ts"
import DetailsHeading from "$lib/components/details-heading.svelte"
import MoveRightIcon from "virtual:icons/lucide/move-right"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import * as Card from "$lib/components/ui/card/index.ts"
import { toIsoString } from "$lib/utilities/date"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { derived, get, readable, type Readable } from "svelte/store"
import { toDisplayName } from "$lib/utilities/chains.ts"
import { raise } from "$lib/utilities"
import type { Step } from "$lib/stepper-types.ts"
import Stepper from "$lib/components/stepper.svelte"
import { zip } from "$lib/utilities/helpers.ts"
import type { Chain } from "$lib/types"
import { submittedTransfers } from "$lib/stores/submitted-transfers"
import { cn } from "$lib/utilities/shadcn"
import Truncate from "$lib/components/truncate.svelte"
import { formatUnits } from "viem"
import PacketPath from "./packet-path.svelte"
import type { UnwrapReadable } from "$lib/utilities/types"

const source = $page.params.source
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

// @ts-expect-error
let tracesSteps: Readable<Array<Array<Step>> | null> = derived(
  [processedTransfers],
  ([$processedTransfers]) => {
    if (!$processedTransfers) return null

    return $processedTransfers.map(transfer => {
      let traces: Exclude<UnwrapReadable<typeof transfers>["data"], undefined>[number]["traces"] =
        []

      if (!("_is_submitted_transfer" in transfer) && Array.isArray(transfer?.traces)) {
        traces = transfer.traces
      }

      const onSourceTrace = (eventType: string) =>
        traces.find(t => t.type === eventType && t.chain?.chain_id === transfer.source_chain_id)
      const onSource = (eventType: string) => onSourceTrace(eventType) !== undefined
      const onHopTrace = (eventType: string) =>
        traces.find(t => t.type === eventType && t.chain?.chain_id === transfer.hop_chain_id)
      const onHop = (eventType: string) => onHopTrace(eventType) !== undefined
      const onDestinationTrace = (eventType: string) =>
        traces.find(
          t => t.type === eventType && t.chain?.chain_id === transfer.destination_chain_id
        )
      const onDestination = (eventType: string) => onDestinationTrace(eventType) !== undefined

      const sourceChainExplorer = chains
        .find(c => c.chain_id === transfer.source_chain_id)
        ?.explorers?.at(0)
      const hopChainExplorer = chains
        .find(c => c.chain_id === transfer.hop_chain_id)
        ?.explorers?.at(0)
      const destinationChainExplorer = chains
        .find(c => c.chain_id === transfer.destination_chain_id)
        ?.explorers?.at(0)

      const sourceChainName = toDisplayName(transfer.source_chain_id, chains)
      //const hopChainName = toDisplayName(transfer.hop_chain_id, chains)
      const destinationChainName = toDisplayName(transfer.destination_chain_id, chains)

      const traceDetails = (eventType: string, c: "source" | "hop" | "destination") => {
        let trace =
          c === "source"
            ? onSourceTrace(eventType)
            : c === "hop"
              ? onHopTrace(eventType)
              : c === "destination"
                ? onDestinationTrace(eventType)
                : undefined
        let explorer =
          c === "source"
            ? sourceChainExplorer
            : c === "hop"
              ? hopChainExplorer
              : c === "destination"
                ? destinationChainExplorer
                : undefined
        let chain_display_name =
          c === "source"
            ? sourceChainName
            : c === "hop"
              ? hopChainName
              : c === "destination"
                ? destinationChainName
                : undefined

        if (trace === undefined) return undefined

        return explorer === undefined
          ? {
              chain_display_name,
              tx: trace.transaction_hash,
              block: trace.height,
              timestamp: trace.timestamp
            }
          : {
              chain_display_name,
              tx: trace.transaction_hash,
              tx_url: `${explorer.tx_url}${trace.transaction_hash}`,
              block: trace.height,
              block_url: `${explorer.block_url}${trace.height}`,
              timestamp: trace.timestamp
            }
      }

      return [
        {
          status: onSource("PACKET_SEND") ? "COMPLETED" : "IN_PROGRESS",
          title: `Send Packet`,
          description: "Waiting on indexer",
          traceDetails: traceDetails("PACKET_SEND", "source")
        },
        //(() => {
        //  let status = onDestination("LIGHTCLIENT_UPDATE")
        //    ? "COMPLETED"
        //    : onSource("PACKET_SEND")
        //      ? "IN_PROGRESS"
        //      : "PENDING"
        //  return {
        //    status,
        //    title: `Light Client Update`,
        //    description: status === "IN_PROGRESS" ? `Waiting on ${sourceChainName} finality` : "",
        //    traceDetails: traceDetails("LIGHTCLIENT_UPDATE", "destination")
        //  }
        //})(),
        (() => {
          let status = onDestination("PACKET_RECV")
            ? "COMPLETED"
            : onSource("PACKET_SEND")
              ? "IN_PROGRESS"
              : "PENDING"
          return {
            status,
            title: `Receive Packet`,
            traceDetails: traceDetails("PACKET_RECV", "destination")
          }
        })(),
        (() => {
          let status = onDestination("WRITE_ACK")
            ? "COMPLETED"
            : onDestination("PACKET_RECV")
              ? "IN_PROGRESS"
              : "PENDING"
          return {
            status,
            title: `Write Acknowledgement`,
            traceDetails: traceDetails("WRITE_ACK", "destination")
          }
        })(),
        (() => {
          let status = onSource("PACKET_ACK")
            ? "COMPLETED"
            : onDestination("PACKET_RECV")
              ? "IN_PROGRESS"
              : "PENDING"
          return {
            status,
            title: `Receive Acknowledgement`,
            traceDetails: traceDetails("PACKET_ACK", "source")
          }
        })()
      ]
    })
  }
)
</script>

<!--
<h1 class="font-bold text-md">Transfer for <span class="font-mono">{source}</span></h1>
<a href="/explorer/transfers">Back to all transfers </a>
!-->
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

      <!--
    <pre>{JSON.stringify($transfers.data, null, 2)}</pre>
    !-->

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
            <h2 class="font-supermolot uppercase md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap">
              {#if transfer.base_amount}
              {@const base_amount = BigInt(transfer.base_amount)}
                <Truncate
                  value={("base_token_details" in transfer && transfer.base_token_details?.decimals) ? formatUnits(base_amount, transfer.base_token_details.decimals) : base_amount}
                  type="full"
                />
              {/if}
              <Truncate
                value={("base_token_details" in transfer && transfer.base_token_details?.display_symbol) ? transfer.base_token_details.display_symbol : transfer.base_token}
                type="address"
              />
            </h2>
            <p class="text-muted-foreground text-sm break-words">
            <Truncate
              value={transfer.base_token}
              type="address"
            /> | {#if "base_token_details" in transfer && transfer.base_token_details}{transfer.base_token_details?.origin}{:else}NO DETAILS{/if}
            </p>

            </div>

            
            <div class="flex flex-col items-end">
            {#if "quote_token" in transfer}
            <h2 class="font-supermolot uppercase md:font-expanded text-2xl font-extrabold text-foreground whitespace-nowrap">
              {#if "quote_amount" in transfer && transfer.quote_amount}
              {@const quote_amount = BigInt(transfer.quote_amount)}
                <Truncate
                  value={("quote_token_details" in transfer && transfer.quote_token_details?.decimals) ? formatUnits(quote_amount, transfer.quote_token_details.decimals) : quote_amount}
                  type="full"
                />
              {/if}
            <Truncate
                value={(transfer.quote_token_details && transfer.quote_token_details.display_symbol) ? transfer.quote_token_details.display_symbol : transfer.quote_token}
              type="address"
            />
            </h2>
            <p class="text-muted-foreground text-sm break-words">
            <Truncate
              value={transfer.quote_token}
              type="address"
            /> | {#if  "quote_token_details" in transfer && "quote_token_details" in transfer && transfer.quote_token_details}{transfer.quote_token_details?.origin}{:else}NO DETAILS{/if}

            </p>
            {/if}
            </div>
            
          </section>

          <section>
            <!-- typescript is stupid here, as source_chain_id and destination_chain_id have already established not to be null. !-->
            <PacketPath packet={transfer} {chains}/>
          </section>
          <section class="flex flex-col lg:flex-row gap-8">
            <div class="flex-col text-muted-foreground">
              <DetailsHeading>
                Sender
              </DetailsHeading>
              {#if sourceExplorer !== undefined}
                <a
                  href={`/explorer/address/${transfer.sender}`}
                  class="block text-sm underline break-words"
                  ><Truncate class="underline" value={transfer.sender} type="address"/>
                </a>{:else}<p class="text-sm break-words">
                  <Truncate value={transfer.sender} type="address"/>
                </p>{/if}
              <p
                class={cn(
                  "text-[10px] break-words",
                  "normalized_sender" in transfer && transfer.normalized_sender
                    ? "text-black dark:text-muted-foreground"
                    : "text-transparent"
                )}
              >
              </p>
            </div>
            <div class="flex-1 lg:text-right flex-col text-muted-foreground">
              <DetailsHeading>
                Receiver
              </DetailsHeading>
              {#if destinationExplorer !== undefined}
                <a
                  href={`/explorer/address/${transfer.receiver}`}
                  class="block text-sm underline break-words"
                  ><Truncate class="underline" value={transfer.receiver} type="address"/>
                </a>{:else}<p class="text-sm break-words">
                  <Truncate value={transfer.receiver} type="address"/>
                </p>{/if}
              <p
                class={cn(
                  "text-[10px] break-words",
                  "normalized_receiver" in transfer && transfer.normalized_receiver
                    ? "text-black dark:text-muted-foreground"
                    : "text-transparent"
                )}
              >
              </p>
            </div>
          </section>
        </Card.Content>
        <Card.Footer class="items-start flex flex-col w-full gap-4">
          <div class="mt-6 font-bold text-md">{transfer.transfer_day}</div>
          <!-- bit of a hack, pTrace is used to check if there is a trace, and if there is, we show the steps !-->
          {@const pTrace = $processedTransfers?.at(transferIndex) ?? null}
          {@const ts = derived(
            tracesSteps,
            ($tracesSteps) => $tracesSteps?.at(transferIndex) ?? []
          )}
          {#if pTrace}
            <Stepper steps={ts} />
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

<pre>
{JSON.stringify($transfers, null, 2)}
</pre>


