import { graphql } from "../index.ts"
import { transferListDataFragment } from "$lib/graphql/fragments/transfers.ts"

export const transfersTimestampQuery = graphql(
  /* GraphQL */ `
    query TransfersTimestampQuery($limit: Int!, $timestamp: timestamptz!)
    @cached(ttl: 1000) {
      newer: v1_ibc_union_fungible_asset_orders(
        limit: $limit
        order_by: [{ packet_send_timestamp: asc }, { packet_send_transaction_hash: asc }]
        where: { 
            packet_send_timestamp: { _gte: $timestamp }
            quote_amount: { _neq: "0x00" }
        }
      ) {
        ...TransferListData
      }
      older: v1_ibc_union_fungible_asset_orders(
        limit: $limit
        order_by: [
          { packet_send_timestamp: desc }
          { packet_send_transaction_hash: desc }
        ]
        where: { packet_send_timestamp: { _lt: $timestamp } }
      ) {
        ...TransferListData
      }
    }
  `,
  [transferListDataFragment]
)

export const transfersLatestQuery = graphql(
  /* GraphQL */ `
    query TransfersLatestQuery($limit: Int! = 8) {
      data: v1_ibc_union_fungible_asset_orders(
        limit: $limit
        order_by: [
            { packet_send_timestamp: desc }
            { packet_send_transaction_hash: desc }
        ]
          where: {
              quote_amount: { _neq: "0x00" }
          }
      ) {
        ...TransferListData
      }
    }
  `,
  [transferListDataFragment]
)

export const transfersIncompleteQuery = graphql(
  /* GraphQL */ `
    query TransfersLatestQuery($cutoff: timestamptz! = "2025-02-11") {
      data:   v1_ibc_union_fungible_asset_orders(where: {
          packet_send_timestamp: {_gt: $cutoff}, 
          status: {_neq: "PACKET_ACK"}
          quote_amount: { _neq: "0x00" }
      }
        order_by: [
          { packet_send_timestamp: desc }
          { packet_send_transaction_hash: desc }
        ]
      ) {
        ...TransferListData
      }
    }
  `,
  [transferListDataFragment]
)

// export const transfersByAddressesTimestampQuery = graphql(
//   /* graphql */ `
//     query TransfersByAddressesTimestampQuery(
//       $limit: Int!
//       $addresses: [String!]!
//       $timestamp: timestamptz!
//     ) @cached(ttl: 1000) {
//       newer: v1_ibc_union_fungible_asset_orders(
//         limit: $limit
//         order_by: [{ packet_send_timestamp: asc }, { packet_send_transaction_hash: asc }]
//         where: {
//           _and: [
//             { packet_send_timestamp: { _gte: $timestamp } }
//             {
//               _or: [
//                 { normalized_sender: { _in: $addresses } }
//                 { normalized_receiver: { _in: $addresses } }
//               ]
//             }
//           ]
//         }
//       ) {
//         ...TransferListData
//       }

//       older: v1_ibc_union_fungible_asset_orders(
//         limit: $limit
//         order_by: [
//           { packet_send_timestamp: desc }
//           { packet_send_transaction_hash: desc }
//         ]
//         where: {
//           _and: [
//             { packet_send_timestamp: { _lt: $timestamp } }
//             {
//               _or: [
//                 { normalized_sender: { _in: $addresses } }
//                 { normalized_receiver: { _in: $addresses } }
//               ]
//             }
//           ]
//         }
//       ) {
//         ...TransferListData
//       }
//     }
//   `,
//   [transferListDataFragment]
// )

// export const transfersByAddressesLatestQuery = graphql(
//   /* graphql */ `
//     query TransfersByAddressesLatestQuery($limit: Int!, $addresses: [String!]!) {
//       data: v1_ibc_union_fungible_asset_orders(
//         limit: $limit
//         order_by: [
//           { packet_send_timestamp: desc }
//           { packet_send_transaction_hash: desc }
//         ]
//         where: {
//           _or: [
//             { normalized_sender: { _in: $addresses } }
//             { normalized_receiver: { _in: $addresses } }
//           ]
//         }
//       ) {
//         ...TransferListData
//       }
//     }
//   `,
//   [transferListDataFragment]
// )
