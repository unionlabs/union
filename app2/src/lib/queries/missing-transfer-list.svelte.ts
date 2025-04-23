// src/lib/queries/transfer-list-expired-window.ts
import { Effect, Option } from "effect"
import { fetchDecodeGraphql } from "$lib/utils/queries"
import { graphql } from "gql.tada"
import { Schema } from "effect"
import { transferListItemFragmentAckMissing } from "$lib/queries/fragments/transfer-list-item"
import { TransferListMissingAck } from "@unionlabs/sdk/schema"
import { incompleteTransferList } from "$lib/stores/incomplete-transfers.svelte"

export const LIMIT = 10

// SLA windows in ms for each source|dest pair
const TIMEFRAMES: Record<string, number> = {
  "babylon.bbn-1|bob.60808":     54_000_000,
  "bob.60808|babylon.bbn-1":     54_000_000,
  "corn.21000000|babylon.bbn-1": 61_200_000,
  "babylon.bbn-1|corn.21000000": 61_200_000,
  "ethereum.1|babylon.bbn-1":    2_700_000,
  "babylon.bbn-1|ethereum.1":    2_700_000,
}

// one doc that brings in the traces
const missingAckDoc = graphql(`
  query TransferListMissingAck($limit: Int!, $sortOrder: String) {
    v2_transfers(
      args: { p_limit: $limit, p_sort_order: $sortOrder, p_comparison: "lt" }
    ) {
      ...TransferListItemMissingAck
    }
  }
`, [transferListItemFragmentAckMissing])

/**
 * Fetch *only* those transfers whose send-timestamp falls in the window
 *   [ now - 2*SLA,  now - SLA )
 * and which still have an ACK‐trace with `transaction_hash == null`.
 */
// export function transferListInWindow(
//   source: string,
//   destination: string,
//   limit = LIMIT
// ) {
//     limit = 100 
//   const key        = `${source}|${destination}`
//   const sla        = TIMEFRAMES[key] ?? limit * 1000
//   const windowEnd  = Date.now() - sla
//   const windowStart= windowEnd - sla

//   return Effect.gen(function* () {
//     let cursor: string | undefined
//     let found: typeof TransferListMissingAck.Type = []

//     while (true) {
//       // page one batch (descending by sort_order)
//       const page = yield* fetchDecodeGraphql(
//         Schema.Struct({ v2_transfers: TransferListMissingAck }),
//         missingAckDoc,
//         { limit, sortOrder: cursor }
//       )
//       const txs = page.v2_transfers
//       if (txs.length === 0) break

//       // collect those in our [start, end) time window
//       const inWindow = txs.filter(tx => {
//         const sent = tx.transfer_send_timestamp.epochMillis
//         const hasNullAck = tx.traces.some(
//             ({ type, transaction_hash }) =>
//               type === "PACKET_ACK" && !transaction_hash?.value
//           )
          
//         return (
//           tx.source_chain.universal_chain_id     === source  &&
//           tx.destination_chain.universal_chain_id=== destination &&
//           sent >= windowStart &&
//           sent <  windowEnd &&
//           hasNullAck
//         )
//       })
//       found.push(...inWindow)

//       // if the **oldest** in this page is already before windowStart, we can stop
//       const lastSent = txs[txs.length - 1].transfer_send_timestamp.epochMillis
//       if (txs.length < limit || lastSent < windowStart) break

//       // otherwise keep paging
//       cursor = txs[txs.length - 1].sort_order
//     }

//     return found
//   })
// }

// fetch only the packet fields we need

// GraphQL query page for v2_packets with filtering on source, destination & cursor
const packetPageDoc = graphql(`
    query PacketPage($src: String!, $dst: String!, $limit: Int!, $sortOrder: String) {
      v2_packets(
        args: {
          p_source_universal_chain_id: $src,
          p_destination_universal_chain_id: $dst,
          p_limit: $limit,
          p_sort_order: $sortOrder,
          p_comparison: "lt"
        }
      ) {
        packet_hash
        packet_send_timestamp
        write_ack_timestamp
        packet_ack_timestamp
        sort_order
      }
    }
  `, [] as const)
  
  // Schema to decode one page of packets (timestamps are ISO strings)
  const PacketPageSchema = Schema.Struct({
    v2_packets: Schema.Array(
      Schema.Struct({
        packet_hash:           Schema.String,
        packet_send_timestamp: Schema.String,
        write_ack_timestamp:   Schema.Optional(Schema.String),
        packet_ack_timestamp:  Schema.Optional(Schema.String),
        sort_order:            Schema.String
      })
    )
  })
  
  /**
   * Page through v2_packets from newest to older until we hit rows older than `sinceIso`.
   * Returns all packets with send_timestamp >= sinceIso.
   */
  function fetchPacketsUntil(sinceIso: string, source: string, dest: string, limit = LIMIT) {
    return Effect.gen(function* () {
      let cursor: string | undefined
      const collected: Array<{
        packet_hash: string,
        packet_send_timestamp: string,
        write_ack_timestamp: string | null,
        packet_ack_timestamp: string | null,
        sort_order: string
      }> = []
  
      while (true) {
        const page = yield* fetchDecodeGraphql(
          PacketPageSchema,
          packetPageDoc,
          { src: source, dst: dest, limit, sortOrder: cursor }
        )
  
        const rows = page.v2_packets
        if (rows.length === 0) break
  
        // stop once we hit older than our cutoff
        let stop = false
        for (const r of rows) {
          const sentMs = new Date(r.packet_send_timestamp).getTime()
          if (sentMs < new Date(sinceIso).getTime()) {
            stop = true
            break
          }
          collected.push(r)
        }
        if (stop || rows.length < limit) break
  
        cursor = rows[rows.length - 1]!.sort_order
      }
  
      return collected
    })
  }
  
  /**
   * For a given source/dest & SLA timeframe, return packets whose send_timestamp
   * lies in [now - 2*SLA, now - SLA) and which still lack an ACK.
   */
  export function expiredPacketsForPair(
    source: string,
    dest: string,
    limit = LIMIT
  ) {
    const key = `${source}|${dest}`
    const sla = TIMEFRAMES[key] ?? limit * 1000
    const now = Date.now()
    const windowEnd = now - sla
    const windowStart = windowEnd - sla
  
    // convert to ISO for our fetch cutoff
    const sinceIso = new Date(windowStart).toISOString()
  
    return fetchPacketsUntil(sinceIso, source, dest, limit).pipe(
      Effect.map(all =>
        all.filter(r => {
          const sent = new Date(r.packet_send_timestamp).getTime()
          const inWindow = sent >= windowStart && sent < windowEnd
          const ackMissing = r.packet_ack_timestamp === null
          return inWindow && ackMissing
        })
      )
    )
  }
    

/**
 * Kick it off and write into your store
 */
export function runInWindow(
  source: string,
  destination: string,
  limit = LIMIT
) {
  incompleteTransferList.data  = Option.none()
  incompleteTransferList.error = Option.none()

  return incompleteTransferList.runEffect(
    transferListInWindow(source, destination, limit).pipe(
      Effect.tapBoth({
        onSuccess: list =>
          Effect.sync(() => {
            incompleteTransferList.data = Option.some(list)
          }),
        onFailure: err =>
          Effect.sync(() => {
            incompleteTransferList.error = Option.some(err)
          })
      })
    )
  )
}
