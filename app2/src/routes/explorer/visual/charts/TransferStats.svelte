<script lang="ts">
// TransferStats with data availability and wallet info
import Card from "$lib/components/ui/Card.svelte"
import { transactionAudio } from "../audio"

interface TransferRates {
  txPerMinute: number
  txPerHour: number
  txPerDay: number
  txPer7Days: number
  txPer14Days: number
  txPer30Days: number
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
}

interface ActiveWalletRates {
  sendersLastMin: number
  sendersLastHour: number
  sendersLastDay: number
  sendersLast7d: number
  sendersLast14d: number
  sendersLast30d: number
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
  totalLastMinChange?: number
  totalLastHourChange?: number
  totalLastDayChange?: number
  totalLast7dChange?: number
  totalLast14dChange?: number
  totalLast30dChange?: number
  uniqueSendersTotal: number
  uniqueReceiversTotal: number
  uniqueTotalWallets: number
  dataAvailability: {
    hasMinute: boolean
    hasHour: boolean
    hasDay: boolean
    has7Days: boolean
    has14Days: boolean
    has30Days: boolean
  }
  serverUptimeSeconds: number
}

interface Props {
  transferRates?: TransferRates | null
  activeWalletRates?: ActiveWalletRates | null
  dataAvailability?: {
    hasMinute: boolean
    hasHour: boolean
    hasDay: boolean
    has7Days: boolean
    has14Days: boolean
    has30Days: boolean
  }
  connectionStatus?: "connecting" | "connected" | "disconnected" | "error"
}

const DEFAULT_TRANSFER_RATES: TransferRates = {
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
  dataAvailability: {
    hasMinute: false,
    hasHour: false,
    hasDay: false,
    has7Days: false,
    has14Days: false,
    has30Days: false,
  },
  serverUptimeSeconds: 0,
}

let {
  transferRates = null,
  activeWalletRates = null,
  dataAvailability = null,
  connectionStatus = "disconnected",
}: Props = $props()

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
  transferRates || DEFAULT_TRANSFER_RATES,
)

let wallets = $derived(
  activeWalletRates || DEFAULT_WALLET_RATES,
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

// Debug logging in development
$effect(() => {
  if (import.meta.env.DEV) {
    console.log("TransferStats data:", {
      hasTransferRates: !!transferRates,
      hasWalletRates: !!activeWalletRates,
      connectionStatus,
      uptimeSeconds: rates.serverUptimeSeconds,
    })
  }
})

// Get data availability - prefer from activeWalletRates, fallback to prop
let walletDataAvailability = $derived(
  wallets.dataAvailability || dataAvailability || {
    hasMinute: false,
    hasHour: false,
    hasDay: false,
    has7Days: false,
    has14Days: false,
    has30Days: false,
  },
)
</script>

<Card class="h-full p-0">
  <div class="flex flex-col h-full font-mono">
    <!-- Terminal Header -->
    <header class="flex items-center justify-between border-b border-zinc-800 p-2">
      <div class="flex items-center space-x-2">
        <span class="text-zinc-500 text-xs">$</span>
        <h3 class="text-xs text-zinc-300 font-semibold">transfer-stats</h3>
        <span class="text-zinc-600 text-xs">--up={uptimeDisplay()}</span>
      </div>
      <div class="flex items-center space-x-1">
        <!-- Audio Control -->
        <button
          onclick={toggleMute}
          class="flex items-center px-1 py-0.5 rounded hover:bg-zinc-800 transition-colors"
          class:text-zinc-500={!isMuted}
          class:text-red-400={isMuted}
          title={isMuted ? "Enable audio" : "Mute audio"}
        >
          {#if isMuted}
            <!-- Muted icon -->
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M16.5 12C16.5 10.23 15.5 8.71 14 7.97V9.18L16.45 11.63C16.48 11.86 16.5 12.12 16.5 12ZM19 12C19 12.94 18.8 13.82 18.46 14.64L19.97 16.15C20.63 14.91 21 13.5 21 12C21 7.72 18 4.14 14 3.23V5.29C16.89 6.15 19 8.83 19 12ZM4.27 3L3 4.27L7.73 9H3V15H7L12 20V13.27L16.25 17.53C15.58 18.04 14.83 18.45 14 18.7V20.77C15.38 20.45 16.63 19.82 17.68 18.96L19.73 21L21 19.73L12 10.73L4.27 3ZM12 4L9.91 6.09L12 8.18V4Z"
                fill="currentColor"
              />
            </svg>
          {:else}
            <!-- Unmuted icon with animated sound waves -->
            <svg
              class="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M3 9V15H7L12 20V4L7 9H3Z"
                fill="currentColor"
              />
              <path
                d="M16.5 12C16.5 10.23 15.5 8.71 14 7.97V16.02C15.5 15.29 16.5 13.77 16.5 12Z"
                fill="currentColor"
              >
                <animate
                  attributeName="opacity"
                  values="0.4;1;0.4"
                  dur="2s"
                  repeatCount="indefinite"
                />
              </path>
              <path
                d="M19 12C19 8.83 16.89 6.15 14 5.29V7.36C15.84 8.16 17 9.96 17 12S15.84 15.84 14 16.64V18.71C16.89 17.85 19 15.17 19 12Z"
                fill="currentColor"
              >
                <animate
                  attributeName="opacity"
                  values="0.2;0.8;0.2"
                  dur="2s"
                  begin="0.3s"
                  repeatCount="indefinite"
                />
              </path>
              <path
                d="M21 12C21 7.72 18 4.14 14 3.23V5.29C16.89 6.15 19 8.83 19 12S16.89 17.85 14 18.71V20.77C18 19.86 21 16.28 21 12Z"
                fill="currentColor"
              >
                <animate
                  attributeName="opacity"
                  values="0.1;0.6;0.1"
                  dur="2s"
                  begin="0.6s"
                  repeatCount="indefinite"
                />
              </path>
            </svg>
          {/if}
        </button>
      </div>
    </header>

    <!-- Transfer Rates Grid -->
    <main class="flex-1 p-2">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4 text-xs">
        <!-- Transfers Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">transfers:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if rates.dataAvailability.hasMinute}
                  <span
                    class="text-[11px] sm:text-[10px] mr-1 {rates.txPerMinuteChange && rates.txPerMinuteChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(rates.txPerMinuteChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[11px] sm:text-[10px]">
                  {
                    rates.dataAvailability.hasMinute
                    ? rates.txPerMinute
                    : "--"
                  }
                </span>
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
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {rates.dataAvailability.hasHour ? rates.txPerHour : "--"}
                </span>
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
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {rates.dataAvailability.hasDay ? rates.txPerDay : "--"}
                </span>
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
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {rates.dataAvailability.has7Days ? rates.txPer7Days : "--"}
                </span>
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
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    rates.dataAvailability.has30Days
                    ? rates.txPer30Days
                    : "--"
                  }
                </span>
              </div>
            </div>
          </div>
        </section>

        <!-- Senders Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">senders:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastMinChange && wallets.sendersLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasMinute
                    ? wallets.sendersLastMin
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastHourChange && wallets.sendersLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasHour
                    ? wallets.sendersLastHour
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLastDayChange && wallets.sendersLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasDay
                    ? wallets.sendersLastDay
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLast7dChange && wallets.sendersLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has7Days
                    ? wallets.sendersLast7d
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.sendersLast30dChange && wallets.sendersLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.sendersLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has30Days
                    ? wallets.sendersLast30d
                    : "--"
                  }
                </span>
              </div>
            </div>
          </div>
        </section>

        <!-- Receivers Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">receivers:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastMinChange && wallets.receiversLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasMinute
                    ? wallets.receiversLastMin
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastHourChange && wallets.receiversLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasHour
                    ? wallets.receiversLastHour
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLastDayChange && wallets.receiversLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasDay
                    ? wallets.receiversLastDay
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLast7dChange && wallets.receiversLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has7Days
                    ? wallets.receiversLast7d
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.receiversLast30dChange && wallets.receiversLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.receiversLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has30Days
                    ? wallets.receiversLast30d
                    : "--"
                  }
                </span>
              </div>
            </div>
          </div>
        </section>

        <!-- Total Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">total:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1m:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasMinute}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastMinChange && wallets.totalLastMinChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastMinChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasMinute
                    ? wallets.totalLastMin
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasHour}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastHourChange && wallets.totalLastHourChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastHourChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasHour
                    ? wallets.totalLastHour
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <div class="text-right">
                {#if walletDataAvailability.hasDay}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLastDayChange && wallets.totalLastDayChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLastDayChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.hasDay
                    ? wallets.totalLastDay
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has7Days}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLast7dChange && wallets.totalLast7dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLast7dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has7Days
                    ? wallets.totalLast7d
                    : "--"
                  }
                </span>
              </div>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <div class="text-right">
                {#if walletDataAvailability.has30Days}
                  <span
                    class="text-[10px] mr-1 {wallets.totalLast30dChange && wallets.totalLast30dChange >= 0 ? 'text-green-400' : 'text-red-400'}"
                  >{formatPercentageChange(wallets.totalLast30dChange)}</span>
                {/if}
                <span class="text-zinc-100 tabular-nums text-[10px]">
                  {
                    walletDataAvailability.has30Days
                    ? wallets.totalLast30d
                    : "--"
                  }
                </span>
              </div>
            </div>
          </div>
        </section>
      </div>
    </main>

    <!-- Info Footer -->
    <footer class="p-2 border-t border-zinc-800">
      <div class="text-[9px] text-zinc-400 font-mono leading-relaxed">
        <span class="text-zinc-300">info:</span> rolling timeframes show activity within each period
        <span class="text-zinc-300">%:</span> change vs previous period
        <span class="text-green-400">(+)</span> increase
        <span class="text-red-400">(-)</span> decrease
        <span class="text-zinc-300">--:</span> insufficient data for timeframe
      </div>
    </footer>
  </div>
</Card>
