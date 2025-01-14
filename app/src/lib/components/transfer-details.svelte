<script lang="ts">
import { page } from "$app/stores"
import request from "graphql-request"
import {
  transfersBySourceHashBaseQueryDocument
  // transfersBySourceHashTracesAndHopsQueryDocument
} from "$lib/graphql/queries/transfer-details.ts"
import DetailsHeading from "$lib/components/details-heading.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { URLS } from "$lib/constants"
import * as Card from "$lib/components/ui/card/index.ts"
import { toIsoString } from "$lib/utilities/date"
import LoadingLogo from "$lib/components/loading-logo.svelte"
import { derived, readable, type Readable } from "svelte/store"
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

//@ts-ignore
let processedTransfers = derived(
  [transfers, submittedTransfers],
  ([$transfers, $submittedTransfers]) => {
    if ($transfers.data === undefined || $transfers.data.length === 0) {
      // @ts-expect-error
      if ($submittedTransfers[source] === undefined) {
        return null
      }
      // @ts-expect-error
      return [$submittedTransfers[source]]
    }
    //@ts-ignore
    return $transfers.data.map(transfer => {
      let tx = structuredClone(transfer)

      let hop_chain_id: string | null = null
      let hop_chain_destination_connection_id: string | null = null
      let hop_chain_destination_channel_id: string | null = null
      let hop_chain_source_connection_id: string | null = null
      let hop_chain_source_channel_id: string | null = null

      // overwrite destination and receiver if to last forward

      // forwards does not contain sequence numbers,
      // so we cannot construct the destination sequence at this stage yet.
      // const lastForward = tx.forwards?.at(-1)
      // if (lastForward) {
      //   hop_chain_id = tx.destination_chain_id
      //   hop_chain_destination_connection_id = tx.destination_connection_id
      //   hop_chain_destination_channel_id = tx.destination_channel_id
      //   hop_chain_source_connection_id = lastForward.source_connection_id
      //   hop_chain_source_channel_id = lastForward.source_channel_id
      //   tx.destination_chain_id = lastForward.destination_chain_id
      //   tx.destination_connection_id = lastForward.destination_connection_id
      //   tx.destination_channel_id = lastForward.destination_channel_id
      //   tx.receiver = lastForward.receiver
      //   tx.normalized_receiver = lastForward.receiver
      // }

      // if (tx.hop !== null) {
      //   // hop_chain = tx.destination_chain
      //   // hop_chain_id = tx.destination_chain_id

      //   // tx.destination_chain = tx.hop.destination_chain
      //   // tx.destination_chain_id = tx.hop.destination_chain_id
      //   // tx.destination_connection_id = tx.hop.destination_connection_id
      //   // tx.destination_channel_id = tx.hop.destination_channel_id
      //   // tx.receiver = tx.hop.receiver
      //   tx.normalized_receiver = tx.hop.normalized_receiver
      //   tx.traces.push.apply(tx.traces, tx.hop.traces)
      //   tx.traces.sort((a, b) => {
      //     // @ts-ignore timestamp is guaranteed to be a date
      //     return new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
      //   })
      // }

      return {
        hop_chain_id,
        hop_chain_destination_connection_id,
        hop_chain_destination_channel_id,
        hop_chain_source_connection_id,
        hop_chain_source_channel_id,
        // @ts-ignore
        transfer_day: toIsoString(new Date(tx.source_timestamp)).split("T")[0],
        ...tx
      }
    })
  }
)

// let tracesAndHops = createQuery({
//   queryKey: ["transfers-by-source-traces-and-hops", source],
//   refetchInterval: 1_000,
//   placeholderData: (previousData, _) => previousData,
//   queryFn: async () =>
//     (
//       await request(URLS().GRAPHQL, transfersBySourceHashTracesAndHopsQueryDocument, {
//         source_transaction_hash: source
//       })
//     ).v1_transfers
// })

// let processedTraces = derived(
//   [tracesAndHops, submittedTransfers],
//   ([$tracesAndHops, $submittedTransfers]) => {
//     if (!$tracesAndHops.data || $tracesAndHops.data.length === 0) {
//       // @ts-expect-error
//       if ($submittedTransfers[source] !== undefined) {
//         return [[]] // pre-generate trace for submitted transfer
//       }
//       return null
//     }
//
//     return $tracesAndHops.data.map(tx => {
//       if (tx.hop !== null) {
//         tx.traces.push.apply(tx.traces, tx.hop.traces)
//         tx.traces.sort((a, b) => {
//           return (
//             // @ts-ignore timestamp is guaranteed to be a date
//             new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
//           )
//         })
//       }
//
//       return tx.traces
//     })
//   }
// )

// // @ts-expect-error
// let tracesSteps: Readable<Array<Array<Step>> | null> = derived(
//   [processedTraces, processedTransfers],
//   ([$processedTraces, $processedTransfers]) => {
//     if (!($processedTraces && $processedTransfers)) return null
//
//     return zip($processedTransfers, $processedTraces).map(([transfer, traces]) => {
//       const onSourceTrace = (eventType: string) =>
//         traces.find(t => t.type === eventType && t.chain?.chain_id === transfer.source_chain_id)
//       const onSource = (eventType: string) => onSourceTrace(eventType) !== undefined
//       const onHopTrace = (eventType: string) =>
//         traces.find(t => t.type === eventType && t.chain?.chain_id === transfer.hop_chain_id)
//       const onHop = (eventType: string) => onHopTrace(eventType) !== undefined
//       const onDestinationTrace = (eventType: string) =>
//         traces.find(
//           t => t.type === eventType && t.chain?.chain_id === transfer.destination_chain_id
//         )
//       const onDestination = (eventType: string) => onDestinationTrace(eventType) !== undefined
//
//       const sourceChainExplorer = chains
//         .find(c => c.chain_id === transfer.source_chain_id)
//         ?.explorers?.at(0)
//       const hopChainExplorer = chains
//         .find(c => c.chain_id === transfer.hop_chain_id)
//         ?.explorers?.at(0)
//       const destinationChainExplorer = chains
//         .find(c => c.chain_id === transfer.destination_chain_id)
//         ?.explorers?.at(0)
//
//       const sourceChainName = toDisplayName(transfer.source_chain_id, chains)
//       const hopChainName = toDisplayName(transfer.hop_chain_id, chains)
//       const destinationChainName = toDisplayName(transfer.destination_chain_id, chains)
//
//       const traceDetails = (eventType: string, c: "source" | "hop" | "destination") => {
//         let trace =
//           c === "source"
//             ? onSourceTrace(eventType)
//             : c === "hop"
//               ? onHopTrace(eventType)
//               : c === "destination"
//                 ? onDestinationTrace(eventType)
//                 : undefined
//         let explorer =
//           c === "source"
//             ? sourceChainExplorer
//             : c === "hop"
//               ? hopChainExplorer
//               : c === "destination"
//                 ? destinationChainExplorer
//                 : undefined
//         let chain_display_name =
//           c === "source"
//             ? sourceChainName
//             : c === "hop"
//               ? hopChainName
//               : c === "destination"
//                 ? destinationChainName
//                 : undefined
//
//         if (trace === undefined) return undefined
//
//         return explorer === undefined
//           ? {
//               chain_display_name,
//               tx: trace.transaction_hash,
//               block: trace.height,
//               timestamp: trace.timestamp
//             }
//           : {
//               chain_display_name,
//               tx: trace.transaction_hash,
//               tx_url: `${explorer.tx_url}${trace.transaction_hash}`,
//               block: trace.height,
//               block_url: `${explorer.block_url}${trace.height}`,
//               timestamp: trace.timestamp
//             }
//       }
//
//       if (!transfer.hop_chain_id) {
//         return [
//           {
//             status: onSource("SEND_PACKET") ? "COMPLETED" : "IN_PROGRESS",
//             title: `Send Packet`,
//             description: "Waiting on indexer",
//             traceDetails: traceDetails("SEND_PACKET", "source")
//           },
//           (() => {
//             let status = onDestination("LIGHTCLIENT_UPDATE")
//               ? "COMPLETED"
//               : onSource("SEND_PACKET")
//                 ? "IN_PROGRESS"
//                 : "PENDING"
//             return {
//               status,
//               title: `Light Client Update`,
//               description: status === "IN_PROGRESS" ? `Waiting on ${sourceChainName} finality` : "",
//               traceDetails: traceDetails("LIGHTCLIENT_UPDATE", "destination")
//             }
//           })(),
//           (() => {
//             let status = onDestination("RECEIVE_PACKET")
//               ? "COMPLETED"
//               : onDestination("LIGHTCLIENT_UPDATE")
//                 ? "IN_PROGRESS"
//                 : "PENDING"
//             return {
//               status,
//               title: `Receive Packet`,
//               traceDetails: traceDetails("RECEIVE_PACKET", "destination")
//             }
//           })(),
//           (() => {
//             let status = onSource("ACKNOWLEDGE_PACKET")
//               ? "COMPLETED"
//               : onDestination("RECEIVE_PACKET")
//                 ? "IN_PROGRESS"
//                 : "PENDING"
//             return {
//               status,
//               title: `Acknowledge Packet`,
//               traceDetails: traceDetails("ACKNOWLEDGE_PACKET", "source")
//             }
//           })()
//         ]
//       }
//
//       return [
//         {
//           status: onSource("SEND_PACKET") ? "COMPLETED" : "IN_PROGRESS",
//           title: `Send Packet`,
//           description: "Waiting on indexer",
//           traceDetails: traceDetails("SEND_PACKET", "source")
//         },
//         (() => {
//           let status = onHop("LIGHTCLIENT_UPDATE")
//             ? "COMPLETED"
//             : onSource("SEND_PACKET")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Light Client Update`,
//             description: status === "IN_PROGRESS" ? `Waiting on ${sourceChainName} finality` : "",
//             traceDetails: traceDetails("LIGHTCLIENT_UPDATE", "hop")
//           }
//         })(),
//         (() => {
//           let status = onHop("RECEIVE_PACKET")
//             ? "COMPLETED"
//             : onHop("LIGHTCLIENT_UPDATE")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Receive Packet`,
//             traceDetails: traceDetails("RECEIVE_PACKET", "hop")
//           }
//         })(),
//         (() => {
//           let status = onHop("SEND_PACKET")
//             ? "COMPLETED"
//             : onHop("RECEIVE_PACKET")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Send Packet`,
//             traceDetails: traceDetails("SEND_PACKET", "hop")
//           }
//         })(),
//         (() => {
//           let status = onDestination("LIGHTCLIENT_UPDATE")
//             ? "COMPLETED"
//             : onHop("SEND_PACKET")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Light Client Update`,
//             description: status === "IN_PROGRESS" ? `Waiting on ${hopChainName} finality` : "",
//             traceDetails: traceDetails("LIGHTCLIENT_UPDATE", "destination")
//           }
//         })(),
//         (() => {
//           let status = onDestination("RECEIVE_PACKET")
//             ? "COMPLETED"
//             : onDestination("LIGHTCLIENT_UPDATE")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Receive Packet`,
//             traceDetails: traceDetails("RECEIVE_PACKET", "destination")
//           }
//         })(),
//         (() => {
//           let status = onHop("ACKNOWLEDGE_PACKET")
//             ? "COMPLETED"
//             : onDestination("RECEIVE_PACKET")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Acknowledge Packet`,
//             traceDetails: traceDetails("ACKNOWLEDGE_PACKET", "hop")
//           }
//         })(),
//         (() => {
//           let status = onSource("ACKNOWLEDGE_PACKET")
//             ? "COMPLETED"
//             : onHop("ACKNOWLEDGE_PACKET")
//               ? "IN_PROGRESS"
//               : "PENDING"
//           return {
//             status,
//             title: `Acknowledge Packet`,
//             traceDetails: traceDetails("ACKNOWLEDGE_PACKET", "source")
//           }
//         })()
//       ]
//     })
//   }
// )
</script>

<!--
<h1 class="font-bold text-md">Transfer for <span class="font-mono">{source}</span></h1>
<a href="/explorer/transfers">Back to all transfers </a>
!-->

<pre>
{JSON.stringify($transfers.data, null, 2)}
</pre>

