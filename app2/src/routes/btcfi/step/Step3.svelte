<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import { verifyBTCFIWallet } from "$lib/dashboard/queries/public"
import { runPromise } from "$lib/runtime"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { cosmosStore } from "$lib/wallet/cosmos/config.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import * as Address from "@unionlabs/sdk/schema/address"
import { Effect, Option, ParseResult, pipe, Schema as S } from "effect"
import StepLayout from "./StepLayout.svelte"

interface Props {
  walletAddress: string
  btcfiPoints: Option.Option<number>
  onBack: () => void
  onSuccess: () => void
}

let { walletAddress, btcfiPoints, onBack, onSuccess }: Props = $props()

let points = $derived(Option.getOrElse(btcfiPoints, () => 0))

let evmAddress = $state("")
let isLoading = $state(false)
let isVerified = $state(false)

let isCosmosConnected = $derived(cosmosStore.connectionStatus === "connected")

let cosmosAddressBbn = $derived(
  pipe(
    wallets.cosmosAddress,
    Option.map(cosmosDisplay =>
      pipe(
        cosmosDisplay,
        Ucs05.anyDisplayToCanonical,
        canonical =>
          S.decodeUnknownSync(Ucs05.Bech32FromCanonicalBytesWithPrefix("bbn"))(canonical),
      )
    ),
  ),
)

// Check if the connected Babylon address matches the one from Step 1
let walletMatches = $derived(
  Option.match(cosmosAddressBbn, {
    onNone: () => false,
    onSome: (addr) => addr.toLowerCase() === walletAddress.toLowerCase(),
  }),
)

// Validate EVM address
const validateEvmAddress = (address: string): { valid: boolean; error: string } => {
  if (!address || address.trim() === "") {
    return { valid: false, error: "" }
  }

  const trimmed = address.trim()

  return pipe(
    S.validateEither(Address.ERC55)(trimmed),
    (either) =>
      either._tag === "Right"
        ? { valid: true, error: "" }
        : {
          valid: false,
          error: pipe(
            either.left,
            ParseResult.ArrayFormatter.formatErrorSync,
            (errors) => errors[0]?.message ?? "Invalid EVM address",
          ),
        },
  )
}

let evmValidation = $derived(validateEvmAddress(evmAddress))
let isEvmAddressValid = $derived(evmValidation.valid)
let evmAddressError = $derived(evmValidation.error)

const handleConnectCosmosWallet = () => {
  uiStore.openWalletModal()
}

const handleVerify = () => {
  if (!isCosmosConnected) {
    uiStore.openWalletModal()
    return
  }

  if (!isEvmAddressValid) {
    return
  }

  const addressToUse = evmAddress.trim()
  isLoading = true

  const message =
    `I verify that I own this Babylon address and want to receive BTCfi rewards on Ethereum address: ${addressToUse}`

  runPromise(
    pipe(
      Effect.sync(() => {
        const walletApi = cosmosStore.connectedWallet
          && (window as any)[cosmosStore.connectedWallet]
        if (!walletApi) {
          throw new Error("No wallet available")
        }
        return walletApi
      }),
      Effect.flatMap((walletApi) =>
        Effect.tryPromise({
          try: () => {
            const chainId = "bbn-1"
            return walletApi.signArbitrary(chainId, walletAddress, message)
          },
          catch: (error) => new Error(`Failed to sign message: ${error}`),
        })
      ),
      Effect.flatMap((signature) =>
        verifyBTCFIWallet({
          bbnAddress: walletAddress,
          message,
          signature: JSON.stringify(signature),
          evmAddress: addressToUse,
        })
      ),
      Effect.tap((result) =>
        Effect.sync(() => {
          if (result) {
            isVerified = true
          }
        })
      ),
      Effect.catchAll((error) =>
        Effect.sync(() => {
          console.error("Error signing message:", error)
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
          <h1 class="text-4xl font-semibold">Verify Ownership</h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect wallet and sign message to link your Ethereum address.
          </p>
          <p class="text-xs font-mono text-zinc-500 mt-2">
            {walletAddress?.slice(0, 14)}...{walletAddress?.slice(-6)} · {points.toLocaleString()}
            points
          </p>
        </div>
      </div>

      <div class="space-y-3">
        {#if isVerified}
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
                <div class="text-sm font-medium text-accent">Ethereum Address Linked</div>
                <div class="text-xs text-zinc-400">
                  Your rewards will be sent to this address. You can update it anytime by repeating
                  this process.
                </div>
              </div>
            </div>
          </div>
        {:else}
          {#if !isCosmosConnected}
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
                  <div class="text-sm font-medium text-orange-400">
                    Babylon Wallet Not Connected
                  </div>
                  <div class="text-xs text-zinc-400">
                    <button
                      type="button"
                      onclick={handleConnectCosmosWallet}
                      class="underline text-white hover:text-zinc-200 cursor-pointer transition-colors"
                    >
                      Connect wallet
                    </button>
                    {" "}to continue
                  </div>
                </div>
              </div>
            </div>
          {:else if !walletMatches}
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
                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    />
                  </svg>
                </div>
                <div class="flex-1">
                  <div class="text-sm font-medium text-orange-400">Wrong Wallet Connected</div>
                  <div class="text-xs text-zinc-400">
                    Please connect the wallet from Step 1
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
                    {walletAddress?.slice(0, 10)}...{walletAddress?.slice(-4)}
                  </div>
                </div>
              </div>
            </div>
          {/if}

          <div class="space-y-1">
            <Input
              id="evm-address"
              value={evmAddress}
              label=""
              placeholder="Enter your Ethereum address"
              type="text"
              disabled={isLoading || !isCosmosConnected || !walletMatches}
              oninput={(e) => evmAddress = (e.target as HTMLInputElement).value}
            />
            {#if evmAddressError && evmAddress.trim()}
              <div class="text-xs text-red-400 text-center">
                {evmAddressError}
              </div>
            {/if}
          </div>

          <Button
            variant="primary"
            onclick={handleVerify}
            disabled={!isCosmosConnected || !walletMatches || !isEvmAddressValid || isLoading}
            class="w-full"
          >
            {#if isLoading}
              <div class="flex items-center gap-2">
                <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                </div>
                <span>Signing...</span>
              </div>
            {:else}
              Sign & Link
            {/if}
          </Button>
        {/if}

        <Button
          variant="secondary"
          onclick={isVerified ? onSuccess : onBack}
          class="w-full"
          disabled={isLoading}
        >
          {isVerified ? "Check Another Address" : "Back"}
        </Button>
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="block lg:hidden absolute top-4 left-4 right-4 z-10">
      <h1 class="text-4xl font-semibold text-center">Verify Ownership</h1>
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
      {#if isVerified}
        <svg
          class="w-20 h-20 text-accent mx-auto relative z-10"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      {:else}
        <svg
          class="w-20 h-20 text-accent mx-auto relative z-10 star-spin"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
        >
          <path
            fill="currentColor"
            d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"
          />
        </svg>
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
