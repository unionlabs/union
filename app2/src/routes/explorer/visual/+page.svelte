<script lang="ts">
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { onDestroy, onMount } from "svelte"
import AssetVolumeChart from "./charts/AssetVolumeChart.svelte"
import ChainFlowChart from "./charts/ChainFlowChart.svelte"
import LatencyChart from "./charts/LatencyChart.svelte"
import NetworkVisualizer from "./charts/NetworkVisualizer.svelte"
import PopularRoutesChart from "./charts/PopularRoutesChart.svelte"
import TerminalLog from "./charts/TerminalLog.svelte"
import TransferStats from "./charts/TransferStats.svelte"
import WalletActivityChart from "./charts/WalletActivityChart.svelte"
import NodeHealthChart from "./charts/NodeHealthChart.svelte";
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

// WebSocket configuration
const WS_URL = "ws://localhost:8080/ws"

let ws: WebSocket | null = null
let transfers: EnhancedTransferListItem[] = $state([])
let selectedFromChain: string | null = $state(null)
let selectedToChain: string | null = $state(null)
let connectionStatus: "connecting" | "connected" | "disconnected" | "error" = $state("disconnected")
let reconnectAttempts = 0
let maxReconnectAttempts = 5
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null

// Transfer rates from backend
let transferRates = $state({
  txPerSecond: 0,
  txPer30Seconds: 0,
  txPerMinute: 0,
  txPerHour: 0,
  txPerDay: 0,
  txPer7Days: 0,
  txPer14Days: 0,
  txPer30Days: 0,
  txPerMinuteChange: 0,
  txPerHourChange: 0,
  txPerDayChange: 0,
  txPer7DaysChange: 0,
  txPer14DaysChange: 0,
  txPer30DaysChange: 0,
  totalTracked: 0,
  serverUptimeSeconds: 0,
})

// Active wallet rates from backend
let activeWalletRates = $state({
  sendersLastMin: 0,
  sendersLastHour: 0,
  sendersLastDay: 0,
  sendersLast7d: 0,
  sendersLast14d: 0,
  sendersLast30d: 0,
  sendersLastMinChange: 0,
  sendersLastHourChange: 0,
  sendersLastDayChange: 0,
  sendersLast7dChange: 0,
  sendersLast14dChange: 0,
  sendersLast30dChange: 0,
  receiversLastMin: 0,
  receiversLastHour: 0,
  receiversLastDay: 0,
  receiversLast7d: 0,
  receiversLast14d: 0,
  receiversLast30d: 0,
  receiversLastMinChange: 0,
  receiversLastHourChange: 0,
  receiversLastDayChange: 0,
  receiversLast7dChange: 0,
  receiversLast14dChange: 0,
  receiversLast30dChange: 0,
  totalLastMin: 0,
  totalLastHour: 0,
  totalLastDay: 0,
  totalLast7d: 0,
  totalLast14d: 0,
  totalLast30d: 0,
  totalLastMinChange: 0,
  totalLastHourChange: 0,
  totalLastDayChange: 0,
  totalLast7dChange: 0,
  totalLast14dChange: 0,
  totalLast30dChange: 0,
  uniqueSendersTotal: 0,
  uniqueReceiversTotal: 0,
  uniqueTotalWallets: 0,
  serverUptimeSeconds: 0,
})

// Chart data from backend
let chartData = $state({
  popularRoutes: [],
  activeSenders: [],
  activeReceivers: [],
  currentRates: null,
  popularRoutesTimeScale: {},
  activeSendersTimeScale: {},
  activeReceiversTimeScale: {},
  chainFlowData: {
    chains: [],
    chainFlowTimeScale: {},
    totalOutgoing: 0,
    totalIncoming: 0,
    serverUptimeSeconds: 0,
  },
  assetVolumeData: {
    assets: [],
    assetVolumeTimeScale: {},
    totalAssets: 0,
    totalVolume: 0,
    totalTransfers: 0,
    serverUptimeSeconds: 0,
  },
  latencyData: [],
  nodeHealthData: null as any, // Can be array (legacy) or processed object (new format)
})

// Chains data from backend
let chainsData = []

// Track if we've received initial data
let hasInitialData = false

const processedNodeHealthData = $derived(chartData.nodeHealthData || {
  totalNodes: 0,
  healthyNodes: 0,
  degradedNodes: 0,
  unhealthyNodes: 0,
  avgResponseTime: 0,
  nodesWithRpcs: [],
  chainHealthStats: {},
})

function handleChainSelection(fromChain: string | null, toChain: string | null) {
  selectedFromChain = fromChain
  selectedToChain = toChain

  // Send chain filter to WebSocket server for server-side filtering
  if (ws?.readyState === WebSocket.OPEN) {
    const filterData = {
      fromChain,
      toChain,
    }

    ws.send(JSON.stringify({
      type: "setChainFilter",
      data: filterData,
    }))

    console.log(`🔍 Set chain filter:`, filterData)
  }
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

    // Restore chain filter if we had one set
    if (selectedFromChain || selectedToChain) {
      setTimeout(() => {
        handleChainSelection(selectedFromChain, selectedToChain)
      }, 100) // Small delay to ensure connection is fully established
    }
  }

  ws.onmessage = (event) => {
    try {
      const message = JSON.parse(event.data)

      console.log("message", message)

      if (message.type === "transfers" && Array.isArray(message.data)) {
        transfers = [...transfers, ...message.data]
        console.log(
          `📦 Received ${message.data.length} new transfers (server-optimized). Total: ${transfers.length}`,
        )
      } else if (message.type === "transfer" && message.data) {
        // Handle individual transfer (singular)
        transfers = [...transfers, message.data]
        console.log(`📦 Received 1 new transfer (server-optimized). Total: ${transfers.length}`)
      } else if (message.type === "rates" && message.data) {
        // Update transfer rates from backend (legacy)
        transferRates = message.data
      } else if (message.type === "chartData" && message.data) {
        // Update chart data from backend
        console.log("chartData", message.data)
        chartData = message.data
        hasInitialData = true

        // Handle both basic and enhanced chart data structures
        if (message.data.currentRates && message.data.activeWalletRates) {
          // Enhanced chart data structure - preserve existing availability if new one is false
          transferRates = {
            ...message.data.currentRates,
          }

          activeWalletRates = {
            ...message.data.activeWalletRates,
          }

          // Update chart data
          chartData = {
            ...chartData,
            ...message.data,
          }

          console.log(
            `📊 Updated enhanced charts: ${message.data.popularRoutes?.length || 0} routes, ${
              message.data.activeSenders?.length || 0
            } senders, ${message.data.activeReceivers?.length || 0} receivers, uptime: ${
              message.data.currentRates?.serverUptimeSeconds || 0
            }s`,
          )
        } else if (message.data.currentRates) {
          // Basic chart data structure (legacy compatibility)
          transferRates = {
            ...transferRates,
            ...message.data.currentRates,
          }
          console.log(
            `📊 Updated basic charts: ${message.data.popularRoutes?.length || 0} routes, ${
              message.data.activeSenders?.length || 0
            } senders, ${message.data.activeReceivers?.length || 0} receivers`,
          )
        }
      } else if (message.type === "chains" && Array.isArray(message.data)) {
        // Update chains data from server
        chainsData = message.data
        console.log(`⛓️  Received chains data: ${chainsData.length} chains`)
      } else if (message.type === "filterSet" && message.data) {
        console.log(
          "✅ Chain filter applied:",
          `${message.data.fromChain || "any"} → ${message.data.toChain || "any"}`,
        )
      } else if (message.type === "serverInfo" && message.data) {
        console.log("🔧 Server optimizations active:", message.data.features)
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
        popularRoutes={chartData.popularRoutes}
        popularRoutesTimeScale={chartData.popularRoutesTimeScale}
      />
      <WalletActivityChart
        activeSenders={chartData.activeSenders}
        activeReceivers={chartData.activeReceivers}
        activeSendersTimeScale={chartData.activeSendersTimeScale}
        activeReceiversTimeScale={chartData.activeReceiversTimeScale}
        activeWalletRates={activeWalletRates}
      />
    </div>
  </div>

  <!-- Stats - spans full width, now includes connection status -->
  <div class="order-2 lg:order-1 lg:col-span-3">
    <TransferStats
      {transferRates}
      {activeWalletRates}
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
      <ChainFlowChart
        chainFlowData={chartData.chainFlowData}
      />
      <AssetVolumeChart
        assetVolumeData={chartData.assetVolumeData}
      />
    </div>
  </div>

  <!-- Latency Chart - full width row -->
  <div class="order-6 lg:order-6 lg:col-span-3 min-h-0">
    <LatencyChart latencyData={chartData.latencyData} />
  </div>
  
  <!-- Node Health Chart - full width row -->
  <div class="order-7 lg:order-7 lg:col-span-3 min-h-0">
    <NodeHealthChart nodeHealthData={processedNodeHealthData} />
  </div>
</div>
