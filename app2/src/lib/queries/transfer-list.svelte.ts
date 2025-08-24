import { transferListItemFragment } from "$lib/queries/fragments/transfer-list-item"
import { transferList } from "$lib/stores/transfers.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import type { SortOrder } from "@unionlabs/sdk/schema"
import { TransferList } from "@unionlabs/sdk/schema"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"

const LIMIT = 10

export let transferListLatestQuery = (limit = LIMIT, mainnetOnly = false) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListLatest($limit: Int!, $network: String) @cached(ttl: 1) {
      v2_transfers(args: {
        p_limit: $limit,
        p_network: $network
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment],
    ),
    variables: {
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "1 second",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers))
    },
    writeError: error => {
      transferList.error = error
    },
  })

export let transferListPageLtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT,
  mainnetOnly = false,
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!, $network: String)
    @cached(ttl: 30) {
      v2_transfers(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_network: $network
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment],
    ),
    variables: {
      page,
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers))
    },
    writeError: error => {
      transferList.error = error
    },
  })

export let transferListPageGtQuery = (
  page: typeof SortOrder.Type,
  limit = LIMIT,
  mainnetOnly = false,
) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_transfers: TransferList }),
    document: graphql(
      `
    query TransferListPage($page: String!, $limit: Int!, $network: String) @cached(ttl: 30) {
      v2_transfers(args: {
        p_limit: $limit,
        p_sort_order: $page,
        p_comparison: "gt",
        p_network: $network
      }) {
      ...TransferListItem
      }
    }
  `,
      [transferListItemFragment],
    ),
    variables: {
      page,
      limit,
      network: mainnetOnly ? "mainnet" : null,
    },
    refetchInterval: "30 seconds",
    writeData: data => {
      transferList.data = data.pipe(Option.map(d => d.v2_transfers.toReversed()))
    },
    writeError: error => {
      transferList.error = error
    },
  })
