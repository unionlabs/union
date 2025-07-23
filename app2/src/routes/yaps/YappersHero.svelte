<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import YappersInfoModal from "./YappersInfoModal.svelte"

import { onDestroy, onMount } from "svelte"

let showPlayButton = $state(true)
let videoHovered = $state(false)
let seasonStartTime = new Date("2025-07-23T00:00:00Z") // Season 1 start date
let currentTime = $state(new Date())
let timerInterval: ReturnType<typeof setInterval>
let isInfoModalOpen = $state(false)

let videoElement: HTMLVideoElement

function handlePlayClick() {
  if (!videoElement) {
    return
  }
  videoElement.currentTime = 0
  videoElement.muted = false
  showPlayButton = false

  const element = videoElement as any
  if (element.requestFullscreen) {
    element.requestFullscreen()
  } else if (element.webkitRequestFullscreen) {
    element.webkitRequestFullscreen()
  } else if (element.msRequestFullscreen) {
    element.msRequestFullscreen()
  }
}

function formatTimeDuration(startDate: Date, currentDate: Date) {
  const diff = currentDate.getTime() - startDate.getTime()
  if (diff <= 0) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0 }
  }

  const second = 1000
  const minute = second * 60
  const hour = minute * 60
  const day = hour * 24

  return {
    days: Math.floor(diff / day),
    hours: Math.floor((diff % day) / hour),
    minutes: Math.floor((diff % hour) / minute),
    seconds: Math.floor((diff % minute) / second),
  }
}

function openInfoModal() {
  isInfoModalOpen = true
}

function closeInfoModal() {
  isInfoModalOpen = false
}

onMount(() => {
  // Update timer every second
  timerInterval = setInterval(() => {
    currentTime = new Date()
  }, 1000)
})

onDestroy(() => {
  if (timerInterval) {
    clearInterval(timerInterval)
  }
})
</script>

<!-- Hero Content Card -->
<div class="relative">
  <Card
    class="relative p-6 bg-gradient-to-br from-zinc-900/90 via-zinc-950/90 to-orange-950/30 border border-orange-900/50 backdrop-blur-sm"
  >
    <div class="flex flex-col lg:flex-row gap-4 lg:gap-8">
      <!-- Video Section -->
      <div
        class="relative group/video w-full lg:w-2/5 flex-shrink-0"
        onmouseenter={() => videoHovered = true}
        onmouseleave={() => videoHovered = false}
      >
        <div class="relative aspect-square bg-zinc-900 rounded-lg overflow-hidden ring-1 ring-zinc-800">
          <video
            bind:this={videoElement}
            class="w-full h-full object-cover"
            controls
            loop
            muted
            autoplay
            playsinline
          >
            <source
              src="https://videos.cdn.union.build/mad-yaps-v3.mp4"
              type="video/mp4"
            />
            Your browser does not support the video tag.
          </video>

          <!-- Play button overlay for unmuting -->
          {#if showPlayButton}
            <button
              onclick={handlePlayClick}
              class="absolute inset-0 flex items-center justify-center bg-black/20 hover:bg-black/30 transition-colors cursor-pointer group"
            >
              <div class="w-20 h-20 rounded-full bg-white/10 backdrop-blur-sm flex items-center justify-center group-hover:bg-white/20 transition-colors">
                <svg
                  class="w-8 h-8 text-white ml-1"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
                </svg>
              </div>
              <span
                class="absolute bottom-4 text-white text-sm font-medium bg-black/50 px-3 py-1 rounded-full"
              >
                Click to unmute
              </span>
            </button>
          {/if}
        </div>
      </div>

      <!-- Text Content Section -->
      <div class="flex flex-col justify-between w-full">
        <div class="space-y-6">
          <!-- Badge & Timer -->
          <div class="flex flex-row items-center gap-3">
            <div class="relative inline-flex">
              <div class="absolute inset-0 bg-gradient-to-r from-orange-500/20 to-yellow-500/20 blur-lg">
              </div>
              <div class="relative inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-gradient-to-r from-orange-500/10 to-yellow-500/10 border border-orange-500/30 backdrop-blur-sm">
                <div class="w-2 h-2 rounded-full bg-gradient-to-r from-orange-400 to-yellow-400 animate-pulse">
                </div>
                <span
                  class="text-xs font-bold text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-yellow-400 uppercase tracking-wider"
                >Season 1</span>
              </div>
            </div>

            <!-- Season Timer -->
            {#if currentTime}
              {@const timeDiff = formatTimeDuration(seasonStartTime, currentTime)}
              <div class="inline-flex items-center gap-2 text-sm font-mono">
                <svg
                  class="w-4 h-4 text-orange-500"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M16.2,16.2L11,13V7H12.5V12.2L17,14.9L16.2,16.2Z" />
                </svg>
                <span class="text-orange-300">
                  {timeDiff.days}d {timeDiff.hours.toString().padStart(2, "0")}h {
                    timeDiff.minutes.toString().padStart(2, "0")
                  }m {timeDiff.seconds.toString().padStart(2, "0")}s
                </span>
              </div>
            {/if}
          </div>

          <!-- Main Content -->
          <div class="space-y-4">
            <h1 class="text-2xl font-black leading-tight uppercase">
              <span>Kaito x Union Presents: Mad Yaps</span>
              <br />
            </h1>

            <p class="text-orange-200/80 text-md leading-relaxed font-medium">
              The old world collapsed. Chains shattered, giants fell, and liquidity turned to dust.
              From its ashes, Union rose. Ready to rebuild what once was, trustless and wild. In
              July 2025, the Yapocalypse begins. Speak now or be forgotten.
            </p>

            <div class="flex flex-wrap gap-3 text-sm pt-2">
              <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                <svg
                  class="w-4 h-4 text-orange-500 drop-shadow-[0_0_10px_rgba(234,88,12,0.8)]"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                </svg>
                <span>WAR READY</span>
              </div>
              <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                <svg
                  class="w-4 h-4 text-red-500 drop-shadow-[0_0_10px_rgba(239,68,68,0.8)]"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M17.66 11.2C17.43 10.9 17.15 10.64 16.89 10.38C16.22 9.78 15.46 9.35 14.82 8.72C13.33 7.26 13 4.85 13.95 3C13 3.23 12.17 3.75 11.46 4.32C8.87 6.4 7.85 10.07 9.07 13.22C9.11 13.32 9.15 13.42 9.15 13.55C9.15 13.77 9 13.97 8.8 14.05C8.57 14.15 8.33 14.09 8.14 13.93C8.08 13.88 8.04 13.83 8 13.76C6.87 12.33 6.69 10.28 7.45 8.64C5.78 10 4.87 12.3 5 14.47C5.06 14.97 5.12 15.47 5.29 15.97C5.43 16.57 5.7 17.17 6 17.7C7.08 19.43 8.95 20.67 10.96 20.92C13.1 21.19 15.39 20.8 17.03 19.32C18.86 17.66 19.5 15 18.56 12.72L18.43 12.46C18.22 12 17.66 11.2 17.66 11.2M14.5 17.5C14.22 17.74 13.76 18 13.4 18.1C12.28 18.5 11.16 17.94 10.5 17.28C11.69 17 12.4 16.12 12.61 15.23C12.78 14.43 12.46 13.77 12.33 13C12.21 12.26 12.23 11.63 12.5 10.94C12.69 11.32 12.89 11.7 13.13 12C13.9 13 15.11 13.44 15.37 14.8C15.41 14.94 15.43 15.08 15.43 15.23C15.46 16.05 15.1 16.95 14.5 17.5H14.5Z" />
                </svg>
                <span>FULL OCTANE</span>
              </div>
              <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                <svg
                  class="w-4 h-4 text-yellow-500 drop-shadow-[0_0_10px_rgba(234,179,8,0.8)]"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M7.5,2C5.71,3.15 4.5,5.18 4.5,7.5C4.5,9.82 5.71,11.85 7.53,13L4.5,22H10.5L14.5,10L18.5,22H24.5L21.47,13C23.29,11.85 24.5,9.82 24.5,7.5C24.5,5.18 23.29,3.15 21.5,2" />
                </svg>
                <span>CHROME BLESSED</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="mt-6 flex flex-col sm:flex-row gap-3">
          <Button
            variant="primary"
            class="order-1 sm:order-0"
            href="/auth/sign-in"
          >
            <span class="skew-x-[5deg]">JOIN THE YAPPERS</span>
            <svg
              class="w-5 h-5 skew-x-[5deg]"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M5,17.59L15.59,7H9V5H19V15H17V8.41L6.41,19L5,17.59Z" />
            </svg>
          </Button>
          <Button
            variant="text"
            onclick={openInfoModal}
          >
            <span class="skew-x-[5deg]">LEARN MORE</span>
            <svg
              class="w-5 h-5 skew-x-[5deg]"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z" />
            </svg>
          </Button>
        </div>
      </div>
    </div>
  </Card>
</div>

<YappersInfoModal
  isOpen={isInfoModalOpen}
  onClose={closeInfoModal}
/>
