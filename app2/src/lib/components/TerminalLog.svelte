<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"

// Extended transfer type with server pre-computed fields
type EnhancedTransferListItem = TransferListItem & {
  isTestnetTransfer?: boolean
  sourceDisplayName?: string
  destinationDisplayName?: string
  formattedTimestamp?: string
  routeKey?: string
  senderDisplay?: string
  receiverDisplay?: string
}
import { Option } from "effect"
import { onMount } from "svelte"
import { transactionAudio } from "../../routes/test/audio"
import Card from "./ui/Card.svelte"

let {
  transfers = [],
  selectedFromChain = null,
  selectedToChain = null,
}: {
  transfers: EnhancedTransferListItem[]
  selectedFromChain?: string | null
  selectedToChain?: string | null
} = $props()

interface LogEntry {
  id: number
  timestamp: string
  type: string
  message: string
  sourceChain: string
  destChain: string
  hash: string
  sender?: string | undefined
  receiver?: string | undefined
  sourceChainId?: string | undefined
  destChainId?: string | undefined
}

let allLogs: LogEntry[] = $state([])
let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let containerElement: HTMLElement
let processedCount = $state(0)
let logIdCounter = $state(0)

// Canvas settings
let canvasWidth = $state(400)
let canvasHeight = $state(600)
let scrollOffset = $state(0)
let maxScrollOffset = $state(0)
let isMouseDown = false
let lastMouseY = 0

// Visual constants
const LOG_HEIGHT = 48
const PADDING = 8
const LINE_HEIGHT = 12
const FONT_SIZE = 10
const COLORS = {
  background: "#00000000",
  text: "#d4d4d8", // zinc-300
  textSecondary: "#9ca3af", // gray-400
  filter: "#f59e0b", // amber-400
  filterBg: "#27272a", // zinc-800
  border: "#3f3f46", // zinc-700
}

// Optimization: Cache chain display names
const chainDisplayNameCache = new Map<string, string>()

const getChainDisplayName = (universalChainId: string): string => {
  if (chainDisplayNameCache.has(universalChainId)) {
    return chainDisplayNameCache.get(universalChainId)!
  }

  let displayName = universalChainId

  if (Option.isSome(chains.data)) {
    const chain = chains.data.value.find(c => c.universal_chain_id === universalChainId)
    if (chain) {
      displayName = chain.display_name || chain.chain_id
    }
  } else {
    const simpleNames: Record<string, string> = {
      "union-testnet-8": "Union",
      "osmo-test-5": "Osmosis",
      "sepolia-1": "Ethereum",
      "stride-internal-1": "Stride",
    }
    displayName = simpleNames[universalChainId] || universalChainId
  }

  chainDisplayNameCache.set(universalChainId, displayName)
  return displayName
}

const formatHash = (hash: string): string => {
  return hash ? `${hash.slice(0, 8)}...${hash.slice(-4)}` : "N/A"
}

// Display logs without filtering (server handles filtering now)
let filteredLogs = $derived(allLogs.slice(0, 1000))

// Canvas drawing functions
function updateCanvasSize() {
  if (!containerElement || !canvas) {
    return
  }

  const rect = containerElement.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  canvasWidth = rect.width
  canvasHeight = rect.height

  canvas.width = canvasWidth * dpr
  canvas.height = canvasHeight * dpr
  canvas.style.width = canvasWidth + "px"
  canvas.style.height = canvasHeight + "px"

  if (ctx) {
    ctx.scale(dpr, dpr)
    ctx.textBaseline = "top"
    setupCanvasFont()
  }

  // Update max scroll
  const filterHeight = (selectedFromChain || selectedToChain) ? 32 : 0
  const contentHeight = filteredLogs.length * LOG_HEIGHT
  const viewportHeight = canvasHeight - filterHeight
  maxScrollOffset = Math.max(0, contentHeight - viewportHeight)
}

function setupCanvasFont() {
  if (!ctx) {
    return
  }
  ctx.font =
    `${FONT_SIZE}px ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace`
  ctx.textAlign = "left"
}

function drawFilterBar() {
  if (!selectedFromChain && !selectedToChain) {
    return
  }

  ctx.fillStyle = COLORS.filterBg
  ctx.fillRect(0, 0, canvasWidth, 32)

  // Border line
  ctx.fillStyle = COLORS.border
  ctx.fillRect(0, 31, canvasWidth, 1)

  ctx.fillStyle = COLORS.filter
  ctx.font = `${
    FONT_SIZE - 1
  }px ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace`

  let filterText = "Server filter: "
  if (selectedFromChain && selectedToChain) {
    filterText += `${getChainDisplayName(selectedFromChain)} → ${
      getChainDisplayName(selectedToChain)
    }`
  } else if (selectedFromChain) {
    filterText += `from ${getChainDisplayName(selectedFromChain)}`
  } else if (selectedToChain) {
    filterText += `to ${getChainDisplayName(selectedToChain)}`
  }

  ctx.fillText(filterText, PADDING, 10)
  setupCanvasFont() // Reset font
}

function drawLogEntry(log: LogEntry, y: number) {
  const baseY = y + PADDING

  // Message line
  ctx.fillStyle = COLORS.text
  ctx.fillText(log.message, PADDING, baseY)

  // Hash line
  ctx.fillText(`tx: ${log.hash}`, PADDING, baseY + LINE_HEIGHT)

  let currentY = baseY + LINE_HEIGHT * 2

  // Sender line
  if (log.sender) {
    ctx.fillStyle = COLORS.textSecondary
    ctx.fillText(`from: ${log.sender}`, PADDING, currentY)
    currentY += LINE_HEIGHT
  }

  // Receiver line
  if (log.receiver) {
    ctx.fillStyle = COLORS.textSecondary
    ctx.fillText(`to: ${log.receiver}`, PADDING, currentY)
  }
}

function drawCanvas() {
  if (!ctx) {
    return
  }

  // Clear canvas
  ctx.fillStyle = COLORS.background
  ctx.fillRect(0, 0, canvasWidth, canvasHeight)

  // Draw filter bar
  const filterHeight = (selectedFromChain || selectedToChain) ? 32 : 0
  drawFilterBar()

  // Calculate visible log range (virtual scrolling)
  const viewportTop = scrollOffset
  const viewportBottom = viewportTop + (canvasHeight - filterHeight)

  const startIndex = Math.max(0, Math.floor(viewportTop / LOG_HEIGHT))
  const endIndex = Math.min(filteredLogs.length, Math.ceil(viewportBottom / LOG_HEIGHT) + 1)

  // Save context for clipping
  ctx.save()
  ctx.beginPath()
  ctx.rect(0, filterHeight, canvasWidth, canvasHeight - filterHeight)
  ctx.clip()

  // Draw visible logs
  for (let i = startIndex; i < endIndex; i++) {
    const logY = (i * LOG_HEIGHT) - scrollOffset + filterHeight
    if (logY > -LOG_HEIGHT && logY < canvasHeight) {
      drawLogEntry(filteredLogs[i], logY)
    }
  }

  ctx.restore()

  // Draw "waiting" message if no logs
  if (filteredLogs.length === 0) {
    ctx.fillStyle = "#6b7280" // zinc-500
    ctx.textAlign = "center"
    const centerY = canvasHeight / 2
    ctx.fillText("Waiting for transfers...", canvasWidth / 2, centerY - 10)
    ctx.fillText("_", canvasWidth / 2, centerY + 10)
    ctx.textAlign = "left"
  }
}

// Scroll handling
function handleWheel(event: WheelEvent) {
  event.preventDefault()
  const delta = event.deltaY
  scrollOffset = Math.max(0, Math.min(maxScrollOffset, scrollOffset + delta))
  drawCanvas()
}

function handleMouseDown(event: MouseEvent) {
  isMouseDown = true
  lastMouseY = event.clientY
}

function handleMouseMove(event: MouseEvent) {
  if (!isMouseDown) {
    return
  }

  const deltaY = event.clientY - lastMouseY
  scrollOffset = Math.max(0, Math.min(maxScrollOffset, scrollOffset - deltaY))
  lastMouseY = event.clientY
  drawCanvas()
}

function handleMouseUp() {
  isMouseDown = false
}

// Add log function
const addLog = (
  type: string,
  sourceChain: string,
  destChain: string,
  hash: string,
  sender?: string,
  receiver?: string,
  sourceChainId?: string,
  destChainId?: string,
) => {
  const timestamp = new Date().toLocaleTimeString()
  logIdCounter++

  const logEntry: LogEntry = {
    id: logIdCounter,
    timestamp,
    type,
    message: `${sourceChain} → ${destChain}`,
    sourceChain,
    destChain,
    hash,
    sender,
    receiver,
    sourceChainId,
    destChainId,
  }

  // Add to beginning of array
  allLogs = [logEntry, ...allLogs].slice(0, 1000) // Keep max 1000 logs

  // Auto-scroll to top for new logs if user is near top
  if (scrollOffset < 100) {
    scrollOffset = 0
  }

  // Update max scroll and redraw
  updateCanvasSize()
  drawCanvas()
}

// Sound filtering function
const shouldPlaySound = (transfer: any): boolean => {
  if (!selectedFromChain && !selectedToChain) {
    return true
  }

  const sourceId = transfer.source_chain?.universal_chain_id
  const destId = transfer.destination_chain?.universal_chain_id

  if (selectedFromChain && selectedToChain) {
    return sourceId === selectedFromChain && destId === selectedToChain
  }

  if (selectedFromChain) {
    return sourceId === selectedFromChain || destId === selectedFromChain
  }

  if (selectedToChain) {
    return sourceId === selectedToChain || destId === selectedToChain
  }

  return true
}

// Process new transfers
$effect(() => {
  if (transfers.length > processedCount) {
    const newTransfers = transfers.slice(processedCount)
    newTransfers.forEach((transfer: any) => {
      if (shouldPlaySound(transfer)) {
        const value = parseFloat(transfer.base_amount?.toString() || "0") || 1
        const sourceChainId = transfer.source_chain?.universal_chain_id
        const destChainId = transfer.destination_chain?.universal_chain_id
        transactionAudio.playSound(value, sourceChainId, destChainId)
      }

      // Use pre-computed display data from server
      const sourceChain = transfer.sourceDisplayName || "unknown"
      const destChain = transfer.destinationDisplayName || "unknown"
      const hash = formatHash(transfer.packet_hash)
      const sender = transfer.senderDisplay
      const receiver = transfer.receiverDisplay

      addLog(
        "transfer",
        sourceChain,
        destChain,
        hash,
        sender,
        receiver,
        transfer.source_chain?.universal_chain_id,
        transfer.destination_chain?.universal_chain_id,
      )
    })
    processedCount = transfers.length
  }
})

// Reactive updates for filtering
$effect(() => {
  updateCanvasSize()
  drawCanvas()
})

onMount(() => {
  if (canvas && containerElement) {
    ctx = canvas.getContext("2d")!
    updateCanvasSize()

    // Add event listeners
    canvas.addEventListener("wheel", handleWheel, { passive: false })
    canvas.addEventListener("mousedown", handleMouseDown)
    window.addEventListener("mousemove", handleMouseMove)
    window.addEventListener("mouseup", handleMouseUp)

    // Resize observer
    const resizeObserver = new ResizeObserver(() => {
      updateCanvasSize()
      drawCanvas()
    })
    resizeObserver.observe(containerElement)

    // Initial log
    addLog("system", "Terminal", "Ready", "waiting for transfers...")

    return () => {
      canvas.removeEventListener("wheel", handleWheel)
      canvas.removeEventListener("mousedown", handleMouseDown)
      window.removeEventListener("mousemove", handleMouseMove)
      window.removeEventListener("mouseup", handleMouseUp)
      resizeObserver.disconnect()
    }
  }
})
</script>

<Card class="h-full p-2">
  <div
    class="relative w-full h-full"
    bind:this={containerElement}
  >
    <canvas
      bind:this={canvas}
      class="w-full h-full cursor-default"
      style="background: transparent;"
    ></canvas>
  </div>
</Card>
