<script lang="ts">
// TransferStats with data availability and wallet info
import { transactionAudio } from "../../routes/test/audio"
import Card from "./ui/Card.svelte"

let {
  transferRates = null,
  activeWalletRates = null,
  dataAvailability = {
    hasMinute: false,
    hasHour: false,
    hasDay: false,
    has7Days: false,
    has14Days: false,
    has30Days: false,
  },
  connectionStatus = "disconnected",
}: {
  transferRates?: {
    txPerMinute: number
    txPerHour: number
    txPerDay: number
    txPer7Days: number
    txPer14Days: number
    txPer30Days: number
    // Percentage changes from previous periods
    txPerMinuteChange?: number
    txPerHourChange?: number
    txPerDayChange?: number
    txPer7DaysChange?: number
    txPer14DaysChange?: number
    txPer30DaysChange?: number
    totalTracked: number
    dataAvailability: {
      hasMinute: boolean
      hasHour: boolean
      hasDay: boolean
      has7Days: boolean
      has14Days: boolean
      has30Days: boolean
    }
    serverUptimeSeconds: number
  } | null
  activeWalletRates?: {
    sendersLastMin: number
    sendersLastHour: number
    sendersLastDay: number
    sendersLast7d: number
    sendersLast14d: number
    sendersLast30d: number
    // Percentage changes for senders
    sendersLastMinChange?: number
    sendersLastHourChange?: number
    sendersLastDayChange?: number
    sendersLast7dChange?: number
    sendersLast14dChange?: number
    sendersLast30dChange?: number
    receiversLastMin: number
    receiversLastHour: number
    receiversLastDay: number
    receiversLast7d: number
    receiversLast14d: number
    receiversLast30d: number
    // Percentage changes for receivers
    receiversLastMinChange?: number
    receiversLastHourChange?: number
    receiversLastDayChange?: number
    receiversLast7dChange?: number
    receiversLast14dChange?: number
    receiversLast30dChange?: number
    totalLastMin: number
    totalLastHour: number
    totalLastDay: number
    totalLast7d: number
    totalLast14d: number
    totalLast30d: number
    // Percentage changes for total active wallets
    totalLastMinChange?: number
    totalLastHourChange?: number
    totalLastDayChange?: number
    totalLast7dChange?: number
    totalLast14dChange?: number
    totalLast30dChange?: number
    uniqueSendersTotal: number
    uniqueReceiversTotal: number
    uniqueTotalWallets: number
    serverUptimeSeconds: number
  } | null
  dataAvailability?: {
    hasMinute: boolean
    hasHour: boolean
    hasDay: boolean
    has7Days: boolean
    has14Days: boolean
    has30Days: boolean
  }
  connectionStatus?: "connecting" | "connected" | "disconnected" | "error"
} = $props()

// Helper function to format percentage changes
function formatPercentageChange(change?: number): string {
  if (change === undefined || change === null || !isFinite(change)) {
    return ""
  }
  const sign = change >= 0 ? "+" : ""
  return `(${sign}${change.toFixed(1)}%)`
}

// Default empty state
let rates = $derived(
  transferRates || {
    txPerMinute: 0,
    txPerHour: 0,
    txPerDay: 0,
    txPer7Days: 0,
    txPer14Days: 0,
    txPer30Days: 0,
    totalTracked: 0,
    dataAvailability: {
      hasMinute: false,
      hasHour: false,
      hasDay: false,
      has7Days: false,
      has14Days: false,
      has30Days: false,
    },
    serverUptimeSeconds: 0,
  },
)

let wallets = $derived(
  activeWalletRates || {
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
    serverUptimeSeconds: 0,
  },
)

// Format uptime for display
let uptimeDisplay = $derived(() => {
  const seconds = rates.serverUptimeSeconds
  if (seconds < 60) {
    return `${seconds}s`
  }
  if (seconds < 3600) {
    return `${Math.floor(seconds / 60)}m`
  }
  if (seconds < 86400) {
    return `${Math.floor(seconds / 3600)}h`
  }
  return `${Math.floor(seconds / 86400)}d`
})

let isMuted = $state(!transactionAudio.isEnabled())

const toggleMute = async () => {
  if (isMuted) {
    await transactionAudio.resumeIfNeeded()
    transactionAudio.enable()
    isMuted = false
  } else {
    transactionAudio.disable()
    isMuted = true
  }
}
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <div class="flex items-center justify-between border-b border-zinc-800 p-3">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500">$</span>
        <h3 class="text-xs text-zinc-300">transfer-stats</h3>
        <span class="text-zinc-600 text-xs">--up={uptimeDisplay()}</span>
        <span class="text-zinc-600 text-xs">--total={rates.totalTracked}</span>
      </div>
      <div class="flex items-center space-x-1">
        {#if connectionStatus === "connected"}
          <span class="text-green-500 text-xs">●</span>
        {:else if connectionStatus === "connecting"}
          <span class="text-yellow-500 text-xs animate-pulse">●</span>
        {:else if connectionStatus === "error"}
          <span class="text-red-500 text-xs">●</span>
        {:else}
          <span class="text-zinc-600 text-xs">●</span>
        {/if}
        <span class="text-xs text-zinc-500">
          {
            connectionStatus === "connected"
            ? "live"
            : connectionStatus === "connecting"
            ? "conn"
            : connectionStatus === "error"
            ? "err"
            : "off"
          }
        </span>

        <!-- Audio Control -->
        <button
          onclick={toggleMute}
          class="flex items-center px-1 py-0.5 rounded hover:bg-zinc-800 transition-colors"
          class:text-zinc-500={!isMuted}
          class:text-red-400={isMuted}
          title={isMuted ? "Enable audio" : "Mute audio"}
        >
          <span class="text-xs">{isMuted ? "🔇" : "🔊"}</span>
        </button>
      </div>
    </div>

    <!-- Transfer Rates - Terminal Style -->
    <div class="flex-1">
      <div class="grid grid-cols-4 gap-4 text-xs p-3">
        <div class="space-y-1">
          <div class="text-zinc-500 font-mono text-xs">transfers:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if rates.dataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {rates.txPerMinuteChange && rates.txPerMinuteChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPerMinuteChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  rates.dataAvailability.hasMinute ? rates.txPerMinute : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if rates.dataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {rates.txPerHourChange && rates.txPerHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPerHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  rates.dataAvailability.hasHour ? rates.txPerHour : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if rates.dataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {rates.txPerDayChange && rates.txPerDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPerDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  rates.dataAvailability.hasDay ? rates.txPerDay : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if rates.dataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {rates.txPer7DaysChange && rates.txPer7DaysChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPer7DaysChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  rates.dataAvailability.has7Days ? rates.txPer7Days : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if rates.dataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {rates.txPer30DaysChange && rates.txPer30DaysChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPer30DaysChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  rates.dataAvailability.has30Days ? rates.txPer30Days : "--"
                }</span>
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-1">
          <div class="text-zinc-500 font-mono text-xs">senders:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if dataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastMinChange && wallets.sendersLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasMinute ? wallets.sendersLastMin : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if dataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastHourChange && wallets.sendersLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasHour ? wallets.sendersLastHour : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if dataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastDayChange && wallets.sendersLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasDay ? wallets.sendersLastDay : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if dataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLast7dChange && wallets.sendersLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has7Days ? wallets.sendersLast7d : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if dataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLast30dChange && wallets.sendersLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has30Days ? wallets.sendersLast30d : "--"
                }</span>
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-1">
          <div class="text-zinc-500 font-mono text-xs">receivers:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if dataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastMinChange && wallets.receiversLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasMinute ? wallets.receiversLastMin : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if dataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastHourChange && wallets.receiversLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasHour ? wallets.receiversLastHour : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if dataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastDayChange && wallets.receiversLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasDay ? wallets.receiversLastDay : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if dataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLast7dChange && wallets.receiversLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has7Days ? wallets.receiversLast7d : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if dataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLast30dChange && wallets.receiversLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has30Days ? wallets.receiversLast30d : "--"
                }</span>
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-1">
          <div class="text-zinc-500 font-mono text-xs">total:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if dataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastMinChange && wallets.totalLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasMinute ? wallets.totalLastMin : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if dataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastHourChange && wallets.totalLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasHour ? wallets.totalLastHour : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if dataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastDayChange && wallets.totalLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.hasDay ? wallets.totalLastDay : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if dataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLast7dChange && wallets.totalLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has7Days ? wallets.totalLast7d : "--"
                }</span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if dataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLast30dChange && wallets.totalLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">{
                  dataAvailability.has30Days ? wallets.totalLast30d : "--"
                }</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Explanatory text - Full width in its own row -->
    <div class="p-3 border-t border-zinc-800 w-full col-span-full flex-1">
      <div class="text-[9px] text-zinc-400 font-mono leading-relaxed">
        <span class="text-zinc-300">info:</span> rolling timeframes show activity within each period
        <span class="text-zinc-300">%:</span> change vs previous period
        <span class="text-green-400">(+)</span> increase
        <span class="text-red-400">(-)</span> decrease
        <span class="text-zinc-300">--:</span> insufficient data for timeframe
      </div>
    </div>
  </div>
</Card>
