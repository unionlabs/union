import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList, transferListMissingAck } from "$lib/stores/transfers.svelte"
import { transferListItemFragment, transferListItemFragmentAckMissing } from "$lib/queries/fragments/transfer-list-item"
import type { SortOrder } from "@unionlabs/sdk/schema"
import { TransferList, TransferListMissingAck } from "@unionlabs/sdk/schema"

export const LIMIT = 10

export let transferListLatestQuery = (limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListLatest($limit: Int!) @cached(ttl: 1) {
      v2_transfers(args: {
        p_limit: $limit
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { limit },
    refetchInterval: "1 second",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers))
    },
    writeError: error => {
      transferList.error = error
    }
  })

  export let transferListPendingAckQuery = (limit = LIMIT) =>
    createQueryGraphql({
      schema: Schema.Struct({ v2_transfers: TransferListMissingAck }),
      document: graphql(
        `
        query TransferListPendingAck($limit: Int!) @cached(ttl: 1) {
          v2_transfers(args: { p_limit: $limit }) {
            ...TransferListItemMissingAck
          }
        }
        `,
        [transferListItemFragmentAckMissing]
      ),
      variables: { limit },
      // run every second so the “pending” list auto‑refreshes
      refetchInterval: "1 second",
      writeData: (data) => {
        console.info("transferListPendingAckQuery", data)
        // data: Option<{ v2_transfers: TransferListItem[] }>
        transferList.data = data.pipe(
          Option.map(resp =>
            resp.v2_transfers.filter(tx =>
              // keep only those with a PACKET_ACK trace but no tx hash yet
              tx.traces.some(t => t.type === "PACKET_ACK" &&  Option.isNone(t.transaction_hash)
            )
            )
          )
        )
        //0xd61ee964e748d5ba1536924db05107d23efa02e9f719ea1c1361025667294ed7
      },
  
      writeError: (err) => {
        transferList.error = err
      }
    })

export let transferListPendingAckPageLtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferListMissingAck }),
    document: graphql(`
      query TransferListPendingAckPage($page: String!, $limit: Int!) {
        v2_transfers(args: {
          p_limit: $limit,
          p_sort_order: $page
        }) {
          ...TransferListItemMissingAck
        }
      }
    `, [transferListItemFragmentAckMissing]),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(
        Option.map(resp =>
          resp.v2_transfers.filter(tx =>
            tx.traces.some(t => t.type === "PACKET_ACK" && Option.isNone(t.transaction_hash))
          )
        )
      )
    },
    writeError: err => {
      transferList.error = err
    }
  })

// 2) Greater‑than page (prev page):
export let transferListPendingAckPageGtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferListMissingAck }),
    document: graphql(`
      query TransferListPendingAckPageGt($page: String!, $limit: Int!) {
        v2_transfers(args: {
          p_limit: $limit,
          p_sort_order: $page,
          p_comparison: "gt"
        }) {
          ...TransferListItemMissingAck
        }
      }
    `, [transferListItemFragmentAckMissing]),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(
        Option.map(resp =>
          // reverse so “prev” page still orders newest‑first
          resp.v2_transfers
            .toReversed()
            .filter(tx =>
              tx.traces.some(
                t => t.type === "PACKET_ACK" && Option.isNone(t.transaction_hash)
              )
            )
        )
      )
    },
    writeError: err => {
      transferList.error = err
    }
  })

    
export let transferListPageLtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!)
    @cached(ttl: 30) {
      v2_transfers(args: {
        p_limit: $limit,
        p_sort_order: $page
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers))
    },
    writeError: error => {
      transferList.error = error
    }
  })

export let transferListPageGtQuery = (page: typeof SortOrder.Type, limit = LIMIT) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!) @cached(ttl: 30) {
      v2_transfers(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "gt"
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment]
    ),
    variables: { page, limit },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers.toReversed()))
    },
    writeError: error => {
      transferList.error = error
    }
  })
