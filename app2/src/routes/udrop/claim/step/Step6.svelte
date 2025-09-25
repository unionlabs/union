<script lang="ts">
import { goto } from "$app/navigation"
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { formatUnits } from "viem"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onRestart: () => void
}

let { onRestart }: Props = $props()

let transactionHash = $derived<Option.Option<string>>(
  typeof window !== "undefined"
    ? Option.fromNullable(localStorage.getItem("lastClaimTxHash"))
    : Option.none(),
)

function handleViewTransaction() {
  Option.match(transactionHash, {
    onNone: () => {},
    onSome: (hash) => window.open(`https://etherscan.io/tx/${hash}`, "_blank"),
  })
}

function handleStake() {
  goto("/stake")
}

function handleUniswap() {
  // Direct link to the Union/WETH pool on Uniswap
  const uniswapPoolUrl =
    "https://app.uniswap.org/explore/pools/ethereum/0x0801481ba598d86e221a5ff0ccb02c97d5b0fbd803c662c66af604aa35119fe0"
  window.open(uniswapPoolUrl, "_blank")
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            eU Claimed Successfully!
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Congratulations! Your eU tokens have been successfully claimed. View your staking
            position in the dashboard or explore the liquidity pool on Uniswap.
          </p>
        </div>
      </div>

      <div class="space-y-4">
        <!-- Transaction Info -->
        {#if Option.isSome(transactionHash)}
          <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm font-medium text-white mb-1">Transaction Hash</div>
                <div class="text-xs font-mono text-zinc-400">
                  {transactionHash.value.slice(0, 10)}...{transactionHash.value.slice(-8)}
                </div>
              </div>
              <Button
                variant="secondary"
                class="text-xs px-3 py-1.5"
                onclick={handleViewTransaction}
              >
                View
              </Button>
            </div>
          </div>
        {/if}

        <!-- Action Buttons -->
        <div class="flex flex-col gap-3">
          <Button
            variant="primary"
            class="flex items-center justify-center gap-3 w-full"
            onclick={handleStake}
          >
            <svg
              class="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
            View Staking Dashboard
          </Button>

          <Button
            variant="secondary"
            class="flex items-center justify-center gap-3 w-full"
            onclick={handleUniswap}
          >
            <svg
              class="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"
              />
            </svg>
            View Uniswap Pool
          </Button>
        </div>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">eU Claimed Successfully!</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Congratulations! Your eU tokens have been successfully claimed.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Thank eU Video -->
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
              src="https://videos.cdn.union.build/thank-u.webm"
              type="video/webm"
            >
            <!-- Fallback for browsers that don't support the video -->
            <div class="w-full h-full flex items-center justify-center">
              <div class="text-center">
                <div class="text-xl font-bold text-accent mb-2">Thank eU!</div>
              </div>
            </div>
          </video>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
