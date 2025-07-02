<script lang="ts">
import { Option, pipe } from "effect"
import { onDestroy, onMount } from "svelte"
import AssetVolumeChart from "./charts/AssetVolumeChart.svelte"
import ChainFlowChart from "./charts/ChainFlowChart.svelte"
import LatencyChart from "./charts/LatencyChart.svelte"
import NetworkVisualizer from "./charts/NetworkVisualizer.svelte"
import NodeHealthChart from "./charts/NodeHealthChart.svelte"
import PopularRoutesChart from "./charts/PopularRoutesChart.svelte"
import TerminalLog from "./charts/TerminalLog.svelte"
import TransferStats from "./charts/TransferStats.svelte"
import WalletActivityChart from "./charts/WalletActivityChart.svelte"
import type { ActiveWalletRates, ChartData, EnhancedTransferListItem, TransferRates } from "./types"

const WS_URL = "wss://ws.union.build/ws"
let connectionStatus: "connecting" | "connected" | "disconnected" | "error" = $state("disconnected")
let reconnectAttempts = 0
let maxReconnectAttempts = 5

let ws: WebSocket | null = null
let transfers: EnhancedTransferListItem[] = $state([])
let selectedFromChain: string | null = $state(null)
let selectedToChain: string | null = $state(null)
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null

let transferRates: Option.Option<TransferRates> = $state(Option.none())
let activeWalletRates: Option.Option<ActiveWalletRates> = $state(Option.none())
let chartData: Option.Option<ChartData> = $state(Option.none())

const popularRoutes = $derived(pipe(chartData, Option.map((data) => data.popularRoutes)))
const popularRoutesTimeScale = $derived(
  pipe(chartData, Option.map((data) => data.popularRoutesTimeScale)),
)
const activeSenders = $derived(pipe(chartData, Option.map((data) => data.activeSenders)))
const activeReceivers = $derived(pipe(chartData, Option.map((data) => data.activeReceivers)))
const activeSendersTimeScale = $derived(
  pipe(chartData, Option.map((data) => data.activeSendersTimeScale)),
)
const activeReceiversTimeScale = $derived(
  pipe(chartData, Option.map((data) => data.activeReceiversTimeScale)),
)
const chainFlowData = $derived(pipe(chartData, Option.map((data) => data.chainFlowData)))
const assetVolumeData = $derived(pipe(chartData, Option.map((data) => data.assetVolumeData)))
const latencyData = $derived(pipe(chartData, Option.map((data) => data.latencyData)))
const nodeHealthData = $derived(
  pipe(
    chartData,
    Option.flatMap((data) =>
      data.nodeHealthData ? Option.some(data.nodeHealthData) : Option.none()
    ),
  ),
)

function handleChainSelection(fromChain: string | null, toChain: string | null) {
  selectedFromChain = fromChain
  selectedToChain = toChain
}

function connectWebSocket() {
  if (ws?.readyState === WebSocket.OPEN) {
    return // Already connected
  }

  connectionStatus = "connecting"
  console.log("🔗 Connecting to WebSocket:", WS_URL)

  ws = new WebSocket(WS_URL)

  ws.onopen = () => {
    connectionStatus = "connected"
    reconnectAttempts = 0
    console.log("✅ Connected to real-time transfer stream")
  }

  ws.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data)

      console.log("message", message)

      if (message.type === "transfer" && message.data) {
        transfers = [...transfers, message.data]
      } else if (message.type === "rates" && message.data) {
        console.log("rates", message.data)
        transferRates = Option.some(message.data)
      } else if (message.type === "chartData" && message.data) {
        console.log("chartData", message.data)
        chartData = Option.some(message.data)

        // Handle both basic and enhanced chart data structures
        if (message.data.currentRates && message.data.activeWalletRates) {
          transferRates = Option.some(message.data.currentRates)
          activeWalletRates = Option.some(message.data.activeWalletRates)
        } else if (message.data.currentRates) {
          // Basic chart data structure (legacy compatibility)
          transferRates = Option.some(message.data.currentRates)
          console.log(
            `📊 Updated basic charts: ${message.data.popularRoutes?.length || 0} routes, ${
              message.data.activeSenders?.length || 0
            } senders, ${message.data.activeReceivers?.length || 0} receivers`,
          )
        }
      } else if (message.type === "connected") {
        console.log("🎉 WebSocket handshake complete:", message.message)
      } else if (message.type === "error") {
        console.error("⚠️ Server error:", message.message)
      }
    } catch (error) {
      console.error("❌ Failed to parse WebSocket message:", error)
    }
  }

  ws.onclose = (event) => {
    connectionStatus = "disconnected"
    console.log("❌ WebSocket disconnected:", event.code, event.reason)

    // Auto-reconnect with exponential backoff
    if (reconnectAttempts < maxReconnectAttempts) {
      const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000) // Max 30s
      reconnectAttempts++

      console.log(
        `🔄 Reconnecting in ${delay}ms (attempt ${reconnectAttempts}/${maxReconnectAttempts})`,
      )

      reconnectTimeout = setTimeout(() => {
        connectWebSocket()
      }, delay)
    } else {
      connectionStatus = "error"
      console.error("💥 Max reconnection attempts reached")
    }
  }

  ws.onerror = (error) => {
    connectionStatus = "error"
    console.error("⚠️ WebSocket error:", error)
  }
}

function disconnectWebSocket() {
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout)
    reconnectTimeout = null
  }

  if (ws) {
    ws.close()
    ws = null
  }

  connectionStatus = "disconnected"
}

onMount(() => {
  connectWebSocket()
})

onDestroy(() => {
  disconnectWebSocket()
})
</script>

<div class="grid grid-cols-1 lg:grid-cols-3 gap-4 p-4 h-auto">
  <!-- Network Visualizer - spans 2 columns -->
  <div class="order-1 lg:order-3 lg:col-span-2 min-h-0 h-full">
    <NetworkVisualizer
      {transfers}
      onChainSelection={handleChainSelection}
    />
  </div>

  <!-- Charts - full width row with 50/50 split -->
  <div class="order-3 lg:order-4 lg:col-span-3 min-h-0">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 h-full">
      <PopularRoutesChart
        popularRoutes={popularRoutes}
        popularRoutesTimeScale={popularRoutesTimeScale}
      />
      <WalletActivityChart
        activeSenders={activeSenders}
        activeReceivers={activeReceivers}
        activeSendersTimeScale={activeSendersTimeScale}
        activeReceiversTimeScale={activeReceiversTimeScale}
        activeWalletRates={activeWalletRates}
      />
    </div>
  </div>

  <!-- Stats - spans full width, now includes connection status -->
  <div class="order-2 lg:order-1 lg:col-span-3">
    <TransferStats
      transferRates={transferRates}
      activeWalletRates={activeWalletRates}
      {connectionStatus}
    />
  </div>

  <!-- Terminal Log - left column -->
  <div class="order-4 lg:order-2 lg:col-span-1 min-h-0">
    <TerminalLog
      {transfers}
      {selectedFromChain}
      {selectedToChain}
    />
  </div>

  <!-- Bottom Charts - 2 column row -->
  <div class="order-5 lg:order-5 lg:col-span-3 min-h-0">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 h-full">
      <ChainFlowChart chainFlowData={chainFlowData} />
      <AssetVolumeChart assetVolumeData={assetVolumeData} />
    </div>
  </div>

  <!-- Latency Chart - full width row -->
  <div class="order-6 lg:order-6 lg:col-span-3 min-h-0">
    <LatencyChart latencyData={latencyData} />
  </div>

  <!-- Node Health Chart - full width row -->
  <div class="order-7 lg:order-7 lg:col-span-3 min-h-0">
    <NodeHealthChart nodeHealthData={nodeHealthData} />
  </div>
</div>
