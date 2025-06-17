<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import { chains } from "$lib/stores/chains.svelte"
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import { onMount } from "svelte"
import { transactionAudio } from "../audio"
import { initializeCanvasWithCleanup } from "../canvasInit"

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

let {
  transfers = [],
  selectedFromChain = null,
  selectedToChain = null,
}: {
  transfers: EnhancedTransferListItem[]
  selectedFromChain?: string | null
  selectedToChain?: string | null
} = $props()

// State variables
let allLogs: LogEntry[] = $state([])
let canvas: HTMLCanvasElement
let ctx: CanvasRenderingContext2D
let containerElement: HTMLElement
let processedCount = $state(0)
let logIdCounter = $state(0)
let canvasReady = $state(false)
let canvasWidth = $state(400)
let canvasHeight = $state(600)
let scrollOffset = $state(0)
let maxScrollOffset = $state(0)
let isMouseDown = false
let lastMouseY = 0

// Reactive derived values
let transfersLength = $derived(transfers.length)
let filteredLogs = $derived(allLogs.slice(0, 1000))
let hasFilter = $derived(selectedFromChain || selectedToChain)

// Audio configuration

// Visual constants
const LOG_HEIGHT = 48
const PADDING = 8
const LINE_HEIGHT = 12
const FONT_SIZE = 10
const COLORS = {
  background: "#00000000",
  text: "#d4d4d8",
  textSecondary: "#9ca3af",
  filter: "#f59e0b",
  filterBg: "#27272a",
  border: "#3f3f46",
  waiting: "#6b7280",
} as const

// Cache for chain display names
const chainDisplayNameCache = new Map<string, string>()

// Utility functions
const formatHash = (hash: string): string =>
  hash ? `${hash.slice(0, 8)}...${hash.slice(-4)}` : "N/A"

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

const shouldPlaySound = (transfer: EnhancedTransferListItem): boolean => {
  if (!hasFilter) {
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

// Canvas functions
function setupCanvasContext() {
  if (!ctx) {
    return
  }

  const dpr = window.devicePixelRatio || 1
  ctx.scale(dpr, dpr)
  ctx.textBaseline = "top"
  ctx.font =
    `${FONT_SIZE}px ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace`
  ctx.textAlign = "left"
}

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
  canvas.style.width = `${canvasWidth}px`
  canvas.style.height = `${canvasHeight}px`

  if (ctx) {
    setupCanvasContext()
  }

  // Update max scroll
  const filterHeight = hasFilter ? 32 : 0
  const contentHeight = filteredLogs.length * LOG_HEIGHT
  const viewportHeight = canvasHeight - filterHeight
  maxScrollOffset = Math.max(0, contentHeight - viewportHeight)
}

function drawFilterBar() {
  if (!hasFilter) {
    return
  }

  // Background
  ctx.fillStyle = COLORS.filterBg
  ctx.fillRect(0, 0, canvasWidth, 32)

  // Border
  ctx.fillStyle = COLORS.border
  ctx.fillRect(0, 31, canvasWidth, 1)

  // Text
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

  // Reset font
  ctx.font =
    `${FONT_SIZE}px ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace`
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

function drawWaitingMessage() {
  ctx.fillStyle = COLORS.waiting
  ctx.textAlign = "center"
  const centerY = canvasHeight / 2
  ctx.fillText("Waiting for transfers...", canvasWidth / 2, centerY - 10)
  ctx.fillText("_", canvasWidth / 2, centerY + 10)
  ctx.textAlign = "left"
}

function drawCanvas() {
  if (!ctx) {
    return
  }

  // Clear canvas
  ctx.fillStyle = COLORS.background
  ctx.fillRect(0, 0, canvasWidth, canvasHeight)

  const filterHeight = hasFilter ? 32 : 0
  drawFilterBar()

  if (filteredLogs.length === 0) {
    drawWaitingMessage()
    return
  }

  // Virtual scrolling optimization
  const viewportTop = scrollOffset
  const viewportBottom = viewportTop + (canvasHeight - filterHeight)
  const startIndex = Math.max(0, Math.floor(viewportTop / LOG_HEIGHT))
  const endIndex = Math.min(filteredLogs.length, Math.ceil(viewportBottom / LOG_HEIGHT) + 1)

  // Clip to viewport
  ctx.save()
  ctx.beginPath()
  ctx.rect(0, filterHeight, canvasWidth, canvasHeight - filterHeight)
  ctx.clip()

  // Draw visible logs only
  for (let i = startIndex; i < endIndex; i++) {
    const logY = (i * LOG_HEIGHT) - scrollOffset + filterHeight
    if (logY > -LOG_HEIGHT && logY < canvasHeight) {
      drawLogEntry(filteredLogs[i], logY)
    }
  }

  ctx.restore()
}

// Event handlers
const handleWheel = (event: WheelEvent) => {
  event.preventDefault()
  scrollOffset = Math.max(0, Math.min(maxScrollOffset, scrollOffset + event.deltaY))
  drawCanvas()
}

const handleMouseDown = (event: MouseEvent) => {
  isMouseDown = true
  lastMouseY = event.clientY
}

const handleMouseMove = (event: MouseEvent) => {
  if (!isMouseDown) {
    return
  }

  const deltaY = event.clientY - lastMouseY
  scrollOffset = Math.max(0, Math.min(maxScrollOffset, scrollOffset - deltaY))
  lastMouseY = event.clientY
  drawCanvas()
}

const handleMouseUp = () => {
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
  const logEntry: LogEntry = {
    id: ++logIdCounter,
    timestamp: new Date().toLocaleTimeString(),
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

  allLogs = [logEntry, ...allLogs].slice(0, 1000)

  // Auto-scroll to top for new logs if user is near top
  if (scrollOffset < 100) {
    scrollOffset = 0
  }

  if (canvasReady) {
    updateCanvasSize()
    drawCanvas()
  }
}

// Process new transfers
$effect(() => {
  if (canvasReady && transfersLength > processedCount) {
    const newTransfers = transfers.slice(processedCount)

    newTransfers.forEach((transfer) => {
      if (shouldPlaySound(transfer)) {
        requestAnimationFrame(() => {
          transactionAudio.playSound(
            1,
            transfer.source_chain?.universal_chain_id,
            transfer.destination_chain?.universal_chain_id,
          )
        })
      }

      const sourceChain = transfer.sourceDisplayName || transfer.source_chain?.display_name
        || "unknown"
      const destChain = transfer.destinationDisplayName || transfer.destination_chain?.display_name
        || "unknown"

      addLog(
        "transfer",
        sourceChain,
        destChain,
        formatHash(transfer.packet_hash),
        transfer.senderDisplay,
        transfer.receiverDisplay,
        transfer.source_chain?.universal_chain_id,
        transfer.destination_chain?.universal_chain_id,
      )
    })

    processedCount = transfers.length
  }
})

// Reactive canvas updates
$effect(() => {
  if (canvasReady) {
    updateCanvasSize()
    drawCanvas()
  }
})

onMount(() => {
  if (canvas && containerElement) {
    ctx = canvas.getContext("2d")!

    const cleanup = initializeCanvasWithCleanup({
      canvas,
      container: containerElement,
      onInitialized: () => {
        updateCanvasSize()
        canvasReady = true
        drawCanvas()
        addLog("system", "Terminal", "Ready", "waiting for transfers...")
      },
      onResize: () => {
        updateCanvasSize()
        drawCanvas()
      },
      eventListeners: [
        {
          element: canvas,
          event: "wheel",
          handler: handleWheel as (event: Event) => void,
          options: { passive: false },
        },
        { element: canvas, event: "mousedown", handler: handleMouseDown as (event: Event) => void },
        { element: window, event: "mousemove", handler: handleMouseMove as (event: Event) => void },
        { element: window, event: "mouseup", handler: handleMouseUp as (event: Event) => void },
      ],
    })

    return cleanup
  }
})
</script>

<Card class="h-full p-2 pointer-events-none">
  <div
    class="relative w-full h-full pointer-events-none"
    bind:this={containerElement}
  >
    <canvas
      bind:this={canvas}
      class="w-full h-full cursor-default pointer-events-none"
      style="background: transparent;"
    />
  </div>
</Card>
