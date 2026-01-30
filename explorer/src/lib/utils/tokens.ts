// Token movement extraction from transaction events

import type { Coin } from "$lib/types/cosmos"

export interface TokenMovement {
  type: "native" | "cw20"
  direction: "sent" | "received" | "burned" | "minted"
  address: string // who sent or received
  amount: string
  denom: string // native denom or CW20 contract address
  msgIndex: number
}

export interface TokenMovementSummary {
  movements: TokenMovement[]
  perMessage: Map<number, TokenMovement[]>
  totals: {
    sent: Map<string, bigint> // denom -> total amount
    received: Map<string, bigint>
  }
  netFlow: Map<string, bigint> // denom -> net (positive = received, negative = sent)
}

type TxEvent = {
  type: string
  attributes: Array<{ key: string; value: string; index?: boolean }>
}

function getAttr(attrs: Array<{ key: string; value: string }>, key: string): string | undefined {
  return attrs.find((a) => a.key === key)?.value
}

function getMsgIndex(attrs: Array<{ key: string; value: string }>): number {
  const idx = getAttr(attrs, "msg_index")
  return idx ? parseInt(idx) : 0
}

// Parse amount string like "1000000uatom" or "500000ibc/..."
function parseAmountDenom(amountStr: string): { amount: string; denom: string } | null {
  // Match number followed by denom (which can include /, -, letters, numbers)
  const match = amountStr.match(/^(\d+)(.+)$/)
  if (!match) return null
  return { amount: match[1], denom: match[2] }
}

// Parse comma-separated amounts like "1000uatom,500uosmo"
function parseAmounts(amountStr: string): Array<{ amount: string; denom: string }> {
  if (!amountStr) return []
  return amountStr
    .split(",")
    .map((s) => parseAmountDenom(s.trim()))
    .filter((x): x is { amount: string; denom: string } => x !== null)
}

export function extractTokenMovements(events: TxEvent[]): TokenMovementSummary {
  const movements: TokenMovement[] = []

  for (const event of events) {
    const msgIndex = getMsgIndex(event.attributes)

    // Native token: coin_spent
    if (event.type === "coin_spent") {
      const spender = getAttr(event.attributes, "spender")
      const amount = getAttr(event.attributes, "amount")
      if (spender && amount) {
        for (const parsed of parseAmounts(amount)) {
          movements.push({
            type: "native",
            direction: "sent",
            address: spender,
            amount: parsed.amount,
            denom: parsed.denom,
            msgIndex,
          })
        }
      }
    }

    // Native token: coin_received
    if (event.type === "coin_received") {
      const receiver = getAttr(event.attributes, "receiver")
      const amount = getAttr(event.attributes, "amount")
      if (receiver && amount) {
        for (const parsed of parseAmounts(amount)) {
          movements.push({
            type: "native",
            direction: "received",
            address: receiver,
            amount: parsed.amount,
            denom: parsed.denom,
            msgIndex,
          })
        }
      }
    }

    // Native token: transfer event (has recipient and sender)
    if (event.type === "transfer") {
      const sender = getAttr(event.attributes, "sender")
      const recipient = getAttr(event.attributes, "recipient")
      const amount = getAttr(event.attributes, "amount")
      // Note: coin_spent/coin_received usually accompany transfer, so we track
      // transfer events separately for more context but may dedupe later
    }

    // Native token: burn
    if (event.type === "burn") {
      const burner = getAttr(event.attributes, "burner")
      const amount = getAttr(event.attributes, "amount")
      if (burner && amount) {
        for (const parsed of parseAmounts(amount)) {
          movements.push({
            type: "native",
            direction: "burned",
            address: burner,
            amount: parsed.amount,
            denom: parsed.denom,
            msgIndex,
          })
        }
      }
    }

    // Native token: mint
    if (event.type === "mint" || event.type === "coinbase") {
      const minter = getAttr(event.attributes, "minter") || getAttr(event.attributes, "receiver")
      const amount = getAttr(event.attributes, "amount")
      if (minter && amount) {
        for (const parsed of parseAmounts(amount)) {
          movements.push({
            type: "native",
            direction: "minted",
            address: minter,
            amount: parsed.amount,
            denom: parsed.denom,
            msgIndex,
          })
        }
      }
    }

    // CW20 token transfers via wasm events
    if (event.type === "wasm" || event.type.startsWith("wasm-")) {
      const action = getAttr(event.attributes, "action")
      const contractAddr = getAttr(event.attributes, "_contract_address") || getAttr(event.attributes, "contract_address")

      if (!contractAddr) continue

      // CW20 transfer
      if (action === "transfer" || action === "transfer_from") {
        const from = getAttr(event.attributes, "from") || getAttr(event.attributes, "sender")
        const to = getAttr(event.attributes, "to") || getAttr(event.attributes, "recipient")
        const amount = getAttr(event.attributes, "amount")

        if (from && amount) {
          movements.push({
            type: "cw20",
            direction: "sent",
            address: from,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
        if (to && amount) {
          movements.push({
            type: "cw20",
            direction: "received",
            address: to,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
      }

      // CW20 send (transfer to contract)
      if (action === "send" || action === "send_from") {
        const from = getAttr(event.attributes, "from") || getAttr(event.attributes, "sender")
        const to = getAttr(event.attributes, "to") || getAttr(event.attributes, "contract")
        const amount = getAttr(event.attributes, "amount")

        if (from && amount) {
          movements.push({
            type: "cw20",
            direction: "sent",
            address: from,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
        if (to && amount) {
          movements.push({
            type: "cw20",
            direction: "received",
            address: to,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
      }

      // CW20 burn
      if (action === "burn" || action === "burn_from") {
        const from = getAttr(event.attributes, "from") || getAttr(event.attributes, "sender")
        const amount = getAttr(event.attributes, "amount")

        if (from && amount) {
          movements.push({
            type: "cw20",
            direction: "burned",
            address: from,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
      }

      // CW20 mint
      if (action === "mint") {
        const to = getAttr(event.attributes, "to") || getAttr(event.attributes, "recipient")
        const amount = getAttr(event.attributes, "amount")

        if (to && amount) {
          movements.push({
            type: "cw20",
            direction: "minted",
            address: to,
            amount,
            denom: contractAddr,
            msgIndex,
          })
        }
      }
    }
  }

  // Group by message
  const perMessage = new Map<number, TokenMovement[]>()
  for (const m of movements) {
    if (!perMessage.has(m.msgIndex)) perMessage.set(m.msgIndex, [])
    perMessage.get(m.msgIndex)!.push(m)
  }

  // Calculate totals
  const sent = new Map<string, bigint>()
  const received = new Map<string, bigint>()

  for (const m of movements) {
    if (m.direction === "sent" || m.direction === "burned") {
      sent.set(m.denom, (sent.get(m.denom) ?? 0n) + BigInt(m.amount))
    }
    if (m.direction === "received" || m.direction === "minted") {
      received.set(m.denom, (received.get(m.denom) ?? 0n) + BigInt(m.amount))
    }
  }

  // Calculate net flow
  const netFlow = new Map<string, bigint>()
  const allDenoms = new Set([...sent.keys(), ...received.keys()])
  for (const denom of allDenoms) {
    const s = sent.get(denom) ?? 0n
    const r = received.get(denom) ?? 0n
    netFlow.set(denom, r - s)
  }

  return {
    movements,
    perMessage,
    totals: { sent, received },
    netFlow,
  }
}

// Format denom for display (shorten IBC denoms, etc.)
export function formatDenom(denom: string): string {
  if (denom.startsWith("ibc/")) {
    return `ibc/${denom.slice(4, 10)}...`
  }
  if (denom.length > 20) {
    return `${denom.slice(0, 8)}...${denom.slice(-6)}`
  }
  return denom
}

// Check if denom is an IBC denom
export function isIBCDenom(denom: string): boolean {
  return denom.startsWith("ibc/")
}

// Check if denom looks like a CW20 contract address
export function isCW20Denom(denom: string): boolean {
  // Cosmos contract addresses are typically bech32 with specific prefixes
  return denom.length > 40 && (denom.includes("1") || denom.startsWith("0x"))
}

// Get movement direction color
export function getMovementColor(direction: TokenMovement["direction"]): string {
  switch (direction) {
    case "sent":
      return "text-destructive"
    case "received":
      return "text-success"
    case "burned":
      return "text-warning"
    case "minted":
      return "text-info"
    default:
      return "text-foreground"
  }
}

// Get movement direction icon name
export function getMovementIcon(direction: TokenMovement["direction"]): string {
  switch (direction) {
    case "sent":
      return "arrow-up-right"
    case "received":
      return "arrow-down-left"
    case "burned":
      return "flame"
    case "minted":
      return "sparkles"
    default:
      return "circle"
  }
}
