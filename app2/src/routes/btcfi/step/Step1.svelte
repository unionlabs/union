<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import { getBTCFIPoints } from "$lib/dashboard/queries/public"
import { runPromise } from "$lib/runtime"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { cosmosStore } from "$lib/wallet/cosmos/config.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import { Effect, Option, pipe } from "effect"
import * as S from "effect/Schema"
import StepLayout from "./StepLayout.svelte"

interface Props {
  onNext: (address: string, points: number) => void
}

let { onNext }: Props = $props()

let walletAddress = $state("")
let isLoading = $state(false)
let hasAutoFilled = $state(false)

let isConnected = $derived(cosmosStore.connectionStatus === "connected")

// Convert Cosmos address to bbn1 format
let cosmosAddressBbn1 = $derived(
  pipe(
    wallets.cosmosAddress,
    Option.map(cosmosDisplay =>
      pipe(
        cosmosDisplay,
        Ucs05.anyDisplayToCanonical,
        canonical =>
          S.decodeUnknownSync(Ucs05.Bech32FromCanonicalBytesWithPrefix("bbn1"))(canonical),
      )
    ),
  ),
)

// Auto-populate input only once when wallet connects
$effect(() => {
  if (isConnected && Option.isSome(cosmosAddressBbn1) && !hasAutoFilled) {
    walletAddress = Option.getOrThrow(cosmosAddressBbn1)
    hasAutoFilled = true
  }
  // Reset auto-fill flag if wallet disconnects
  if (!isConnected) {
    hasAutoFilled = false
  }
})

const handleConnectWallet = () => {
  uiStore.openWalletModal()
}

const handleCheck = () => {
  const addressToUse = walletAddress.trim()
  if (!addressToUse) {
    return
  }

  isLoading = true

  runPromise(
    pipe(
      getBTCFIPoints(addressToUse),
      Effect.tap((pointsOption) =>
        Effect.sync(() => {
          const points = Option.getOrElse(pointsOption, () => 0)
          onNext(addressToUse, points)
        })
      ),
      Effect.ensuring(
        Effect.sync(() => {
          isLoading = false
        }),
      ),
    ),
  )
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-4xl font-semibold">Check Your BTCfi Points</h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect your wallet or enter your Babylon Genesis address to view your Union BTCfi
            points.
          </p>
        </div>
      </div>

      <div class="space-y-3">
        {#if !isConnected}
          <div class="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 bg-orange-500/20 rounded-full flex items-center justify-center">
                <svg
                  class="w-4 h-4 text-orange-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4v2m0 4v2M7 3h10a2 2 0 012 2v14a2 2 0 01-2 2H7a2 2 0 01-2-2V5a2 2 0 012-2z"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-orange-400">Wallet Not Connected</div>
                <div class="text-xs text-zinc-400">
                  <button
                    type="button"
                    onclick={handleConnectWallet}
                    class="underline text-white hover:text-zinc-200 cursor-pointer transition-colors"
                  >
                    Connect a wallet
                  </button>
                  {" "}or enter an address below
                </div>
              </div>
            </div>
          </div>
        {:else}
          <div class="bg-accent/10 border border-accent/20 rounded-lg p-4">
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 bg-accent/20 rounded-full flex items-center justify-center">
                <svg
                  class="w-4 h-4 text-accent"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-accent">Wallet Connected</div>
                <div class="text-xs text-zinc-400 font-mono">
                  {#if Option.isSome(cosmosAddressBbn1)}
                    {Option.getOrThrow(cosmosAddressBbn1)?.slice(0, 12)}...{
                      Option.getOrThrow(cosmosAddressBbn1)?.slice(-6)
                    }
                  {/if}
                </div>
              </div>
              {#if walletAddress !== Option.getOrUndefined(cosmosAddressBbn1)}
                <button
                  type="button"
                  onclick={() => {
                    if (Option.isSome(cosmosAddressBbn1)) {
                      walletAddress = Option.getOrThrow(cosmosAddressBbn1)
                    }
                  }}
                  class="p-1.5 hover:bg-accent/20 rounded transition-colors"
                  title="Use connected address"
                >
                  <svg
                    class="w-4 h-4 text-accent"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                    />
                  </svg>
                </button>
              {/if}
            </div>
          </div>
        {/if}

        <Input
          id="wallet-address"
          value={walletAddress}
          label=""
          placeholder="bbn1..."
          type="text"
          disabled={isLoading}
          oninput={(e) => walletAddress = (e.target as HTMLInputElement).value}
        />

        <Button
          variant="primary"
          onclick={walletAddress.trim() ? handleCheck : handleConnectWallet}
          disabled={(!walletAddress.trim() && !isConnected) || isLoading}
          class="w-full"
        >
          {#if isLoading}
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
              </div>
              <span>Checking...</span>
            </div>
          {:else if walletAddress.trim()}
            Check Points
          {:else}
            Connect Wallet
          {/if}
        </Button>
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
