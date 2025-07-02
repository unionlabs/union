<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import { Option, pipe } from "effect"
import { transactionAudio } from "../audio"
import type { ActiveWalletRates, TransferRates } from "../types"

interface Props {
  transferRates: Option.Option<TransferRates>
  activeWalletRates: Option.Option<ActiveWalletRates>
  connectionStatus?: "connecting" | "connected" | "disconnected" | "error"
}

let {
  transferRates,
  activeWalletRates,
}: Props = $props()

const rates = $derived(transferRates)
const wallets = $derived(activeWalletRates)

// Individual field derivations - no fake defaults, just clean Option handling
const txPerMinute = $derived(
  pipe(rates, Option.map(r => r.txPerMinute), Option.getOrElse(() => "-")),
)
const txPerHour = $derived(pipe(rates, Option.map(r => r.txPerHour), Option.getOrElse(() => "-")))
const txPerDay = $derived(pipe(rates, Option.map(r => r.txPerDay), Option.getOrElse(() => "-")))
const txPer7Days = $derived(pipe(rates, Option.map(r => r.txPer7Days), Option.getOrElse(() => "-")))
const txPer30Days = $derived(
  pipe(rates, Option.map(r => r.txPer30Days), Option.getOrElse(() => "-")),
)

const sendersLastMin = $derived(
  pipe(wallets, Option.map(w => w.sendersLastMin), Option.getOrElse(() => "-")),
)
const sendersLastHour = $derived(
  pipe(wallets, Option.map(w => w.sendersLastHour), Option.getOrElse(() => "-")),
)
const sendersLastDay = $derived(
  pipe(wallets, Option.map(w => w.sendersLastDay), Option.getOrElse(() => "-")),
)
const sendersLast7d = $derived(
  pipe(wallets, Option.map(w => w.sendersLast7d), Option.getOrElse(() => "-")),
)
const sendersLast30d = $derived(
  pipe(wallets, Option.map(w => w.sendersLast30d), Option.getOrElse(() => "-")),
)

const receiversLastMin = $derived(
  pipe(wallets, Option.map(w => w.receiversLastMin), Option.getOrElse(() => "-")),
)
const receiversLastHour = $derived(
  pipe(wallets, Option.map(w => w.receiversLastHour), Option.getOrElse(() => "-")),
)
const receiversLastDay = $derived(
  pipe(wallets, Option.map(w => w.receiversLastDay), Option.getOrElse(() => "-")),
)
const receiversLast7d = $derived(
  pipe(wallets, Option.map(w => w.receiversLast7d), Option.getOrElse(() => "-")),
)
const receiversLast30d = $derived(
  pipe(wallets, Option.map(w => w.receiversLast30d), Option.getOrElse(() => "-")),
)

const totalLastMin = $derived(
  pipe(wallets, Option.map(w => w.totalLastMin), Option.getOrElse(() => "-")),
)
const totalLastHour = $derived(
  pipe(wallets, Option.map(w => w.totalLastHour), Option.getOrElse(() => "-")),
)
const totalLastDay = $derived(
  pipe(wallets, Option.map(w => w.totalLastDay), Option.getOrElse(() => "-")),
)
const totalLast7d = $derived(
  pipe(wallets, Option.map(w => w.totalLast7d), Option.getOrElse(() => "-")),
)
const totalLast30d = $derived(
  pipe(wallets, Option.map(w => w.totalLast30d), Option.getOrElse(() => "-")),
)

// Format uptime for display
const uptimeDisplay = $derived(() => {
  return pipe(
    rates,
    Option.map(r => r.serverUptimeSeconds),
    Option.map(seconds => {
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
    }),
    Option.getOrElse(() => "0s"),
  )
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
              <span class="text-zinc-400">5m:</span>
              <span class="text-zinc-100 tabular-nums text-[11px] sm:text-[10px]">
                {txPerMinute}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {txPerHour}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {txPerDay}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {txPer7Days}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {txPer30Days}
              </span>
            </div>
          </div>
        </section>

        <!-- Senders Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">senders:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">5m:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {sendersLastMin}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {sendersLastHour}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {sendersLastDay}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {sendersLast7d}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {sendersLast30d}
              </span>
            </div>
          </div>
        </section>

        <!-- Receivers Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">receivers:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">5m:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {receiversLastMin}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {receiversLastHour}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {receiversLastDay}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {receiversLast7d}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {receiversLast30d}
              </span>
            </div>
          </div>
        </section>

        <!-- Total Column -->
        <section class="space-y-1">
          <div class="text-zinc-500 font-mono font-medium text-xs">total:</div>
          <div class="space-y-0.5">
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">5m:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {totalLastMin}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1h:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {totalLastHour}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">1d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {totalLastDay}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">7d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {totalLast7d}
              </span>
            </div>
            <div class="flex justify-between font-mono items-center">
              <span class="text-zinc-400">30d:</span>
              <span class="text-zinc-100 tabular-nums text-[10px]">
                {totalLast30d}
              </span>
            </div>
          </div>
        </section>
      </div>
    </main>

    <!-- Info Footer -->
    <footer class="p-2 border-t border-zinc-800">
      <div class="text-[9px] text-zinc-400 font-mono leading-relaxed">
        <span class="text-zinc-300">info:</span> rolling timeframes show activity within each period
      </div>
    </footer>
  </div>
</Card>
