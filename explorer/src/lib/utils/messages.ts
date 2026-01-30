/**
 * Utility functions for handling Cosmos transaction messages
 */

import type { BadgeVariant } from "$lib/components/ui/badge"

/**
 * Extract the message type from a Cosmos message @type field
 * @param msg - Message object with @type field
 * @returns Human-readable message type (e.g., "Send", "Delegate", "Vote")
 */
export function getMsgType(msg: { "@type": string }): string {
  const type = msg["@type"] ?? ""
  return type.split(".").pop()?.replace("Msg", "") ?? "Unknown"
}

/**
 * Get a badge variant for a message type
 * @param type - Message type string (from getMsgType)
 * @returns Badge variant name
 */
export function getMsgTypeVariant(type: string): BadgeVariant {
  const t = type.toLowerCase()
  if (t.includes("send") || t.includes("transfer")) {
    return "info"
  }
  if (t.includes("delegate") || t.includes("stake")) {
    return "purple"
  }
  if (t.includes("vote")) {
    return "warning"
  }
  if (t.includes("submit") || t.includes("proposal")) {
    return "cyan"
  }
  if (t.includes("withdraw")) {
    return "emerald"
  }
  if (t.includes("ibc") || t.includes("channel") || t.includes("client")) {
    return "orange"
  }
  if (t.includes("exec") || t.includes("contract")) {
    return "pink"
  }
  return "secondary"
}
