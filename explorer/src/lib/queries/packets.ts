// Packet queries using Union's GraphQL indexer

import { graphqlFetch } from "./graphql"

export interface PacketTrace {
  type: string
  height: number | null
  block_hash: string | null
  timestamp: string | null
  transaction_hash: string | null
  chain: {
    universal_chain_id: string
    rpc_type: string
  }
}

export interface PacketDetails {
  packet_hash: string
  status: string
  data: string | null
  decoded: unknown
  decoded_flattened: unknown
  acknowledgement: string | null
  channel_version: string | null

  // Source
  source_universal_chain_id: string
  source_channel_id: string
  source_port_id: string
  source_client_id: string
  source_connection_id: string

  // Destination
  destination_universal_chain_id: string
  destination_channel_id: string
  destination_port_id: string
  destination_client_id: string
  destination_connection_id: string

  // Timeout
  timeout_height: string | null
  timeout_timestamp: string | null

  // Send info
  packet_send_block_hash: string | null
  packet_send_height: number | null
  packet_send_timestamp: string | null
  packet_send_transaction_hash: string | null

  // Recv info
  packet_recv_block_hash: string | null
  packet_recv_height: number | null
  packet_recv_timestamp: string | null
  packet_recv_transaction_hash: string | null
  packet_recv_maker: string | null
  packet_recv_maker_msg: string | null

  // Write ack info
  write_ack_block_hash: string | null
  write_ack_height: number | null
  write_ack_timestamp: string | null
  write_ack_transaction_hash: string | null

  // Ack info
  packet_ack_block_hash: string | null
  packet_ack_height: number | null
  packet_ack_timestamp: string | null
  packet_ack_transaction_hash: string | null
  packet_ack_maker: string | null

  // Traces
  traces: PacketTrace[]
}

const PACKET_HASH_BY_TX_QUERY = `
  query GetPacketHashByTxHash($tx_hash: String!) {
    v2_packets(args: { p_transaction_hash: $tx_hash }) {
      packet_hash
    }
  }
`

const PACKET_DETAILS_QUERY = `
  query PacketDetails($packet_hash: String!) {
    v2_packets(args: { p_packet_hash: $packet_hash }) {
      packet_hash
      status
      data
      decoded
      decoded_flattened
      acknowledgement
      channel_version
      source_universal_chain_id
      source_channel_id
      source_port_id
      source_client_id
      source_connection_id
      destination_universal_chain_id
      destination_channel_id
      destination_port_id
      destination_client_id
      destination_connection_id
      timeout_height
      timeout_timestamp
      packet_send_block_hash
      packet_send_height
      packet_send_timestamp
      packet_send_transaction_hash
      packet_recv_block_hash
      packet_recv_height
      packet_recv_timestamp
      packet_recv_transaction_hash
      packet_recv_maker
      packet_recv_maker_msg
      write_ack_block_hash
      write_ack_height
      write_ack_timestamp
      write_ack_transaction_hash
      packet_ack_block_hash
      packet_ack_height
      packet_ack_timestamp
      packet_ack_transaction_hash
      packet_ack_maker
      traces {
        type
        height
        block_hash
        timestamp
        transaction_hash
        chain {
          universal_chain_id
          rpc_type
        }
      }
    }
  }
`

/**
 * Get packet hash from a transaction hash
 */
export async function fetchPacketHashByTxHash(txHash: string): Promise<string | null> {
  try {
    const data = await graphqlFetch<{ v2_packets: { packet_hash: string }[] }>(
      PACKET_HASH_BY_TX_QUERY,
      { tx_hash: txHash }
    )
    return data.v2_packets[0]?.packet_hash ?? null
  } catch {
    return null
  }
}

/**
 * Get full packet details by packet hash
 */
export async function fetchPacketDetails(packetHash: string): Promise<PacketDetails | null> {
  try {
    const data = await graphqlFetch<{ v2_packets: PacketDetails[] }>(
      PACKET_DETAILS_QUERY,
      { packet_hash: packetHash }
    )
    return data.v2_packets[0] ?? null
  } catch {
    return null
  }
}

/**
 * Get packet details by transaction hash (combines both queries)
 */
export async function fetchPacketDetailsByTxHash(txHash: string): Promise<PacketDetails | null> {
  const packetHash = await fetchPacketHashByTxHash(txHash)
  if (!packetHash) return null
  return fetchPacketDetails(packetHash)
}
