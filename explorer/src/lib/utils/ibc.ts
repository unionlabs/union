// IBC packet and trace utilities

export interface IBCPacketEvent {
  type: "send" | "recv" | "ack" | "timeout"
  sequence: string
  sourcePort: string
  sourceChannel: string
  destPort: string
  destChannel: string
  data?: string
  timeoutHeight?: string
  timeoutTimestamp?: string
}

export interface FungibleTokenPacket {
  sender: string
  receiver: string
  denom: string
  amount: string
}

export interface DenomTrace {
  path: string
  baseDenom: string
}

export interface IBCTrace {
  packets: IBCPacketEvent[]
  fungibleTokenPacket?: FungibleTokenPacket
  denomTrace?: DenomTrace
  writeAck?: {
    packetAck: string
  }
}

type EventAttribute = { key: string; value: string }
type Event = { type: string; attributes: EventAttribute[] }

function getAttr(attrs: EventAttribute[], key: string): string | undefined {
  return attrs.find(a => a.key === key)?.value
}

/**
 * Extract IBC-related information from transaction events
 * Handles both standard Cosmos IBC events and CosmWasm-style events (wasm-prefixed)
 */
export function extractIBCTrace(events: Event[]): IBCTrace | null {
  const packets: IBCPacketEvent[] = []
  let fungibleTokenPacket: FungibleTokenPacket | undefined
  let denomTrace: DenomTrace | undefined
  let writeAck: { packetAck: string } | undefined

  for (const event of events) {
    const type = event.type

    // Send packet event (standard or wasm)
    if (type === "send_packet" || type === "wasm-packet_send") {
      packets.push({
        type: "send",
        sequence: getAttr(event.attributes, "packet_sequence") ?? getAttr(event.attributes, "sequence") ?? "",
        sourcePort: getAttr(event.attributes, "packet_src_port") ?? getAttr(event.attributes, "source_port") ?? "",
        sourceChannel: getAttr(event.attributes, "packet_src_channel") ?? getAttr(event.attributes, "source_channel") ?? getAttr(event.attributes, "channel_id") ?? "",
        destPort: getAttr(event.attributes, "packet_dst_port") ?? getAttr(event.attributes, "dest_port") ?? "",
        destChannel: getAttr(event.attributes, "packet_dst_channel") ?? getAttr(event.attributes, "dest_channel") ?? "",
        data: getAttr(event.attributes, "packet_data") ?? getAttr(event.attributes, "data"),
        timeoutHeight: getAttr(event.attributes, "packet_timeout_height") ?? getAttr(event.attributes, "timeout_height"),
        timeoutTimestamp: getAttr(event.attributes, "packet_timeout_timestamp") ?? getAttr(event.attributes, "timeout_timestamp"),
      })
    }

    // Receive packet event (standard or wasm)
    if (type === "recv_packet" || type === "wasm-packet_recv") {
      packets.push({
        type: "recv",
        sequence: getAttr(event.attributes, "packet_sequence") ?? getAttr(event.attributes, "sequence") ?? "",
        sourcePort: getAttr(event.attributes, "packet_src_port") ?? getAttr(event.attributes, "source_port") ?? "",
        sourceChannel: getAttr(event.attributes, "packet_src_channel") ?? getAttr(event.attributes, "source_channel") ?? getAttr(event.attributes, "channel_id") ?? "",
        destPort: getAttr(event.attributes, "packet_dst_port") ?? getAttr(event.attributes, "dest_port") ?? "",
        destChannel: getAttr(event.attributes, "packet_dst_channel") ?? getAttr(event.attributes, "dest_channel") ?? "",
        data: getAttr(event.attributes, "packet_data") ?? getAttr(event.attributes, "data"),
      })
    }

    // Acknowledge packet event (standard or wasm)
    if (type === "acknowledge_packet" || type === "wasm-packet_ack") {
      packets.push({
        type: "ack",
        sequence: getAttr(event.attributes, "packet_sequence") ?? getAttr(event.attributes, "sequence") ?? "",
        sourcePort: getAttr(event.attributes, "packet_src_port") ?? getAttr(event.attributes, "source_port") ?? "",
        sourceChannel: getAttr(event.attributes, "packet_src_channel") ?? getAttr(event.attributes, "source_channel") ?? getAttr(event.attributes, "channel_id") ?? "",
        destPort: getAttr(event.attributes, "packet_dst_port") ?? getAttr(event.attributes, "dest_port") ?? "",
        destChannel: getAttr(event.attributes, "packet_dst_channel") ?? getAttr(event.attributes, "dest_channel") ?? "",
      })
    }

    // Timeout packet event (standard or wasm)
    if (type === "timeout_packet" || type === "wasm-packet_timeout") {
      packets.push({
        type: "timeout",
        sequence: getAttr(event.attributes, "packet_sequence") ?? getAttr(event.attributes, "sequence") ?? "",
        sourcePort: getAttr(event.attributes, "packet_src_port") ?? getAttr(event.attributes, "source_port") ?? "",
        sourceChannel: getAttr(event.attributes, "packet_src_channel") ?? getAttr(event.attributes, "source_channel") ?? getAttr(event.attributes, "channel_id") ?? "",
        destPort: getAttr(event.attributes, "packet_dst_port") ?? getAttr(event.attributes, "dest_port") ?? "",
        destChannel: getAttr(event.attributes, "packet_dst_channel") ?? getAttr(event.attributes, "dest_channel") ?? "",
      })
    }

    // Fungible token packet (ICS-20 transfer)
    if (type === "fungible_token_packet") {
      fungibleTokenPacket = {
        sender: getAttr(event.attributes, "sender") ?? "",
        receiver: getAttr(event.attributes, "receiver") ?? "",
        denom: getAttr(event.attributes, "denom") ?? "",
        amount: getAttr(event.attributes, "amount") ?? "",
      }
    }

    // Denomination trace
    if (type === "denomination_trace") {
      denomTrace = {
        path: getAttr(event.attributes, "trace_hash") ?? getAttr(event.attributes, "path") ?? "",
        baseDenom: getAttr(event.attributes, "denom") ?? getAttr(event.attributes, "base_denom") ?? "",
      }
    }

    // Write acknowledgement (standard or wasm)
    if (type === "write_acknowledgement" || type === "wasm-write_ack") {
      writeAck = {
        packetAck: getAttr(event.attributes, "packet_ack") ?? getAttr(event.attributes, "acknowledgement") ?? "",
      }
    }
  }

  // Return null if no IBC events found
  if (packets.length === 0 && !fungibleTokenPacket && !denomTrace && !writeAck) {
    return null
  }

  return {
    packets,
    fungibleTokenPacket,
    denomTrace,
    writeAck,
  }
}

/**
 * Get a friendly name for packet type (similar to app2's naming)
 */
export function getPacketTypeName(type: IBCPacketEvent["type"]): string {
  switch (type) {
    case "send": return "Packet Sent"
    case "recv": return "Packet Received"
    case "ack": return "Packet Acknowledged"
    case "timeout": return "Packet Timeout"
  }
}

/**
 * Get a short label for packet type
 */
export function getPacketTypeLabel(type: IBCPacketEvent["type"]): string {
  switch (type) {
    case "send": return "SEND"
    case "recv": return "RECV"
    case "ack": return "ACK"
    case "timeout": return "TIMEOUT"
  }
}

/**
 * Get status color for packet type
 */
export function getPacketTypeColor(type: IBCPacketEvent["type"]): "success" | "default" | "destructive" {
  switch (type) {
    case "send": return "default"
    case "recv": return "success"
    case "ack": return "success"
    case "timeout": return "destructive"
  }
}

/**
 * Parse packet data JSON if possible
 */
export function parsePacketData(data: string | undefined): Record<string, unknown> | null {
  if (!data) return null
  try {
    return JSON.parse(data)
  } catch {
    // Try base64 decode
    try {
      const decoded = atob(data)
      return JSON.parse(decoded)
    } catch {
      return null
    }
  }
}
