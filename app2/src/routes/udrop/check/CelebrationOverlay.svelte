<script lang="ts">
import AnimatedArrows from "$lib/components/AnimatedArrows.svelte"
import Button from "$lib/components/ui/Button.svelte"

interface Props {
  show: boolean
  totalTokens: number
  stakingPercentage: number
  onClose: () => void
}

let { show, totalTokens, stakingPercentage, onClose }: Props = $props()

let fadeIn = $state(false)

// Trigger fade in when shown
$effect(() => {
  if (show) {
    setTimeout(() => {
      fadeIn = true
    }, 100)
  } else {
    fadeIn = false
  }
})

function handleClose() {
  onClose()
}
</script>

{#if show}
  <div
    class="fixed inset-0 z-[9999] bg-black flex items-center justify-center"
    role="dialog"
    aria-modal="true"
  >
    <!-- Animated arrows background -->
    <AnimatedArrows
      color="#4bb7c3"
      squareSize={8}
      gridGap={10}
      flickerChance={0.4}
      maxOpacity={0.8}
      fadeTop={true}
      class="z-0"
    />

    <div class="relative z-10 text-center px-4 sm:px-8 max-w-6xl mx-auto">
      <!-- Total Tokens -->
      <div
        class="mb-12 sm:mb-16 md:mb-24 transition-all duration-1000 ease-out"
        class:opacity-100={fadeIn}
        class:translate-y-0={fadeIn}
        class:opacity-0={!fadeIn}
        class:translate-y-8={!fadeIn}
      >
        <div class="text-sm sm:text-lg md:text-xl text-zinc-500 font-semibold uppercase tracking-[0.2em] mb-4 sm:mb-6 md:mb-8 opacity-80">
          TOTAL TOKENS
        </div>
        <div class="text-5xl sm:text-6xl md:text-7xl lg:text-8xl xl:text-9xl font-black text-white leading-none tracking-tight">
          {totalTokens.toLocaleString()}
          <span class="text-accent ml-3 sm:ml-4 md:ml-6 font-black">U</span>
        </div>
      </div>

      <!-- Pre-Stake Allowance -->
      <div
        class="mb-12 sm:mb-16 md:mb-24 transition-all duration-1000 ease-out delay-200"
        class:opacity-100={fadeIn}
        class:translate-y-0={fadeIn}
        class:opacity-0={!fadeIn}
        class:translate-y-8={!fadeIn}
      >
        <div class="text-sm sm:text-lg md:text-xl text-zinc-500 font-semibold uppercase tracking-[0.2em] mb-4 sm:mb-6 md:mb-8 opacity-80">
          PRE-STAKE ALLOWANCE
        </div>
        <div class="text-5xl sm:text-6xl md:text-7xl lg:text-8xl xl:text-9xl font-black text-accent leading-none tracking-tight">
          {stakingPercentage} <span class="font-black">%</span>
        </div>
      </div>

      <!-- Close button -->
      <div
        class="mt-8 sm:mt-12 md:mt-16 transition-all duration-1000 ease-out delay-400"
        class:opacity-100={fadeIn}
        class:translate-y-0={fadeIn}
        class:opacity-0={!fadeIn}
        class:translate-y-8={!fadeIn}
      >
        <Button
          variant="primary"
          class="px-8 sm:px-12 md:px-16 py-3 sm:py-4 md:py-5 text-base sm:text-lg font-semibold rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 hover:scale-105"
          onclick={handleClose}
        >
          Continue →
        </Button>
      </div>
    </div>

    <!-- Close X button -->
    <button
      class="absolute top-6 right-6 text-zinc-400 hover:text-white transition-colors p-2"
      onclick={handleClose}
      aria-label="Close"
    >
      <svg
        class="w-6 h-6"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M6 18L18 6M6 6l12 12"
        />
      </svg>
    </button>
  </div>
{/if}

<style>
/* Enhanced button hover effects */
button:hover {
  transform: translateY(-1px);
}
</style>
