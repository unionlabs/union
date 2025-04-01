import { createQueryGraphql } from "$lib/utils/queries"

export const LIMIT = 10
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferList } from "$lib/stores/transfers.svelte"
import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { TransferList } from "@unionlabs/sdk/schema"
import type { SortOrder } from "@unionlabs/sdk/schema"

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
