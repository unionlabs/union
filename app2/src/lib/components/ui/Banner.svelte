<script lang="ts">
import { bannerStore } from "$lib/stores/banner.svelte"
import { uiStore } from "$lib/stores/ui.svelte"

let showBanner = $state(false)

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

$effect(() => {
  const currentBanner = bannerStore.getBannerForEdition(uiStore.edition)
  showBanner = currentBanner ? currentBanner.enabled : false
})

const currentBanner = $derived(bannerStore.getBannerForEdition(uiStore.edition))
</script>

{#if showBanner && currentBanner}
  <div class="{getBannerStyle(currentBanner.type)} px-3 sm:px-4 py-2 sm:py-3 text-center text-xs sm:text-sm font-medium relative">
    <div class="flex items-center justify-center gap-1 sm:gap-2 pr-8">
      <svg class="w-3 h-3 sm:w-4 sm:h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
        <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
      </svg>
      <span class="leading-tight">
        {currentBanner.message}
      </span>
    </div>
    <button 
      class="absolute right-2 sm:right-4 top-1/2 transform -translate-y-1/2 hover:opacity-70 font-bold cursor-pointer text-base sm:text-lg"
      onclick={() => showBanner = false}
      title="Dismiss banner"
    >
      ×
    </button>
  </div>
{/if} 