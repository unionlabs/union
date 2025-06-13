<script lang="ts">
// TransferStats with data availability and wallet info
import { transactionAudio } from "../../routes/test/audio"
import Card from "./ui/Card.svelte"

let { 
  transferRates = null,
  activeWalletRates = null,
  connectionStatus = 'disconnected'
}: { 
  transferRates?: {
    txPerMinute: number
    txPerHour: number
    txPerDay: number
    txPer7Days: number
    txPer14Days: number
    txPer30Days: number
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
  connectionStatus?: 'connecting' | 'connected' | 'disconnected' | 'error'
} = $props()

// Default empty state
let rates = $derived(transferRates || {
  txPerMinute: 0, txPerHour: 0, txPerDay: 0, txPer7Days: 0,
  txPer14Days: 0, txPer30Days: 0, totalTracked: 0,
  dataAvailability: { hasMinute: false, hasHour: false, hasDay: false, has7Days: false, has14Days: false, has30Days: false },
  serverUptimeSeconds: 0
})

let wallets = $derived(activeWalletRates || {
  sendersLastMin: 0, sendersLastHour: 0, sendersLastDay: 0, sendersLast7d: 0,
  sendersLast14d: 0, sendersLast30d: 0,
  receiversLastMin: 0, receiversLastHour: 0, receiversLastDay: 0, receiversLast7d: 0,
  receiversLast14d: 0, receiversLast30d: 0,
  totalLastMin: 0, totalLastHour: 0, totalLastDay: 0, totalLast7d: 0,
  totalLast14d: 0, totalLast30d: 0,
  uniqueSendersTotal: 0, uniqueReceiversTotal: 0,
  uniqueTotalWallets: 0,
  dataAvailability: { hasMinute: false, hasHour: false, hasDay: false, has7Days: false, has14Days: false, has30Days: false },
  serverUptimeSeconds: 0
})

// Format uptime for display
let uptimeDisplay = $derived(() => {
  const seconds = rates.serverUptimeSeconds
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`
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

<Card>
  <div class="font-mono text-xs space-y-2">
    <!-- Status Line -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div class="w-1.5 h-1.5 rounded-full {
          connectionStatus === 'connected' ? 'bg-green-500' :
          connectionStatus === 'connecting' ? 'bg-yellow-500 animate-pulse' :
          connectionStatus === 'error' ? 'bg-red-500' :
          'bg-zinc-600'
        }"></div>
        <span class="text-zinc-400">
          {connectionStatus === 'connected' ? 'LIVE' :
           connectionStatus === 'connecting' ? 'CONN' :
           connectionStatus === 'error' ? 'ERR' :
           'OFF'}
        </span>
        <span class="text-zinc-600">|</span>
        <span class="text-zinc-500">up:{uptimeDisplay()}</span>
        <span class="text-zinc-600">|</span>
        <span class="text-zinc-500">total:{rates.totalTracked}</span>
      </div>
      
      <!-- Audio Control -->
      <button
        onclick={toggleMute}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded hover:bg-zinc-800 transition-colors"
        class:text-zinc-500={!isMuted}
        class:text-red-400={isMuted}
        title={isMuted ? "Enable audio" : "Mute audio"}
      >
        <span>{isMuted ? '🔇' : '🔊'}</span>
      </button>
    </div>

    <!-- Transfer Rates -->
    <div class="grid grid-cols-4 gap-4 text-zinc-300">
      <div class="space-y-0.5">
        <div class="text-zinc-500">TRANSFERS</div>
        <div class="space-y-0.5">
          <div class="flex justify-between">
            <span class="text-zinc-400">1m:</span>
            <span class="text-zinc-200">{rates.dataAvailability.hasMinute ? rates.txPerMinute : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1h:</span>
            <span class="text-zinc-200">{rates.dataAvailability.hasHour ? rates.txPerHour : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1d:</span>
            <span class="text-zinc-200">{rates.dataAvailability.hasDay ? rates.txPerDay : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">7d:</span>
            <span class="text-zinc-200">{rates.dataAvailability.has7Days ? rates.txPer7Days : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">14d:</span>
            <span class="text-zinc-200">{rates.dataAvailability.has14Days ? rates.txPer14Days : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">30d:</span>
            <span class="text-zinc-200">{rates.dataAvailability.has30Days ? rates.txPer30Days : '--'}</span>
          </div>
        </div>
      </div>

      <div class="space-y-0.5">
        <div class="text-zinc-500">SENDERS</div>
        <div class="space-y-0.5">
          <div class="flex justify-between">
            <span class="text-zinc-400">1m:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasMinute ? wallets.sendersLastMin : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1h:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasHour ? wallets.sendersLastHour : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasDay ? wallets.sendersLastDay : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">7d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has7Days ? wallets.sendersLast7d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">14d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has14Days ? wallets.sendersLast14d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">30d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has30Days ? wallets.sendersLast30d : '--'}</span>
          </div>
        </div>
      </div>

      <div class="space-y-0.5">
        <div class="text-zinc-500">RECEIVERS</div>
        <div class="space-y-0.5">
          <div class="flex justify-between">
            <span class="text-zinc-400">1m:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasMinute ? wallets.receiversLastMin : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1h:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasHour ? wallets.receiversLastHour : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasDay ? wallets.receiversLastDay : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">7d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has7Days ? wallets.receiversLast7d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">14d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has14Days ? wallets.receiversLast14d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">30d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has30Days ? wallets.receiversLast30d : '--'}</span>
          </div>
        </div>
      </div>

      <div class="space-y-0.5">
        <div class="text-zinc-500">TOTAL</div>
        <div class="space-y-0.5">
          <div class="flex justify-between">
            <span class="text-zinc-400">1m:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasMinute ? wallets.totalLastMin : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1h:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasHour ? wallets.totalLastHour : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">1d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.hasDay ? wallets.totalLastDay : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">7d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has7Days ? wallets.totalLast7d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">14d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has14Days ? wallets.totalLast14d : '--'}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-zinc-400">30d:</span>
            <span class="text-zinc-200">{wallets.dataAvailability.has30Days ? wallets.totalLast30d : '--'}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Summary Line -->
    <div class="flex items-center justify-between pt-1 border-t border-zinc-800">
      <div class="flex items-center gap-4 text-zinc-500">
        <span>unique: {wallets.uniqueSendersTotal}s/{wallets.uniqueReceiversTotal}r/{wallets.uniqueTotalWallets}t</span>
        {#if rates.dataAvailability.has7Days}
          <span>7d: {wallets.sendersLast7d}s/{wallets.receiversLast7d}r/{wallets.totalLast7d}t</span>
        {/if}
        {#if rates.dataAvailability.has30Days}
          <span>30d: {wallets.sendersLast30d}s/{wallets.receiversLast30d}r/{wallets.totalLast30d}t</span>
        {/if}
      </div>
      <div class="text-zinc-600">
        {rates.dataAvailability.has30Days ? 'FULL' : 
         rates.dataAvailability.hasDay ? 'PARTIAL' : 
         'COLLECTING'}
      </div>
    </div>
  </div>
</Card>
