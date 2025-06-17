<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"

interface WalletStats {
  count: number
  address: string
  displayAddress: string
  lastActivity: string
}

interface ActiveWalletRates {
  sendersLastMin: number
  sendersLastHour: number
  sendersLastDay: number
  sendersLast7d: number
  sendersLast14d: number
  sendersLast30d: number

  receiversLastMin: number
  receiversLastHour: number
  receiversLastDay: number
  receiversLast7d: number
  receiversLast14d: number
  receiversLast30d: number

  totalLastMin: number
  totalLastHour: number
  totalLastDay: number
  totalLast7d: number
  totalLast14d: number
  totalLast30d: number

  uniqueSendersTotal: number
  uniqueReceiversTotal: number
  uniqueTotalWallets: number
}

interface DataAvailability {
  hasMinute: boolean
  hasHour: boolean
  hasDay: boolean
  has7Days: boolean
  has14Days: boolean
  has30Days: boolean
}

interface Props {
  activeSenders?: WalletStats[]
  activeReceivers?: WalletStats[]
  activeSendersTimeScale?: Record<string, WalletStats[]>
  activeReceiversTimeScale?: Record<string, WalletStats[]>
  activeWalletRates?: ActiveWalletRates
  dataAvailability?: DataAvailability
}

const DEFAULT_WALLET_RATES: ActiveWalletRates = {
  sendersLastMin: 0,
  sendersLastHour: 0,
  sendersLastDay: 0,
  sendersLast7d: 0,
  sendersLast14d: 0,
  sendersLast30d: 0,
  receiversLastMin: 0,
  receiversLastHour: 0,
  receiversLastDay: 0,
  receiversLast7d: 0,
  receiversLast14d: 0,
  receiversLast30d: 0,
  totalLastMin: 0,
  totalLastHour: 0,
  totalLastDay: 0,
  totalLast7d: 0,
  totalLast14d: 0,
  totalLast30d: 0,
  uniqueSendersTotal: 0,
  uniqueReceiversTotal: 0,
  uniqueTotalWallets: 0,
}

const DEFAULT_DATA_AVAILABILITY: DataAvailability = {
  hasMinute: false,
  hasHour: false,
  hasDay: false,
  has7Days: false,
  has14Days: false,
  has30Days: false,
}

let {
  activeSenders = [],
  activeReceivers = [],
  activeSendersTimeScale = {},
  activeReceiversTimeScale = {},
  activeWalletRates = DEFAULT_WALLET_RATES,
  dataAvailability = DEFAULT_DATA_AVAILABILITY,
}: Props = $props()

// Local item count configuration
const itemCounts = [
  { value: 3, label: "3" },
  { value: 5, label: "5" },
  { value: 7, label: "7" },
  { value: 10, label: "10" },
]

// State management
let selectedTimeFrame = $state("1m")
let selectedItemCount = $state(5) // Default to 5 items

// Time frame configuration
const timeFrames = [
  { key: "1m", label: "1m", field: "LastMin", desc: "last minute" },
  { key: "1h", label: "1h", field: "LastHour", desc: "last hour" },
  { key: "1d", label: "1d", field: "LastDay", desc: "last day" },
  { key: "7d", label: "7d", field: "Last7d", desc: "last 7 days" },
  { key: "14d", label: "14d", field: "Last14d", desc: "last 14 days" },
  { key: "30d", label: "30d", field: "Last30d", desc: "last 30 days" },
] as const

// Derived state
const currentSenders = $derived.by(() => {
  let data = []
  if (
    activeSendersTimeScale && activeSendersTimeScale[selectedTimeFrame]
    && activeSendersTimeScale[selectedTimeFrame].length > 0
  ) {
    data = activeSendersTimeScale[selectedTimeFrame]
  } else {
    data = activeSenders
  }

  return data?.slice(0, selectedItemCount) || []
})

const currentReceivers = $derived.by(() => {
  let data = []
  if (
    activeReceiversTimeScale && activeReceiversTimeScale[selectedTimeFrame]
    && activeReceiversTimeScale[selectedTimeFrame].length > 0
  ) {
    data = activeReceiversTimeScale[selectedTimeFrame]
  } else {
    data = activeReceivers
  }

  return data?.slice(0, selectedItemCount) || []
})

const chartData = $derived({
  activeSenders: currentSenders,
  activeReceivers: currentReceivers,
  activeWalletRates,
})

const hasData = $derived(currentSenders.length > 0 || currentReceivers.length > 0)
const isLoading = $derived(!hasData && activeSenders.length === 0 && activeReceivers.length === 0)

// Get total transfer count for percentage calculation
const totalTransfersForTimeframe = $derived(() => {
  const senderSum = currentSenders.reduce((sum, sender) => sum + sender.count, 0)
  const receiverSum = currentReceivers.reduce((sum, receiver) => sum + receiver.count, 0)
  return Math.max(senderSum, receiverSum, 1)
})

// Get max count for progress bar visual scaling
const maxSenderCount = $derived(
  chartData.activeSenders.length > 0
    ? Math.max(...chartData.activeSenders.map(sender => sender.count))
    : 1,
)

const maxReceiverCount = $derived(
  chartData.activeReceivers.length > 0
    ? Math.max(...chartData.activeReceivers.map(receiver => receiver.count))
    : 1,
)

// Utility functions
function formatAddress(address: string): string {
  if (address.length <= 16) {
    return address
  }
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

function formatCount(count: number): string {
  if (count === 0) {
    return "0"
  }
  if (count >= 1000000) {
    return `${(count / 1000000).toFixed(1)}M`
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}K`
  }
  return count.toString()
}

function getWalletCounts() {
  const timeFrame = timeFrames.find(tf => tf.key === selectedTimeFrame)
  if (!timeFrame || !chartData.activeWalletRates) {
    return { senders: 0, receivers: 0, total: 0 }
  }

  const field = timeFrame.field
  return {
    senders: chartData.activeWalletRates[`senders${field}` as keyof ActiveWalletRates] as number
      || 0,
    receivers: chartData.activeWalletRates[`receivers${field}` as keyof ActiveWalletRates] as number
      || 0,
    total: chartData.activeWalletRates[`total${field}` as keyof ActiveWalletRates] as number || 0,
  }
}

function isTimeFrameAvailable(timeFrameKey: string): boolean {
  const availabilityMap: Record<string, keyof DataAvailability> = {
    "1m": "hasMinute",
    "1h": "hasHour",
    "1d": "hasDay",
    "7d": "has7Days",
    "14d": "has14Days",
    "30d": "has30Days",
  }

  return dataAvailability[availabilityMap[timeFrameKey]] || false
}

function getFirstAvailableTimeFrame(): string {
  for (const timeFrame of timeFrames) {
    if (isTimeFrameAvailable(timeFrame.key)) {
      return timeFrame.key
    }
  }
  return "1m"
}

function getPercentageOfTotal(count: number): number {
  return Math.round((count / totalTransfersForTimeframe()) * 100)
}

// Auto-update selected timeframe when data becomes available
$effect(() => {
  const firstAvailable = getFirstAvailableTimeFrame()
  if (!isTimeFrameAvailable(selectedTimeFrame)) {
    selectedTimeFrame = firstAvailable
  }
})

// Debug logging in development
$effect(() => {
  if (import.meta.env.DEV) {
    console.log("WalletActivityChart data:", {
      hasData,
      isLoading,
      currentSendersLength: currentSenders.length,
      currentReceiversLength: currentReceivers.length,
      activeSendersLength: activeSenders?.length || 0,
      activeReceiversLength: activeReceivers?.length || 0,
      selectedItemCount: selectedItemCount,
    })
  }
})

const walletCounts = $derived(getWalletCounts())
const selectedTimeFrameInfo = $derived(timeFrames.find(tf => tf.key === selectedTimeFrame))
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <header class="flex items-center justify-between p-2 border-b border-zinc-800">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">active-wallets</h3>
        {#if selectedTimeFrameInfo}
          <span class="text-zinc-600 text-xs">--tf={selectedTimeFrameInfo.key}</span>
        {/if}
      </div>
      <div class="text-xs text-zinc-500">
        {#if isLoading}
          loading...
        {:else if !hasData}
          no data yet
        {/if}
      </div>
    </header>

    <!-- Controls -->
    <div class="pt-2 px-2">
      <!-- Mobile: Stack vertically, Desktop: Side by side -->
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-1 mb-1">
        <!-- Time Frame Selector -->
        <div class="flex flex-wrap gap-0.5">
          {#each timeFrames as timeFrame}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedTimeFrame === timeFrame.key
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : isTimeFrameAvailable(timeFrame.key)
                ? 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                : 'border-zinc-800 bg-zinc-950 text-zinc-600 cursor-not-allowed'
                }
              "
              disabled={!isTimeFrameAvailable(timeFrame.key)}
              onclick={() => selectedTimeFrame = timeFrame.key}
            >
              {timeFrame.label}
            </button>
          {/each}
        </div>

        <!-- Item Count Selector -->
        <div class="flex items-center gap-0.5">
          <span class="text-zinc-600 text-xs font-mono">show:</span>
          {#each itemCounts as itemCount}
            <button
              class="
                px-2 py-1 text-xs font-mono border transition-colors min-h-[32px] {
                selectedItemCount === itemCount.value
                ? 'border-zinc-500 bg-zinc-800 text-zinc-200 font-medium'
                : 'border-zinc-700 bg-zinc-900 text-zinc-400 hover:border-zinc-600 hover:text-zinc-300'
                }
              "
              onclick={() => selectedItemCount = itemCount.value}
            >
              {itemCount.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col p-2">
      {#if isLoading}
        <!-- Loading State -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-2 flex-1">
          <!-- Top Senders Skeleton -->
          <div>
            <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
              top_senders:
            </div>
            <div class="space-y-0.5">
              {#each Array(selectedItemCount) as _, index}
                <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1">
                      <Skeleton class="w-2 h-2" />
                      <Skeleton class="w-16 h-2" />
                    </div>
                    <Skeleton class="w-8 h-2" />
                  </div>
                  <div class="flex items-center space-x-2">
                    <Skeleton class="flex-1 h-1" />
                    <Skeleton class="w-6 h-2" />
                  </div>
                </div>
              {/each}
            </div>
          </div>

          <!-- Top Receivers Skeleton -->
          <div>
            <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
              top_receivers:
            </div>
            <div class="space-y-0.5">
              {#each Array(selectedItemCount) as _, index}
                <div class="p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1">
                      <Skeleton class="w-2 h-2" />
                      <Skeleton class="w-16 h-2" />
                    </div>
                    <Skeleton class="w-8 h-2" />
                  </div>
                  <div class="flex items-center space-x-2">
                    <Skeleton class="flex-1 h-1" />
                    <Skeleton class="w-6 h-2" />
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {:else if !hasData}
        <!-- No Data State -->
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <div class="text-zinc-600 font-mono">no_data</div>
          </div>
        </div>
      {:else}
        <!-- Wallet Data -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-2 flex-1">
          <!-- Top Senders -->
          <section>
            <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
              top_senders:
            </div>
            <div class="space-y-1 overflow-y-auto">
              {#each chartData.activeSenders.slice(0, selectedItemCount) as sender, index}
                <article class="p-2 sm:p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                  <!-- Sender Header -->
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1 text-xs">
                      <span class="text-zinc-500">#{index + 1}</span>
                      <span
                        class="text-zinc-300 font-medium"
                        title={sender.address}
                      >
                        {formatAddress(sender.displayAddress || sender.address)}
                      </span>
                    </div>
                    <span class="text-zinc-100 text-xs tabular-nums font-medium">
                      {formatCount(sender.count)}
                    </span>
                  </div>

                  <!-- Progress Bar -->
                  <div class="flex items-center space-x-2">
                    <div class="flex-1 flex w-full h-1.5 sm:h-1">
                      <div
                        class="bg-zinc-300 h-full transition-all duration-300"
                        style="width: {(sender.count / totalTransfersForTimeframe()) * 100}%"
                        title="Count: {sender.count}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (sender.count / totalTransfersForTimeframe()) * 100}%"
                      >
                      </div>
                    </div>
                    <span class="text-zinc-500 text-xs tabular-nums">
                      {getPercentageOfTotal(sender.count)}%
                    </span>
                  </div>
                </article>
              {/each}

              {#if chartData.activeSenders.length === 0}
                <div class="text-center py-2">
                  <div class="text-zinc-600 text-xs font-mono">no_data</div>
                </div>
              {/if}
            </div>
          </section>

          <!-- Top Receivers -->
          <section>
            <div class="text-xs text-zinc-500 font-mono font-medium mb-1">
              top_receivers:
            </div>
            <div class="space-y-1 overflow-y-auto">
              {#each chartData.activeReceivers.slice(0, selectedItemCount) as receiver, index}
                <article class="p-2 sm:p-1.5 bg-zinc-900 border border-zinc-800 rounded">
                  <!-- Receiver Header -->
                  <div class="flex items-center justify-between mb-0.5">
                    <div class="flex items-center space-x-1 text-xs">
                      <span class="text-zinc-500">#{index + 1}</span>
                      <span
                        class="text-zinc-300 font-medium"
                        title={receiver.address}
                      >
                        {
                          formatAddress(
                            receiver.displayAddress || receiver.address,
                          )
                        }
                      </span>
                    </div>
                    <span class="text-zinc-100 text-xs tabular-nums font-medium">
                      {formatCount(receiver.count)}
                    </span>
                  </div>

                  <!-- Progress Bar -->
                  <div class="flex items-center space-x-2">
                    <div class="flex-1 flex w-full h-1.5 sm:h-1">
                      <div
                        class="bg-zinc-300 h-full transition-all duration-300"
                        style="width: {(receiver.count / totalTransfersForTimeframe()) * 100}%"
                        title="Count: {receiver.count}"
                      >
                      </div>
                      <div
                        class="bg-zinc-800 h-full transition-all duration-300"
                        style="width: {100 - (receiver.count / totalTransfersForTimeframe()) * 100}%"
                      >
                      </div>
                    </div>
                    <span class="text-zinc-500 text-xs tabular-nums">
                      {getPercentageOfTotal(receiver.count)}%
                    </span>
                  </div>
                </article>
              {/each}

              {#if chartData.activeReceivers.length === 0}
                <div class="text-center py-2">
                  <div class="text-zinc-600 text-xs font-mono">no_data</div>
                </div>
              {/if}
            </div>
          </section>
        </div>
      {/if}
    </main>
  </div>
</Card>

<style>
/* Custom scrollbar styling */
.overflow-y-auto::-webkit-scrollbar {
  width: 4px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: #27272a;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: #52525b;
  border-radius: 2px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: #71717a;
}
</style>
