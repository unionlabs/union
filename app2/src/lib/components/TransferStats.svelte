<script lang="ts">
import type { TransferListItem } from "@unionlabs/sdk/schema"
import { transactionAudio } from "../../routes/test/audio"
import Card from "./ui/Card.svelte"

let { transfers = [] }: { transfers: TransferListItem[] } = $props()

let transferTimestamps: number[] = $state([])
let processedCount = $state(0)

// Calculate transfer rates (pure function - no state mutations)
const getTransferRates = () => {
  const now = Date.now()
  const oneSecondAgo = now - 1000
  const thirtySecondsAgo = now - 30000
  const oneMinuteAgo = now - 60000

  // Filter without mutating state
  const recentTimestamps = transferTimestamps.filter(t => t > oneMinuteAgo)

  const txPerSecond = recentTimestamps.filter(t => t > oneSecondAgo).length
  const txPer30Seconds = recentTimestamps.filter(t => t > thirtySecondsAgo).length
  const txPerMinute = recentTimestamps.length

  return { txPerSecond, txPer30Seconds, txPerMinute }
}

// Clean up old timestamps periodically
const cleanupOldTimestamps = () => {
  const oneMinuteAgo = Date.now() - 60000
  transferTimestamps = transferTimestamps.filter(t => t > oneMinuteAgo)
}

// Process new transfers reactively
$effect(() => {
  if (transfers.length > processedCount) {
    const newTransfers = transfers.slice(processedCount)
    newTransfers.forEach(() => {
      const now = Date.now()
      transferTimestamps = [now, ...transferTimestamps].slice(0, 2000) // Increased for filtering
      cleanupOldTimestamps() // Clean up old entries
    })
    processedCount = transfers.length
  }
})

let rates = $derived(getTransferRates())
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
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-6 text-xs">
      <div class="flex items-center gap-1">
        <span class="text-zinc-400">tx/s:</span>
        <span class="text-zinc-200 font-medium">{rates.txPerSecond}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-zinc-400">tx/30s:</span>
        <span class="text-zinc-200 font-medium">{rates.txPer30Seconds}</span>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-zinc-400">tx/m:</span>
        <span class="text-zinc-200 font-medium">{rates.txPerMinute}</span>
      </div>
    </div>

    <button
      onclick={toggleMute}
      class="flex items-center gap-1 px-2 py-1 text-xs rounded hover:bg-zinc-800 transition-colors"
      class:text-zinc-400={!isMuted}
      class:text-red-400={isMuted}
      title={isMuted ? "Click to enable audio" : "Click to mute audio"}
    >
      {#if isMuted}
        <svg
          class="w-4 h-4"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.617.793L4.617 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.617l3.766-3.793a1 1 0 011.617.793zM16 8a1 1 0 011 1v2a1 1 0 11-2 0V9a1 1 0 011-1z"
            clip-rule="evenodd"
          />
          <path
            fill-rule="evenodd"
            d="M15.293 6.293a1 1 0 011.414 0L18 7.586l1.293-1.293a1 1 0 111.414 1.414L19.414 9l1.293 1.293a1 1 0 01-1.414 1.414L18 10.414l-1.293 1.293a1 1 0 01-1.414-1.414L16.586 9l-1.293-1.293a1 1 0 010-1.414z"
            clip-rule="evenodd"
          />
        </svg>
        <span>Muted</span>
      {:else}
        <svg
          class="w-4 h-4"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.617.793L4.617 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.617l3.766-3.793a1 1 0 011.617.793zM14.657 2.929a1 1 0 011.414 0A9.972 9.972 0 0119 10a9.972 9.972 0 01-2.929 7.071 1 1 0 01-1.414-1.414A7.971 7.971 0 0017 10c0-2.21-.894-4.208-2.343-5.657a1 1 0 010-1.414zm-2.829 2.828a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z"
            clip-rule="evenodd"
          />
        </svg>
        <span>Audio</span>
      {/if}
    </button>
  </div>
</Card>
