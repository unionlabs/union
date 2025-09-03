<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface AuthProvider {
  id: "twitter" | "github" | "discord"
  name: string
  icon: string
  iconColor: string
}

interface Props {
  onNext: () => void
  providers: Array<AuthProvider>
  loading: boolean
  handleLogin: (provider: AuthProvider) => void
}

let { onNext, providers, loading, handleLogin }: Props = $props()
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-4xl font-semibold">
            eU Liquid Staking
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Stake your eU and earn rewards while maintaining liquidity. Sign in to check your pre
            stake allocation.
          </p>
        </div>
      </div>

      {#if Option.isNone(dashboard.session)}
        <div class="space-y-3">
          {#each providers as provider (provider.id)}
            <Button
              variant="secondary"
              class="flex w-full items-center justify-center gap-3 relative hover:bg-accent/10 transition-colors"
              disabled={loading}
              onclick={() => handleLogin(provider)}
            >
              <svg
                class="w-5 h-5 {provider.iconColor}"
                viewBox="0 0 24 24"
              >
                {@html provider.icon}
              </svg>
              <span>
                Continue with {provider.name}
              </span>
              {#if loading}
                <div class="absolute right-4 w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                </div>
              {/if}
            </Button>
          {/each}
        </div>
      {:else}
        <div class="flex gap-3">
          <Button
            variant="primary"
            class="flex flex-1 items-center justify-center gap-3"
            onclick={onNext}
          >
            View Pre Stake
          </Button>
        </div>
      {/if}
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">eU Liquid Staking</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Stake your U and earn rewards while maintaining liquidity.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Union Token Video - Grayscale -->
        <div
          class="w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <video
            class="w-full h-full object-cover filter grayscale"
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
        </div>
        
        <!-- Powered by Escher -->
        <div class="absolute bottom-4 left-4 flex flex-col gap-1 text-zinc-500 text-left">
          <span class="text-xs font-mono mb-1">POWERED BY</span>
          <img src="escher-logo.svg" alt="Escher" class="w-28 h-auto" />
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
