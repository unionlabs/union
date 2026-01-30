<script lang="ts">
import { matchPromiseWithCache } from "$lib/snippet-cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import * as Collapsible from "$lib/components/ui/collapsible/index.js"
import * as Tabs from "$lib/components/ui/tabs/index.js"
import { sectionHeader } from "$lib/components/ui/snippets.svelte"
import { formatTime, formatAmount, truncateAddress } from "$lib/utils/format"
import { getMsgType, getMsgTypeVariant } from "$lib/utils/messages"
import { copyToClipboard as copyUtil } from "$lib/utils/clipboard"
import type { TxResponse } from "$lib/types/cosmos"
import ChevronDownIcon from "@lucide/svelte/icons/chevron-down"
import CopyIcon from "@lucide/svelte/icons/copy"
import CheckIcon from "@lucide/svelte/icons/check"
import XIcon from "@lucide/svelte/icons/x"
import ArrowRightIcon from "@lucide/svelte/icons/arrow-right"
import ExternalLinkIcon from "@lucide/svelte/icons/external-link"
import LoaderIcon from "@lucide/svelte/icons/loader"
import SendIcon from "@lucide/svelte/icons/send"
import PackageIcon from "@lucide/svelte/icons/package"
import CheckCircleIcon from "@lucide/svelte/icons/check-circle"
import AlertCircleIcon from "@lucide/svelte/icons/alert-circle"
import CornerMarks from "$lib/components/corner-marks.svelte"
import { extractIBCTrace, getPacketTypeName, getPacketTypeLabel, getPacketTypeColor, parsePacketData, type IBCTrace, type IBCPacketEvent } from "$lib/utils/ibc"
import { fetchPacketDetailsByTxHash, type PacketDetails, type PacketTrace } from "$lib/queries/packets"
import TokenMovements from "$lib/components/token-movements.svelte"
import { urls } from "$lib/utils/urls"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenBase = $derived(data.chain.assets[0]?.base ?? "")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

// Get exponent for a denom - use chain's native exponent for native token, default 6 for others
const getExponent = (denom: string): number => {
  if (denom === tokenBase) return tokenExponent
  return 6 // Default exponent for other tokens
}

let copied = $state<string | null>(null)
let decodedMessages = $state<Record<number, unknown> | null>(null)
let decodingError = $state<string | null>(null)
let isDecoding = $state(false)

// Packet trace from indexer
let packetDetails = $state<PacketDetails | null>(null)
let packetLoading = $state(false)
let packetError = $state<string | null>(null)

async function loadPacketDetails(txHash: string) {
  if (packetLoading || packetDetails) return
  packetLoading = true
  packetError = null
  try {
    packetDetails = await fetchPacketDetailsByTxHash(txHash)
  } catch (e) {
    packetError = String(e)
  } finally {
    packetLoading = false
  }
}

// JSON-RPC helper for voy.run
async function voyagerRpc(method: string, params: unknown): Promise<unknown> {
  const response = await fetch("https://voy.run", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      method,
      params,
      id: Date.now()
    })
  })
  const json = await response.json()
  if (json.error) throw new Error(json.error.message)
  return json.result
}

// Client type configurations for different scenarios
const CLIENT_CONFIGS = [
  { clientType: "parlia", ibcInterface: "ibc-cosmwasm" },
  { clientType: "cometbls", ibcInterface: "ibc-solidity" },
  { clientType: "cometbls", ibcInterface: "ibc-cosmwasm" },
  { clientType: "ethereum", ibcInterface: "ibc-cosmwasm" },
  { clientType: "tendermint", ibcInterface: "ibc-cosmwasm" },
]

// Decode header using voyager_decodeHeader (array params)
async function decodeHeader(
  clientType: string,
  ibcInterface: string,
  headerBytes: string
): Promise<unknown> {
  const bytes = headerBytes.startsWith("0x") ? headerBytes : `0x${headerBytes}`
  // voyager_decodeHeader takes array params: [client_type, ibc_interface, ibc_spec_id, header]
  return voyagerRpc("voyager_decodeHeader", [clientType, ibcInterface, "ibc-union", bytes])
}

// Try decoding header with multiple client configs until one works
async function tryDecodeHeader(
  hexBytes: string
): Promise<{ decoded: unknown; config: { clientType: string; ibcInterface: string } } | null> {
  const bytes = hexBytes.startsWith("0x") ? hexBytes : `0x${hexBytes}`

  for (const config of CLIENT_CONFIGS) {
    try {
      const result = await decodeHeader(config.clientType, config.ibcInterface, bytes)
      return { decoded: result, config }
    } catch {
      continue
    }
  }
  return null
}

// Try decoding with multiple client configs until one works
async function tryDecodeWithConfigs(
  decodeType: "clientState" | "consensusState",
  hexBytes: string
): Promise<{ decoded: unknown; config: { clientType: string; ibcInterface: string } } | null> {
  const methodMap = {
    clientState: "voyager_decodeClientState",
    consensusState: "voyager_decodeConsensusState",
  }

  const bytes = hexBytes.startsWith("0x") ? hexBytes : `0x${hexBytes}`

  for (const config of CLIENT_CONFIGS) {
    try {
      const paramKey = decodeType === "clientState" ? "client_state" : "consensus_state"

      const result = await voyagerRpc(methodMap[decodeType], {
        client_type: config.clientType,
        ibc_interface: config.ibcInterface,
        ibc_spec_id: "ibc-union",
        [paramKey]: bytes
      })
      return { decoded: result, config }
    } catch {
      continue
    }
  }
  return null
}

// Convert base64 to hex
function base64ToHex(base64: string): string {
  try {
    const binary = atob(base64)
    let hex = "0x"
    for (let i = 0; i < binary.length; i++) {
      hex += binary.charCodeAt(i).toString(16).padStart(2, "0")
    }
    return hex
  } catch {
    return base64
  }
}

// Recursively find and decode hex/base64 blobs in a message
async function decodeMessageFields(obj: unknown, path = ""): Promise<{ original: unknown; decoded: Record<string, unknown>; errors: Record<string, string> }> {
  const decodedFields: Record<string, unknown> = {}
  const decodeErrors: Record<string, string> = {}

  if (!obj || typeof obj !== "object") {
    return { original: obj, decoded: decodedFields, errors: decodeErrors }
  }

  const record = obj as Record<string, unknown>

  for (const [key, value] of Object.entries(record)) {
    const currentPath = path ? `${path}.${key}` : key

    // Check if this looks like encoded data
    if (typeof value === "string") {
      // Hex encoded (0x prefix)
      if (value.startsWith("0x") && value.length > 66) {
        // Try to decode as client_message/header (common in update_client)
        if (key === "client_message" || key === "header") {
          try {
            // Use voyager_decodeHeader for headers
            const result = await tryDecodeHeader(value)
            if (result) {
              decodedFields[`${currentPath}_decoded`] = {
                ...result.decoded as object,
                _decoded_as: `${result.config.clientType}/${result.config.ibcInterface}`
              }
            }
          } catch (e) {
            decodedFields[`${currentPath}_error`] = String(e)
          }
        }
        // Try to decode as client_state
        else if (key === "client_state" || key.includes("client_state")) {
          try {
            const result = await tryDecodeWithConfigs("clientState", value)
            if (result) {
              decodedFields[`${currentPath}_decoded`] = {
                ...result.decoded as object,
                _decoded_as: `${result.config.clientType}/${result.config.ibcInterface}`
              }
            }
          } catch (e) {
            decodedFields[`${currentPath}_error`] = String(e)
          }
        }
        // Try to decode as consensus_state
        else if (key === "consensus_state" || key.includes("consensus_state")) {
          try {
            const result = await tryDecodeWithConfigs("consensusState", value)
            if (result) {
              decodedFields[`${currentPath}_decoded`] = {
                ...result.decoded as object,
                _decoded_as: `${result.config.clientType}/${result.config.ibcInterface}`
              }
            }
          } catch (e) {
            decodedFields[`${currentPath}_error`] = String(e)
          }
        }
      }
      // Base64 encoded (check if it's valid base64 and reasonably long)
      else if (value.length > 100 && /^[A-Za-z0-9+/]+=*$/.test(value)) {
        const hexBytes = base64ToHex(value)
        if (hexBytes.startsWith("0x")) {
          try {
            const result = await tryDecodeWithConfigs("clientState", hexBytes)
            if (result) {
              decodedFields[`${currentPath}_decoded`] = {
                ...result.decoded as object,
                _decoded_as: `${result.config.clientType}/${result.config.ibcInterface}`
              }
            }
          } catch {
            // Ignore decode errors for base64
          }
        }
      }
    }
    // Recurse into nested objects
    else if (typeof value === "object" && value !== null) {
      const nested = await decodeMessageFields(value, currentPath)
      Object.assign(decodedFields, nested.decoded)
    }
  }

  return { original: obj, decoded: decodedFields, errors: decodeErrors }
}

// Fetch decoded messages from voy.run
async function fetchDecodedMessages(messages: unknown[]) {
  if (isDecoding || decodedMessages) return

  isDecoding = true
  decodingError = null

  try {
    const decoded: Record<number, unknown> = {}

    for (let i = 0; i < messages.length; i++) {
      const msg = messages[i] as Record<string, unknown>

      try {
        const result = await decodeMessageFields(msg)

        if (Object.keys(result.decoded).length > 0) {
          decoded[i] = {
            ...msg,
            _voyager_decoded: result.decoded
          }
        } else {
          decoded[i] = msg
        }
      } catch {
        decoded[i] = msg
      }
    }

    decodedMessages = decoded
  } catch (e) {
    decodingError = String(e)
  } finally {
    isDecoding = false
  }
}

function copyToClipboard(text: string) {
  copyUtil(text, () => {
    copied = text
    setTimeout(() => (copied = null), 2000)
  })
}

// Format message for display
const formatMsgValue = (key: string, value: unknown): { display: string; isAddress: boolean; isAmount: boolean } => {
  if (value === null || value === undefined) return { display: "-", isAddress: false, isAmount: false }

  // Check if it's an address
  if (typeof value === "string" && (key.includes("address") || key === "sender" || key === "receiver" || key === "voter" || key === "proposer" || key === "delegator" || key === "validator")) {
    return { display: value, isAddress: true, isAmount: false }
  }

  // Check if it's an amount/coin
  if (typeof value === "object" && "amount" in (value as object) && "denom" in (value as object)) {
    const coin = value as { amount: string; denom: string }
    return { display: `${formatAmount(coin.amount, getExponent(coin.denom))} ${coin.denom}`, isAddress: false, isAmount: true }
  }

  // Array of coins
  if (Array.isArray(value) && value.length > 0 && typeof value[0] === "object" && "amount" in value[0]) {
    const coins = value as { amount: string; denom: string }[]
    return { display: coins.map((c) => `${formatAmount(c.amount, getExponent(c.denom))} ${c.denom}`).join(", "), isAddress: false, isAmount: true }
  }

  if (typeof value === "object") {
    return { display: JSON.stringify(value), isAddress: false, isAmount: false }
  }

  return { display: String(value), isAddress: false, isAmount: false }
}

// Get important fields from a message
const getMessageFields = (msg: Record<string, unknown>) => {
  const skipFields = ["@type"]
  const importantFields = ["sender", "from_address", "receiver", "to_address", "delegator_address", "validator_address", "amount", "voter", "proposal_id", "option"]

  const fields: { key: string; value: unknown; important: boolean }[] = []

  // First add important fields in order
  for (const key of importantFields) {
    if (key in msg && !skipFields.includes(key)) {
      fields.push({ key, value: msg[key], important: true })
    }
  }

  // Then add remaining fields
  for (const [key, value] of Object.entries(msg)) {
    if (!skipFields.includes(key) && !importantFields.includes(key)) {
      fields.push({ key, value, important: false })
    }
  }

  return fields
}

// Get a summary/subtitle for a message to show in the header
const getMessageSummary = (msg: Record<string, unknown>): string | null => {
  const type = (msg["@type"] as string) ?? ""

  // MsgExecuteContract - show action name from msg field
  if (type.includes("MsgExecuteContract")) {
    const execMsg = msg["msg"] as Record<string, unknown> | undefined
    if (execMsg && typeof execMsg === "object") {
      const action = Object.keys(execMsg)[0]
      if (action) return action
    }
  }

  // MsgInstantiateContract - show label or code_id
  if (type.includes("MsgInstantiateContract")) {
    const label = msg["label"] as string | undefined
    if (label) return label
    const codeId = msg["code_id"] as string | undefined
    if (codeId) return `code ${codeId}`
  }

  // MsgUpdateClient - show client_id
  if (type.includes("MsgUpdateClient")) {
    const clientId = msg["client_id"] as string | undefined
    if (clientId) return clientId
  }

  // MsgRecvPacket/MsgAcknowledgement - show channel
  if (type.includes("MsgRecvPacket") || type.includes("MsgAcknowledgement")) {
    const packet = msg["packet"] as Record<string, unknown> | undefined
    if (packet) {
      const dstChannel = packet["destination_channel"] as string | undefined
      if (dstChannel) return dstChannel
    }
  }

  // MsgTransfer - show channel
  if (type.includes("MsgTransfer")) {
    const channel = msg["source_channel"] as string | undefined
    if (channel) return channel
  }

  // MsgVote - show proposal id
  if (type.includes("MsgVote")) {
    const proposalId = msg["proposal_id"] as string | undefined
    if (proposalId) return `proposal ${proposalId}`
  }

  // MsgDelegate/MsgUndelegate/MsgBeginRedelegate - show validator (truncated)
  if (type.includes("Delegate") || type.includes("Redelegate")) {
    const validator = msg["validator_address"] as string | undefined
    if (validator) return truncateAddress(validator, 6)
  }

  return null
}

// Group events by msg_index attribute
type EventWithIndex = { type: string; attributes: { key: string; value: string }[] }
function groupEventsByMessage(events: EventWithIndex[]): Map<number, EventWithIndex[]> {
  const grouped = new Map<number, EventWithIndex[]>()

  for (const event of events) {
    const msgIndexAttr = event.attributes.find(a => a.key === "msg_index")
    const msgIndex = msgIndexAttr ? parseInt(msgIndexAttr.value) : 0

    if (!grouped.has(msgIndex)) grouped.set(msgIndex, [])
    grouped.get(msgIndex)!.push(event)
  }

  return grouped
}

// Get event category for coloring
function getEventCategory(type: string): string {
  if (type.startsWith("wasm")) return "wasm"
  if (type.includes("packet") || type.includes("channel") || type.includes("connection") || type.includes("client") || type.includes("ibc") || type.includes("acknowledgement")) return "ibc"
  if (type.includes("transfer") || type === "coin_spent" || type === "coin_received") return "transfer"
  if (type.includes("delegate") || type.includes("unbond") || type.includes("redelegate") || type.includes("withdraw")) return "staking"
  if (type.includes("proposal") || type.includes("vote") || type.includes("deposit")) return "governance"
  if (type === "tx" || type === "message") return "system"
  return "other"
}

function getEventCategoryColor(cat: string): string {
  const colors: Record<string, string> = {
    ibc: "border-info text-info bg-info/10",
    wasm: "border-purple-500 text-purple-400 bg-purple-500/10",
    transfer: "border-success text-success bg-success/10",
    staking: "border-warning text-warning bg-warning/10",
    governance: "border-cyan-500 text-cyan-400 bg-cyan-500/10",
    system: "border-muted-foreground/50 text-muted-foreground bg-muted/20",
    other: "border-border text-foreground bg-muted/10"
  }
  return colors[cat] ?? colors.other
}

// Format event attribute value for display
function formatEventAttrValue(key: string, value: string): { type: "address" | "amount" | "hex" | "json" | "id" | "text"; display: string; full: string; parsed?: unknown } {
  // Address detection
  if (/^(union1|cosmos1|osmo1|0x)[a-z0-9]{38,}$/i.test(value)) {
    return { type: "address", display: `${value.slice(0, 10)}...${value.slice(-6)}`, full: value }
  }
  // Large number (likely amount)
  if (/^\d{10,}$/.test(value)) {
    return { type: "amount", display: formatAmount(value, tokenExponent), full: value }
  }
  // Hex data
  if (value.startsWith("0x") && value.length > 20) {
    return { type: "hex", display: `${value.slice(0, 14)}...${value.slice(-8)}`, full: value }
  }
  // JSON data
  if ((value.startsWith("{") || value.startsWith("[")) && value.length > 50) {
    try {
      const parsed = JSON.parse(value)
      return { type: "json", display: JSON.stringify(parsed).slice(0, 40) + "...", full: value, parsed }
    } catch {
      return { type: "text", display: value, full: value }
    }
  }
  // Channel/port IDs
  if (value.startsWith("channel-") || value.startsWith("connection-") || value.startsWith("07-tendermint-")) {
    return { type: "id", display: value, full: value }
  }
  return { type: "text", display: value.length > 60 ? value.slice(0, 60) + "..." : value, full: value }
}
</script>

{#snippet loading()}
  <div class="space-y-6">
    <Skeleton class="h-24 w-full" />
    <Skeleton class="h-64 w-full" />
    <Skeleton class="h-48 w-full" />
  </div>
{/snippet}


{#snippet packetTraceTimeline(packet: PacketDetails)}
  {@const traces = packet.traces}
  {@const TRACE_NAMES: Record<string, string> = {
    "PACKET_SEND": "Packet Sent",
    "PACKET_SEND_LC_UPDATE_L0": "LC Update L0",
    "PACKET_SEND_LC_UPDATE_L1": "LC Update L1",
    "PACKET_SEND_LC_UPDATE_L2": "LC Update L2",
    "PACKET_RECV": "Packet Received",
    "WRITE_ACK": "Ack Written",
    "WRITE_ACK_LC_UPDATE_L0": "LC Update L0",
    "WRITE_ACK_LC_UPDATE_L1": "LC Update L1",
    "WRITE_ACK_LC_UPDATE_L2": "LC Update L2",
    "PACKET_ACK": "Packet Acknowledged",
  }}
  <div class="p-4">
    <div class="relative">
      {#each traces as trace, i}
        {@const isLast = i === traces.length - 1}
        {@const isInit = trace.type === "PACKET_SEND"}
        {@const isSuccess = trace.type === "PACKET_RECV" || trace.type === "PACKET_ACK" || trace.type === "WRITE_ACK"}
        {@const isLcUpdate = trace.type.includes("LC_UPDATE")}
        <div class="flex gap-4">
          <!-- Timeline column -->
          <div class="flex flex-col items-center w-8">
            <div class="w-8 h-8 flex items-center justify-center shrink-0 border {
              isInit ? 'bg-info/10 border-info' :
              isSuccess ? 'bg-success/10 border-success' :
              isLcUpdate ? 'bg-warning/10 border-warning' : 'bg-muted border-border'
            }">
              {#if trace.type === "PACKET_SEND"}
                <SendIcon class="h-4 w-4 text-info" />
              {:else if trace.type === "PACKET_RECV"}
                <PackageIcon class="h-4 w-4 text-success" />
              {:else if trace.type === "PACKET_ACK" || trace.type === "WRITE_ACK"}
                <CheckCircleIcon class="h-4 w-4 text-success" />
              {:else if isLcUpdate}
                <ArrowRightIcon class="h-3 w-3 text-warning" />
              {:else}
                <ArrowRightIcon class="h-3 w-3 text-muted-foreground" />
              {/if}
            </div>
            {#if !isLast}
              <div class="w-px h-full min-h-6 {
                isInit ? 'bg-info/50' :
                isSuccess ? 'bg-success/50' :
                isLcUpdate ? 'bg-warning/50' : 'bg-border'
              }"></div>
            {/if}
          </div>

          <!-- Content column -->
          <div class="flex-1 min-w-0 {!isLast ? 'pb-3' : ''}">
            <div class="flex items-center gap-2 mb-0.5">
              <span class="text-sm font-medium {isLcUpdate ? 'text-muted-foreground' : ''}">{TRACE_NAMES[trace.type] ?? trace.type}</span>
              {#if trace.transaction_hash}
                <CheckIcon class="h-3 w-3 text-success" />
              {/if}
            </div>
            <div class="text-xs text-muted-foreground font-mono">{trace.chain.universal_chain_id}</div>
            {#if trace.height}
              <div class="text-xs text-muted-foreground">
                Block #{trace.height}
                {#if trace.timestamp}
                  · {new Date(trace.timestamp).toLocaleString()}
                {/if}
              </div>
            {/if}
            {#if trace.transaction_hash}
              <div class="text-xs font-mono text-muted-foreground truncate mt-1">
                tx: {trace.transaction_hash.slice(0, 16)}...
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet ibcTraceSection(trace: IBCTrace, txHash: string)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center gap-3">
        <span class="text-[10px] font-mono text-muted-foreground">IBC</span>
        <PackageIcon class="h-4 w-4 text-muted-foreground" />
        <span class="text-xs font-medium uppercase tracking-wider">IBC Packet Trace</span>
      </div>
      <div class="flex items-center gap-2">
        {#if packetDetails}
          <Badge variant="success">indexed</Badge>
        {:else if packetLoading}
          <LoaderIcon class="h-3 w-3 animate-spin text-muted-foreground" />
        {:else}
          <button
            onclick={() => loadPacketDetails(txHash)}
            class="text-[10px] font-mono text-muted-foreground hover:text-foreground transition-colors"
          >
            Load full trace
          </button>
        {/if}
      </div>
    </div>

    <!-- Rich packet details from indexer -->
    {#if packetDetails}
      <!-- Status & Packet Hash -->
      <div class="px-4 py-3 border-b border-border bg-muted/10">
        <div class="flex items-center justify-between mb-2">
          <Badge variant={packetDetails.status === "PACKET_ACK" ? "success" : "default"}>{packetDetails.status}</Badge>
          <span class="text-[10px] font-mono text-muted-foreground">{packetDetails.packet_hash.slice(0, 20)}...</span>
        </div>
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div>
            <div class="text-[9px] font-mono uppercase text-muted-foreground">Source Chain</div>
            <div class="font-mono">{packetDetails.source_universal_chain_id}</div>
          </div>
          <div>
            <div class="text-[9px] font-mono uppercase text-muted-foreground">Destination Chain</div>
            <div class="font-mono">{packetDetails.destination_universal_chain_id}</div>
          </div>
        </div>
      </div>

      <!-- Timeline from indexer -->
      {#if packetDetails.traces.length > 0}
        {@render packetTraceTimeline(packetDetails)}
      {/if}

      <!-- Decoded data if available -->
      {#if packetDetails.decoded_flattened}
        <Collapsible.Root>
          <Collapsible.Trigger class="w-full px-4 py-2 border-t border-border hover:bg-muted/20 transition-colors">
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
              <ChevronDownIcon class="h-3 w-3 transition-transform [[data-state=open]_&]:rotate-180" />
              <span class="font-mono">decoded_data</span>
            </div>
          </Collapsible.Trigger>
          <Collapsible.Content>
            <pre class="text-xs font-mono bg-muted/30 p-4 border-t border-border overflow-x-auto max-h-48">{JSON.stringify(packetDetails.decoded_flattened, null, 2)}</pre>
          </Collapsible.Content>
        </Collapsible.Root>
      {/if}

    <!-- Fallback: Basic event-based view -->
    {:else}
      <!-- Token Transfer Summary (if exists) -->
      {#if trace.fungibleTokenPacket}
        <div class="p-4 border-b border-border">
          <div class="flex items-center justify-between mb-3">
            <div class="text-[10px] font-mono uppercase tracking-wider text-success">Token Transfer</div>
            <div class="text-lg font-mono font-bold text-success">{formatAmount(trace.fungibleTokenPacket.amount, tokenExponent)}</div>
          </div>
          <div class="grid grid-cols-[1fr_auto_1fr] gap-2 items-center">
            <div class="border border-border p-2 bg-muted/20">
              <div class="text-[9px] font-mono uppercase tracking-wider text-muted-foreground mb-0.5">From</div>
              <a href={urls.account(trace.fungibleTokenPacket.sender)} class="text-sm font-mono hover:underline truncate block">
                {truncateAddress(trace.fungibleTokenPacket.sender, 10)}
              </a>
            </div>
            <ArrowRightIcon class="h-4 w-4 text-muted-foreground" />
            <div class="border border-border p-2 bg-muted/20">
              <div class="text-[9px] font-mono uppercase tracking-wider text-muted-foreground mb-0.5">To</div>
              <a href={urls.account(trace.fungibleTokenPacket.receiver)} class="text-sm font-mono hover:underline truncate block">
                {truncateAddress(trace.fungibleTokenPacket.receiver, 10)}
              </a>
            </div>
          </div>
          <div class="mt-3 border border-border p-2">
            <div class="text-[9px] font-mono uppercase tracking-wider text-muted-foreground mb-0.5">Asset</div>
            <div class="text-xs font-mono truncate">{trace.fungibleTokenPacket.denom}</div>
          </div>
        </div>
      {/if}

      <!-- Vertical Timeline for Packet Events (chronological: oldest at top) -->
      {#if trace.packets.length > 0}
        {@const packets = [...trace.packets].reverse()}
        <div class="p-4">
          <div class="relative">
            {#each packets as packet, i}
              {@const isLast = i === packets.length - 1}
              {@const isInit = packet.type === 'send'}
              {@const isSuccess = packet.type === 'recv' || packet.type === 'ack'}
              {@const isError = packet.type === 'timeout'}
              <div class="flex gap-4">
                <!-- Timeline column -->
                <div class="flex flex-col items-center w-8">
                  <!-- Node box -->
                  <div class="w-8 h-8 flex items-center justify-center shrink-0 border {
                    isError ? 'bg-destructive/10 border-destructive' :
                    isSuccess ? 'bg-success/10 border-success' :
                    isInit ? 'bg-info/10 border-info' : 'bg-muted border-border'
                  }">
                    {#if packet.type === "send"}
                      <SendIcon class="h-4 w-4 text-info" />
                    {:else if packet.type === "recv"}
                      <PackageIcon class="h-4 w-4 text-success" />
                    {:else if packet.type === "ack"}
                      <CheckCircleIcon class="h-4 w-4 text-success" />
                    {:else}
                      <AlertCircleIcon class="h-4 w-4 text-destructive" />
                    {/if}
                  </div>
                  <!-- Vertical line -->
                  {#if !isLast}
                    <div class="w-px h-full min-h-8 {
                      isInit ? 'bg-info/50' :
                      isSuccess ? 'bg-success/50' : 'bg-border'
                    }"></div>
                  {/if}
                </div>

                <!-- Content column -->
                <div class="flex-1 min-w-0 {!isLast ? 'pb-4' : ''}">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-sm font-medium">{getPacketTypeName(packet.type)}</span>
                    <span class="text-[10px] font-mono px-1.5 py-0.5 border border-border">{getPacketTypeLabel(packet.type)}</span>
                  </div>
                  <div class="text-xs text-muted-foreground font-mono mb-2">seq {packet.sequence}</div>

                  <!-- Channel info -->
                  <div class="grid grid-cols-2 gap-px bg-border text-xs">
                    <div class="bg-background p-2">
                      <div class="text-[9px] font-mono uppercase tracking-wider text-muted-foreground mb-0.5">Source</div>
                      <div class="font-mono truncate">{packet.sourceChannel}</div>
                      <div class="text-muted-foreground truncate">{packet.sourcePort}</div>
                    </div>
                    <div class="bg-background p-2">
                      <div class="text-[9px] font-mono uppercase tracking-wider text-muted-foreground mb-0.5">Destination</div>
                      <div class="font-mono truncate">{packet.destChannel}</div>
                      <div class="text-muted-foreground truncate">{packet.destPort}</div>
                    </div>
                  </div>

                  <!-- Packet data (collapsible) -->
                  {#if packet.data}
                    {@const parsedData = parsePacketData(packet.data)}
                    {#if parsedData}
                      <Collapsible.Root>
                        <Collapsible.Trigger class="w-full mt-2">
                          <div class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors">
                            <ChevronDownIcon class="h-3 w-3 transition-transform [[data-state=open]_&]:rotate-180" />
                            <span class="font-mono">packet_data</span>
                          </div>
                        </Collapsible.Trigger>
                        <Collapsible.Content>
                          <pre class="text-xs font-mono bg-muted/30 p-2 border border-border overflow-x-auto max-h-32 mt-2">{JSON.stringify(parsedData, null, 2)}</pre>
                        </Collapsible.Content>
                      </Collapsible.Root>
                    {/if}
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Write Acknowledgement -->
      {#if trace.writeAck}
        <div class="px-4 pb-4">
          <div class="border border-success/50 bg-success/5 p-3">
            <div class="flex items-center gap-2 mb-2">
              <CheckCircleIcon class="h-4 w-4 text-success" />
              <span class="text-xs font-medium">Acknowledgement Written</span>
            </div>
            <div class="text-xs font-mono bg-background p-2 border border-border break-all max-h-20 overflow-auto">{trace.writeAck.packetAck}</div>
          </div>
        </div>
      {/if}

      <!-- Denom Trace -->
      {#if trace.denomTrace}
        <div class="px-4 pb-4">
          <div class="border border-border p-3">
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Denomination Trace</div>
            <div class="space-y-1 text-xs">
              <div class="flex gap-2">
                <span class="text-muted-foreground shrink-0">Path:</span>
                <span class="font-mono break-all">{trace.denomTrace.path}</span>
              </div>
              <div class="flex gap-2">
                <span class="text-muted-foreground shrink-0">Base:</span>
                <span class="font-mono">{trace.denomTrace.baseDenom}</span>
              </div>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
{/snippet}

{#snippet copyButton(text: string)}
  <button
    onclick={() => copyToClipboard(text)}
    class="p-1 hover:bg-muted transition-colors shrink-0"
  >
    {#if copied === text}
      <CheckIcon class="h-3 w-3 text-success" />
    {:else}
      <CopyIcon class="h-3 w-3 text-muted-foreground" />
    {/if}
  </button>
{/snippet}

{#snippet messageEvents(msgEvents: EventWithIndex[])}
  {@const filteredEvents = msgEvents.filter(e => e.type !== "message")}
  {#if filteredEvents.length > 0}
    <div class="border-t border-border bg-muted/10">
      <Collapsible.Root>
        <Collapsible.Trigger class="w-full">
          <div class="flex items-center gap-2 px-4 py-2.5 hover:bg-muted/20 transition-colors">
            <ChevronDownIcon class="h-3.5 w-3.5 text-muted-foreground shrink-0 transition-transform [[data-state=open]_&]:rotate-180" />
            <span class="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">Event Logs</span>
            <span class="text-[10px] text-muted-foreground/70">({filteredEvents.length})</span>
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border/50">
            {#each filteredEvents as event, eventIdx}
              {@const category = getEventCategory(event.type)}
              {@const categoryColor = getEventCategoryColor(category)}
              {@const attrs = event.attributes.filter(a => a.key !== "msg_index")}
              <Collapsible.Root>
                <Collapsible.Trigger class="w-full">
                  <div class="flex items-center gap-3 px-4 py-2 hover:bg-muted/20 transition-colors border-b border-border/30">
                    <span class="text-[10px] font-mono text-muted-foreground/60 w-4">{eventIdx}</span>
                    <ChevronDownIcon class="h-3 w-3 text-muted-foreground/50 shrink-0 transition-transform [[data-state=open]_&]:rotate-180" />
                    <span class="text-[11px] font-mono px-2 py-0.5 border {categoryColor}">{event.type}</span>
                    <span class="text-[10px] text-muted-foreground/50 ml-auto">{attrs.length} attributes</span>
                  </div>
                </Collapsible.Trigger>
                <Collapsible.Content>
                  <div class="bg-background border-b border-border/30">
                    {#each attrs as attr, attrIdx}
                      {@const formatted = formatEventAttrValue(attr.key, attr.value)}
                      <div class="flex items-start gap-3 px-4 py-2 hover:bg-muted/5 text-xs border-b border-border/20 last:border-b-0">
                        <span class="text-[10px] font-mono text-muted-foreground/50 w-4 pt-0.5">{attrIdx}</span>
                        <span class="text-muted-foreground font-mono w-36 shrink-0 truncate pt-0.5">{attr.key}</span>
                        <div class="flex items-center gap-1.5 min-w-0 flex-1">
                          {#if formatted.type === "address"}
                            <a href={urls.account(formatted.full)} class="text-info hover:underline font-mono">{formatted.display}</a>
                          {:else if formatted.type === "amount"}
                            <span class="text-success font-mono">{formatted.display}</span>
                            <span class="text-muted-foreground text-[10px]">({formatted.full})</span>
                          {:else if formatted.type === "hex"}
                            <span class="text-warning font-mono">{formatted.display}</span>
                          {:else if formatted.type === "id"}
                            <span class="text-cyan-400 font-mono">{formatted.display}</span>
                          {:else if formatted.type === "json"}
                            <span class="text-purple-400 font-mono">{formatted.display}</span>
                          {:else}
                            <span class="font-mono break-all">{formatted.display}</span>
                          {/if}
                          {#if formatted.full !== formatted.display}
                            <button onclick={() => copyToClipboard(formatted.full)} class="p-0.5 hover:bg-muted shrink-0">
                              {#if copied === formatted.full}
                                <CheckIcon class="h-2.5 w-2.5 text-success" />
                              {:else}
                                <CopyIcon class="h-2.5 w-2.5 text-muted-foreground" />
                              {/if}
                            </button>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>
                </Collapsible.Content>
              </Collapsible.Root>
            {/each}
          </div>
        </Collapsible.Content>
      </Collapsible.Root>
    </div>
  {/if}
{/snippet}

{#snippet txDetails(result: { tx_response: TxResponse })}
  {@const tx = result.tx_response}
  {@const messages = tx.tx.body.messages}
  {@const events = tx.events ?? []}
  {@const eventsByMsg = groupEventsByMessage(events)}
  {@const fee = tx.tx.auth_info.fee.amount}
  {@const gasPercent = tx.gas_wanted !== "0" ? ((Number(tx.gas_used) / Number(tx.gas_wanted)) * 100).toFixed(1) : "0"}

  <div class="space-y-6">
    <!-- Header Card -->
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <!-- Status Row -->
        <div class="flex items-center gap-4 mb-4">
          <div class="w-10 h-10 flex items-center justify-center {tx.code === 0 ? 'bg-success/20' : 'bg-destructive/20'}">
            {#if tx.code === 0}
              <CheckIcon class="h-5 w-5 text-success" />
            {:else}
              <XIcon class="h-5 w-5 text-destructive" />
            {/if}
          </div>
          <div>
            <div class="text-lg font-medium">{tx.code === 0 ? "Transaction Successful" : "Transaction Failed"}</div>
            <div class="text-xs text-muted-foreground">{formatTime(tx.timestamp)}</div>
          </div>
        </div>

        <!-- Hash -->
        <div class="flex items-center gap-2 p-3 bg-muted/30 mb-4">
          <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground shrink-0">TX HASH</span>
          <span class="font-mono text-sm break-all flex-1">{tx.txhash}</span>
          {@render copyButton(tx.txhash)}
        </div>

        <!-- Quick Stats -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Block</div>
            <a href={urls.block(tx.height)} class="font-mono text-sm hover:underline">#{tx.height}</a>
          </div>
          <div>
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Gas Used</div>
            <div class="font-mono text-sm">{Number(tx.gas_used).toLocaleString()} <span class="text-muted-foreground">({gasPercent}%)</span></div>
          </div>
          <div>
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Fee</div>
            <div class="font-mono text-sm">{fee.length > 0 ? fee.map((c) => `${formatAmount(c.amount, getExponent(c.denom))} ${c.denom}`).join(", ") : "0"}</div>
          </div>
          <div>
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Messages</div>
            <div class="font-mono text-sm">{messages.length}</div>
          </div>
        </div>

        <!-- Error if failed -->
        {#if tx.code !== 0}
          <div class="mt-4 p-3 bg-destructive/10 border border-destructive/20">
            <div class="text-[10px] font-mono uppercase tracking-wider text-destructive mb-1">Error</div>
            <div class="text-sm font-mono text-destructive break-all">{tx.raw_log}</div>
          </div>
        {/if}

        <!-- Memo if present -->
        {#if tx.tx.body.memo}
          <div class="mt-4 p-3 bg-muted/30">
            <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Memo</div>
            <div class="text-sm">{tx.tx.body.memo}</div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Token Movements -->
    <TokenMovements events={events} messageCount={messages.length} />

    <!-- Messages -->
    <div class="relative border border-border overflow-hidden">
      <CornerMarks />

      <Tabs.Root value="parsed" class="w-full min-w-0">
        <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-muted/20">
          <div class="flex items-center gap-3">
            <span class="text-[10px] font-mono text-muted-foreground">01</span>
            <span class="text-xs font-medium uppercase tracking-wider">Messages</span>
            <span class="text-xs text-muted-foreground">({messages.length})</span>
          </div>
          <Tabs.List class="h-7 bg-muted/50 p-0.5">
            <Tabs.Trigger value="parsed" class="text-[10px] font-mono px-2 h-6 data-[state=active]:bg-background">Parsed</Tabs.Trigger>
            <Tabs.Trigger
              value="decoded"
              class="text-[10px] font-mono px-2 h-6 data-[state=active]:bg-background"
              onclick={() => fetchDecodedMessages(messages)}
            >
              Decoded
            </Tabs.Trigger>
            <Tabs.Trigger value="raw" class="text-[10px] font-mono px-2 h-6 data-[state=active]:bg-background">Raw</Tabs.Trigger>
          </Tabs.List>
        </div>

        <!-- Parsed Tab -->
        <Tabs.Content value="parsed" class="mt-0 p-4 space-y-4">
          {#each messages as msg, i}
            {@const msgType = getMsgType(msg as { "@type": string })}
            {@const msgSummary = getMessageSummary(msg as Record<string, unknown>)}
            {@const fields = getMessageFields(msg as Record<string, unknown>)}
            {@const msgEvents = eventsByMsg.get(i) ?? []}
            {@const eventCount = msgEvents.filter(e => e.type !== "message").length}
            <div class="border border-border bg-card">
              <Collapsible.Root open={i === 0}>
                <!-- Message Header -->
                <Collapsible.Trigger class="w-full">
                  <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
                    <div class="flex items-center gap-3">
                      <div class="w-7 h-7 flex items-center justify-center bg-muted/50 text-xs font-mono text-muted-foreground shrink-0">
                        {String(i + 1).padStart(2, "0")}
                      </div>
                      <div class="flex items-center gap-2 min-w-0">
                        <Badge variant={getMsgTypeVariant(msgType)}>{msgType}</Badge>
                        {#if msgSummary}
                          <span class="text-[11px] font-mono px-2 py-0.5 bg-muted/50 text-muted-foreground truncate">{msgSummary}</span>
                        {/if}
                      </div>
                    </div>
                    <div class="flex items-center gap-3 shrink-0">
                      {#if eventCount > 0}
                        <span class="text-[10px] text-muted-foreground px-2 py-1 bg-muted/30">{eventCount} events</span>
                      {/if}
                      <ChevronDownIcon class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
                    </div>
                  </div>
                </Collapsible.Trigger>

                <Collapsible.Content>
                  <!-- Message Fields -->
                  <div class="border-t border-border">
                    {#each fields as field}
                      {@const formatted = formatMsgValue(field.key, field.value)}
                      <div class="flex items-start gap-4 px-4 py-3 hover:bg-muted/10 border-b border-border/50 last:border-b-0 {field.important ? '' : 'opacity-60'} min-w-0">
                        <span class="text-[11px] font-mono text-muted-foreground w-36 shrink-0 pt-0.5">{field.key}</span>
                        <div class="flex-1 min-w-0 overflow-hidden">
                          {#if formatted.isAddress}
                            <div class="flex items-center gap-2">
                              <a href={urls.account(formatted.display)} class="font-mono text-sm hover:underline text-info truncate">{truncateAddress(formatted.display, 12)}</a>
                              {@render copyButton(formatted.display)}
                            </div>
                          {:else if formatted.isAmount}
                            <span class="font-mono text-sm text-success font-medium">{formatted.display}</span>
                          {:else if formatted.display.startsWith("{") || formatted.display.startsWith("[")}
                            <pre class="text-xs font-mono overflow-x-auto max-h-32 whitespace-pre-wrap break-all text-muted-foreground bg-muted/20 p-2">{JSON.stringify(JSON.parse(formatted.display), null, 2)}</pre>
                          {:else}
                            <span class="text-sm font-mono break-all">{formatted.display}</span>
                          {/if}
                        </div>
                      </div>
                    {/each}
                  </div>

                  <!-- Events for this message -->
                  {@render messageEvents(msgEvents)}
                </Collapsible.Content>
              </Collapsible.Root>
            </div>
          {/each}
        </Tabs.Content>

        <!-- Decoded Tab -->
        <Tabs.Content value="decoded" class="mt-0 p-4 space-y-4">
          {#if isDecoding}
            <div class="flex items-center justify-center gap-2 py-12 text-sm text-muted-foreground">
              <LoaderIcon class="h-4 w-4 animate-spin" />
              Decoding messages via voy.run...
            </div>
          {:else if decodingError}
            <div class="p-4 text-sm text-destructive border border-destructive/30 bg-destructive/5">
              Failed to decode: {decodingError}
            </div>
          {:else if decodedMessages}
            {#each messages as msg, i}
              {@const msgType = getMsgType(msg as { "@type": string })}
              {@const msgSummary = getMessageSummary(msg as Record<string, unknown>)}
              {@const decoded = decodedMessages[i] as Record<string, unknown>}
              {@const voyagerDecoded = decoded?._voyager_decoded as Record<string, unknown> | undefined}
              {@const hasDecoded = voyagerDecoded && Object.keys(voyagerDecoded).length > 0}
              <div class="border border-border bg-card">
                <Collapsible.Root open={i === 0}>
                  <Collapsible.Trigger class="w-full">
                    <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
                      <div class="flex items-center gap-3">
                        <div class="w-7 h-7 flex items-center justify-center bg-muted/50 text-xs font-mono text-muted-foreground shrink-0">
                          {String(i + 1).padStart(2, "0")}
                        </div>
                        <div class="flex items-center gap-2 min-w-0">
                          <Badge variant={getMsgTypeVariant(msgType)}>{msgType}</Badge>
                          {#if msgSummary}
                            <span class="text-[11px] font-mono px-2 py-0.5 bg-muted/50 text-muted-foreground truncate">{msgSummary}</span>
                          {/if}
                          {#if hasDecoded}
                            <Badge variant="success">decoded</Badge>
                          {:else}
                            <span class="text-[10px] text-muted-foreground">no decodable fields</span>
                          {/if}
                        </div>
                      </div>
                      <ChevronDownIcon class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180 shrink-0" />
                    </div>
                  </Collapsible.Trigger>
                  <Collapsible.Content>
                    <div class="border-t border-border overflow-hidden">
                      {#if hasDecoded}
                        <!-- Decoded fields section -->
                        <div class="p-4 border-b border-border bg-success/5">
                          <div class="text-[10px] font-mono uppercase tracking-wider text-success mb-3">Decoded via voy.run</div>
                          {#each Object.entries(voyagerDecoded) as [path, value]}
                            <div class="mb-4 last:mb-0 min-w-0">
                              <div class="text-[10px] font-mono text-muted-foreground mb-1 truncate">{path}</div>
                              <pre class="text-xs font-mono overflow-x-auto max-h-64 whitespace-pre-wrap break-all bg-background p-2 border border-border">{JSON.stringify(value, null, 2)}</pre>
                            </div>
                          {/each}
                        </div>
                      {/if}
                      <!-- Original message -->
                      <div class="p-4 min-w-0">
                        <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-3">Original Message</div>
                        <pre class="text-xs font-mono overflow-x-auto max-h-64 whitespace-pre-wrap break-all bg-muted/20 p-2">{JSON.stringify(msg, null, 2)}</pre>
                      </div>
                    </div>
                  </Collapsible.Content>
                </Collapsible.Root>
              </div>
            {/each}
          {:else}
            <div class="p-8 text-center text-sm text-muted-foreground border border-dashed border-border">
              Click to fetch decoded messages from voy.run
            </div>
          {/if}
        </Tabs.Content>

        <!-- Raw Tab -->
        <Tabs.Content value="raw" class="mt-0 p-4">
          <div class="p-4 bg-muted/10 border border-border overflow-hidden">
            <pre class="text-xs font-mono overflow-x-auto max-h-[600px] whitespace-pre-wrap break-all">{JSON.stringify(messages, null, 2)}</pre>
          </div>
        </Tabs.Content>
      </Tabs.Root>
    </div>

    <!-- IBC Trace -->
    {#if true}
      {@const ibcTrace = extractIBCTrace(events)}
      {#if ibcTrace}
        {@render ibcTraceSection(ibcTrace, tx.txhash)}
      {/if}
    {/if}

    <!-- Signers -->
    {#if tx.tx.auth_info.signer_infos.length > 0}
      <div class="relative border border-border">
        <CornerMarks />
        {@render sectionHeader("Signers", "02", undefined, tx.tx.auth_info.signer_infos.length)}

        {#each tx.tx.auth_info.signer_infos as signer, i}
          <div class="px-4 py-3 border-b border-border last:border-b-0 hover:bg-muted/30">
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-3">
                <span class="text-xs font-mono text-muted-foreground">{String(i + 1).padStart(2, "0")}</span>
                <span class="text-[10px] font-mono px-2 py-1 bg-muted">{signer.public_key?.["@type"]?.split(".").pop() ?? "Unknown"}</span>
              </div>
              <span class="text-xs text-muted-foreground">Sequence: {signer.sequence}</span>
            </div>
            {#if signer.public_key?.key}
              <div class="flex items-center gap-2 pl-9">
                <span class="font-mono text-xs break-all text-muted-foreground">{signer.public_key.key}</span>
                {@render copyButton(signer.public_key.key)}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Raw Events (collapsed reference) -->
    {#if events.length > 0}
      <Collapsible.Root>
        <div class="relative border border-border overflow-hidden">
          <CornerMarks />
          <Collapsible.Trigger class="w-full">
            <div class="flex items-center justify-between px-4 py-3 hover:bg-muted/30 transition-colors">
              <div class="flex items-center gap-3">
                <span class="text-[10px] font-mono text-muted-foreground">03</span>
                <span class="text-xs font-medium uppercase tracking-wider">All Events (Raw)</span>
                <span class="text-xs text-muted-foreground">({events.length})</span>
              </div>
              <ChevronDownIcon class="h-4 w-4 text-muted-foreground transition-transform [[data-state=open]_&]:rotate-180" />
            </div>
          </Collapsible.Trigger>
          <Collapsible.Content>
            <div class="border-t border-border p-4 bg-muted/5 max-h-[500px] overflow-auto">
              <pre class="text-xs font-mono whitespace-pre-wrap break-all">{JSON.stringify(events, null, 2)}</pre>
            </div>
          </Collapsible.Content>
        </div>
      </Collapsible.Root>
    {/if}
  </div>
{/snippet}

{#snippet error(err: unknown)}
  {@const errStr = String(err)}
  {@const isNotFound = errStr.includes("500") || errStr.includes("404") || errStr.includes("not found")}
  <div class="relative border border-destructive/50">
    <CornerMarks />
    <div class="p-6">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-10 h-10 flex items-center justify-center bg-destructive/20">
          <XIcon class="h-5 w-5 text-destructive" />
        </div>
        <div>
          <p class="text-lg font-medium text-destructive">
            {isNotFound ? "Transaction Not Found" : "Failed to load transaction"}
          </p>
          <p class="text-xs text-muted-foreground">
            {#if isNotFound}
              This transaction may not exist or the block containing it has been pruned from the node.
            {:else}
              An error occurred while loading the transaction.
            {/if}
          </p>
        </div>
      </div>
      {#if !isNotFound}
        <div class="p-3 bg-muted/30 mb-4">
          <p class="text-xs text-muted-foreground font-mono break-all">{errStr}</p>
        </div>
      {/if}
      <a href={urls.transactions()} class="inline-flex items-center gap-2 text-xs font-mono uppercase tracking-wider hover:underline">
        <ArrowRightIcon class="h-3 w-3 rotate-180" />
        Back to transactions
      </a>
    </div>
  </div>
{/snippet}

{@render matchPromiseWithCache(data.transaction, {
  cacheKey: `${cachePrefix}tx:${data.hash}`,
  onLoading: loading,
  onSuccess: txDetails,
  onError: error,
})}
