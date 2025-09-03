<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onBack: () => void
  isLoadingAllocation: boolean
  preStakedAmount: string
}

let { onBack, isLoadingAllocation, preStakedAmount }: Props = $props()
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-4xl font-semibold">
            Your pre-stake
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Your pre-stake allocation will automatically convert to liquid staking rewards when eU
            launches.
          </p>
        </div>
      </div>

      <div class="flex gap-3">
        <Button
          variant="primary"
          class="flex flex-1 items-center justify-center gap-3"
          href="/dashboard"
        >
          Go to Dashboard
        </Button>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">Your eU Allocation</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Once liquid staking goes live, you will receive
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Union Token Video - Normal (not grayscale) -->
        <div
          class="w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <video
            class="w-full h-full object-cover"
            autoplay
            loop
            muted
            playsinline
          >
            <source
              src="https://videos.cdn.union.build/spin-token.webm"
              type="video/webm"
            >
            <!-- Fallback for browsers that don't support the video -->
            <div class="w-full h-full flex items-center justify-center">
              <div class="w-24 h-24 bg-accent/20 rounded-full flex items-center justify-center border-4 border-accent">
                <span class="text-3xl font-bold text-accent">eU</span>
              </div>
            </div>
          </video>

          <!-- Overlay content for allocation display -->
          {#if !isLoadingAllocation}
            <div class="absolute inset-0 flex items-center justify-center bg-black/40">
              <div class="text-center space-y-4">
                <!-- Big eU Amount -->
                <div class="relative">
                  <div class="text-4xl md:text-5xl lg:text-6xl font-black text-white">
                    {preStakedAmount}
                  </div>
                  <div class="text-xl md:text-2xl font-bold text-accent mt-2">
                    eU
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <!-- Loading overlay -->
            <div class="absolute inset-0 flex items-center justify-center bg-black/40">
              <div class="flex items-center">
                <div class="w-8 h-8 border-4 border-accent border-t-transparent rounded-full animate-spin">
                </div>
                <span class="ml-3 text-zinc-200">Loading your allocation...</span>
              </div>
            </div>
          {/if}
        </div>

        <!-- Powered by Escher -->
        <div class="absolute bottom-4 left-4 flex flex-col gap-1 text-zinc-500 text-left">
          <span class="text-xs font-mono mb-1">POWERED BY</span>
          <img
            src="escher-logo.svg"
            alt="Escher"
            class="w-28 h-auto"
          />
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
