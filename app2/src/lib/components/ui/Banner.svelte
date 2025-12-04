<script lang="ts">
import SharpChevronLeftIcon from "$lib/components/icons/SharpChevronLeftIcon.svelte"
import { bannerQuery } from "$lib/queries/banner.svelte"
import { runFork$ } from "$lib/runtime"
import { bannerStore } from "$lib/stores/banner.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { onMount } from "svelte"

let dismissedBanners = $state<string[]>([])
let currentIndex = $state(0)

function getBannerStyle(type: string) {
  switch (type) {
    case "error":
      return "bg-red-500 text-white"
    case "warning":
      return "bg-yellow-400 text-black"
    case "info":
    default:
      return "bg-blue-500 text-white"
  }
}

function dismissCurrentBanner() {
  if (!currentBanner) {
    return
  }
  dismissedBanners = [...dismissedBanners, currentBanner.message]
  currentIndex = 0
}

function nextBanner() {
  if (visibleBanners.length > 0) {
    currentIndex = (currentIndex + 1) % visibleBanners.length
  }
}

function prevBanner() {
  if (visibleBanners.length > 0) {
    currentIndex = (currentIndex - 1 + visibleBanners.length) % visibleBanners.length
  }
}

onMount(() => {
  runFork$(() => bannerQuery())
})

$effect(() => {
  uiStore.edition
  dismissedBanners = []
  currentIndex = 0
})

const allBanners = $derived(bannerStore.getBannersForEdition(uiStore.edition))
const visibleBanners = $derived(allBanners.filter(b => !dismissedBanners.includes(b.message)))
const currentBanner = $derived(visibleBanners[currentIndex])
</script>

{#if currentBanner}
  <div class="{getBannerStyle(currentBanner.type)} relative">
    <div class="px-4 sm:px-6 py-3 sm:py-3.5">
      <div class="flex flex-col gap-2">
        <!-- Top Row: Type + Actions -->
        <div class="flex items-center justify-between gap-3 sm:gap-4">
          <span class="text-sm font-semibold capitalize">
            {currentBanner.type}
          </span>

          <!-- Actions -->
          <div class="flex items-center gap-2.5 flex-shrink-0">
            {#if visibleBanners.length > 1}
              <div class="flex items-center gap-1 bg-black/10 rounded-md px-2 py-1">
                <button
                  class="hover:opacity-70 transition-opacity cursor-pointer flex items-center justify-center"
                  onclick={prevBanner}
                  title="Previous banner"
                >
                  <SharpChevronLeftIcon class="size-3.5" />
                </button>

                <span class="text-xs font-medium tabular-nums px-1.5">
                  {currentIndex + 1}/{visibleBanners.length}
                </span>

                <button
                  class="hover:opacity-70 transition-opacity cursor-pointer flex items-center justify-center"
                  onclick={nextBanner}
                  title="Next banner"
                >
                  <SharpChevronLeftIcon class="size-3.5 rotate-180" />
                </button>
              </div>
            {/if}

            <button
              class="hover:opacity-70 transition-opacity cursor-pointer flex items-center justify-center p-1"
              onclick={dismissCurrentBanner}
              title="Dismiss banner"
            >
              <svg
                class="size-4"
                viewBox="0 0 16 16"
                fill="currentColor"
              >
                <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Bottom Row: Message -->
        <div class="text-sm leading-relaxed">
          {currentBanner.message}
        </div>
      </div>
    </div>
  </div>
{/if}
