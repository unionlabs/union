<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { formatUnits } from "viem"
import { goto } from "$app/navigation"
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
  // Union token on Ethereum mainnet (from token-whitelist.json)
  const unionTokenAddress = "0xba5eD44733953d79717F6269357C77718C8Ba5ed"
  const uniswapUrl = `https://app.uniswap.org/swap?outputCurrency=${unionTokenAddress}&chain=mainnet`
  window.open(uniswapUrl, "_blank")
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            U Claimed Successfully!
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Congratulations! Your U tokens have been successfully claimed. 
            You can now stake them to earn rewards or trade them on Uniswap.
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
          <div class="text-sm text-zinc-400 mb-2">What would you like to do with your U tokens?</div>
          
          <Button
            variant="primary"
            class="flex items-center justify-center gap-3 w-full"
            onclick={handleStake}
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Stake U Tokens
          </Button>
          
          <Button
            variant="secondary"
            class="flex items-center justify-center gap-3 w-full"
            onclick={handleUniswap}
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
            </svg>
            Trade on Uniswap
          </Button>
        </div>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">U Claimed Successfully!</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Congratulations! Your U tokens have been successfully claimed.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Thank U Video -->
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
                <div class="text-xl font-bold text-accent mb-2">Thank U!</div>
              </div>
            </div>
          </video>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
