<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StepLayout from "./StepLayout.svelte"

interface Props {
  walletAddress: string
  btcfiPoints: number | null
  onBack: () => void
}

let { walletAddress, btcfiPoints, onBack }: Props = $props()

let isEligible = $derived(btcfiPoints && btcfiPoints > 0)

// Tier based on points
const tierEmoji = btcfiPoints === null || btcfiPoints === 0
  ? ""
  : btcfiPoints >= 100000
  ? "🐋"
  : btcfiPoints >= 15000
  ? "🐬"
  : btcfiPoints >= 5000
  ? "🐟"
  : "🦐"
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-4xl font-semibold">Your BTCFI Points</h1>
          {#if isEligible}
            <p class="text-sm text-zinc-400 leading-relaxed mt-3">
              Points will be converted into rewards. Follow <a
                href="https://x.com/union_build"
                target="_blank"
                rel="noopener noreferrer"
                class="underline"
              >@union_build</a> for updates.
            </p>
            <p class="text-xs font-mono text-zinc-400 mt-2">
              {walletAddress?.slice(0, 14)}...{walletAddress?.slice(-6)}
            </p>
          {:else}
            <p class="text-xs font-mono text-zinc-400 mt-2">
              {walletAddress?.slice(0, 14)}...{walletAddress?.slice(-6)}
            </p>
            <p class="text-sm text-zinc-400 leading-relaxed mt-3">
              No points associated with this address. To check another address, click the button
              below.
            </p>
          {/if}
        </div>
      </div>

      <div class="flex flex-col gap-3">
        <div class="flex-1 flex items-center justify-center">
          {#if isEligible}
            <div class="bg-accent/10 border border-accent/20 rounded-lg p-6 w-full">
              <div class="text-5xl font-bold text-accent text-center">
                {btcfiPoints?.toLocaleString()}
              </div>
              <div class="text-sm text-zinc-400 mt-3 text-center">Points</div>
            </div>
          {:else}
            <div class="bg-accent/10 border border-accent/20 rounded-lg p-6 w-full text-center">
              <div class="text-5xl font-bold text-accent text-center">
                {btcfiPoints?.toLocaleString()}
              </div>
              <div class="text-sm text-zinc-400 mt-3 text-center">Points</div>
            </div>
          {/if}
        </div>

        <div class="space-y-3">
          <Button
            variant="primary"
            onclick={onBack}
            class="w-full"
          >
            Check Another Address
          </Button>
        </div>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="block lg:hidden absolute top-4 left-4 right-4 z-10">
      <h1 class="text-4xl font-semibold text-center">Your BTCFI Points</h1>
    </div>
    <div class="aspect-square relative flex items-center justify-center">
      <div class="ball border-accent absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-0">
        <span></span>
        <span></span>
        <span></span>
        <span></span>
        <span></span>
        <span></span>
      </div>
      {#if !isEligible}
        <svg
          class="w-20 h-20 text-accent mx-auto relative z-10 star-spin"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
        >
          <circle
            cx="12"
            cy="12"
            r="10"
            fill="white"
            stroke="currentColor"
            stroke-width="2"
          />
          <path
            fill="currentColor"
            d="M8.17 2.76A10.1 10.1 0 0 1 12 2c1.31 0 2.61.26 3.83.76c1.21.5 2.31 1.24 3.24 2.17s1.67 2.03 2.17 3.24c.5 1.22.76 2.52.76 3.83c0 2.65-1.05 5.2-2.93 7.07A9.97 9.97 0 0 1 12 22a10.1 10.1 0 0 1-3.83-.76a10 10 0 0 1-3.24-2.17A9.97 9.97 0 0 1 2 12c0-2.65 1.05-5.2 2.93-7.07c.93-.93 2.03-1.67 3.24-2.17M12 17l1.56-3.42L17 12l-3.44-1.56L12 7l-1.57 3.44L7 12l3.43 1.58z"
          />
        </svg>
      {:else}
        <div class="text-6xl mx-auto relative z-10">{tierEmoji}</div>
      {/if}
    </div>
  {/snippet}
</StepLayout>

<style>
.ball span {
  width: 0;
  height: 0;
  border-radius: 50%;
  display: block;
  transition: all 2s ease-in-out;
  transform: translate(-50%, -50%);
  position: absolute;
  border: 1px solid transparent;
  animation: ballsAnimationBigger 6s infinite linear;
  pointer-events: none;
}

@keyframes ballsAnimationBigger {
  0% {
    border-color: transparent;
    opacity: 0;
  }
  20% {
    border-color: inherit;
  }
  80% {
    border-color: transparent;
    opacity: 1;
  }
  100% {
    width: 100vh;
    height: 100vh;
  }
}

.ball span:nth-child(2) {
  animation-delay: 1s;
}
.ball span:nth-child(3) {
  animation-delay: 2s;
}
.ball span:nth-child(4) {
  animation-delay: 3s;
}
.ball span:nth-child(5) {
  animation-delay: 4s;
}
.ball span:nth-child(6) {
  animation-delay: 5s;
}

.star-spin {
  animation: starSpin 3s linear infinite, starGlow 3s ease-in-out infinite;
}

@keyframes starSpin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

@keyframes starGlow {
  0% {
    filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.3)) 
            drop-shadow(0 0 4px rgba(255, 255, 255, 0.2));
    opacity: 0.8;
  }
  25% {
    filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.5)) 
            drop-shadow(0 0 12px rgba(255, 255, 255, 0.3))
            drop-shadow(0 0 20px rgba(255, 255, 255, 0.1));
    opacity: 1;
  }
  50% {
    filter: drop-shadow(0 0 8px rgba(255, 255, 255, 0.6)) 
            drop-shadow(0 0 16px rgba(255, 255, 255, 0.4))
            drop-shadow(0 0 28px rgba(255, 255, 255, 0.15));
    opacity: 1;
  }
  75% {
    filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.5)) 
            drop-shadow(0 0 12px rgba(255, 255, 255, 0.3))
            drop-shadow(0 0 20px rgba(255, 255, 255, 0.1));
    opacity: 1;
  }
  100% {
    filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.3)) 
            drop-shadow(0 0 4px rgba(255, 255, 255, 0.2));
    opacity: 0.8;
  }
}
</style>
