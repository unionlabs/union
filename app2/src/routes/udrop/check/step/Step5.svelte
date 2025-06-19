<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import {
  addStargazeWallet,
  AddStargazeWalletState,
  type StargazeWalletResult,
} from "$lib/dashboard/services/add-stargaze-wallet"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { getCosmosWalletClient } from "$lib/services/cosmos/clients"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/config.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import { Effect, Option } from "effect"
import * as S from "effect/Schema"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onSkip: () => void
  onBack?: () => void
}

let { onNext, onSkip, onBack }: Props = $props()

let shouldVerify = $state(false)
let message = $state("")
let hasError = $state(false)
let currentState = $state(Option.none<AddStargazeWalletState>())
let walletConnected = $state(false)
let showSuccessAnimation = $state(false)

// Check if usetextr has Stargaze wallet in database
const hasStargazeWallet = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.hasStargazeWallet,
  }),
)

const connectedCosmosWallet = $derived(
  cosmosStore.connectionStatus === "connected" && cosmosStore.connectedWallet
    && cosmosWalletsInformation
    ? Option.fromNullable(
      cosmosWalletsInformation.find((wallet) => wallet.id === cosmosStore.connectedWallet),
    )
    : Option.none(),
)

const stargazeDisplayAddress = $derived(
  Option.flatMap(wallets.cosmosAddress, (cosmosAddress) => {
    const canonicalBytes = Ucs05.anyDisplayToCanonical(cosmosAddress)
    return S.decodeOption(Ucs05.Bech32FromCanonicalBytesWithPrefix("stars"))(canonicalBytes)
  }),
)

const storedStargazeAddress = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => Option.none(),
    onSome: (store) =>
      Option.match(store.allocation, {
        onNone: () => Option.none(),
        onSome: (alloc) => Option.fromNullable(alloc.stargaze_address),
      }),
  }),
)

// Check if current connected address matches stored address (only when both exist)
const addressMatches = $derived(
  Option.isNone(stargazeDisplayAddress) || Option.isNone(storedStargazeAddress)
    ? true // No comparison needed if either is missing
    : stargazeDisplayAddress.value.toLowerCase() === storedStargazeAddress.value.toLowerCase(),
)

// Show address mismatch warning when user has verified wallet but connected to different address
const showAddressMismatch = $derived(
  cosmosStore.connectionStatus === "connected"
    && Option.isSome(storedStargazeAddress)
    && !addressMatches
    && !walletConnected, // Hide warning if we just connected the wallet
)

// Simplified derived states following WalletDialog pattern
const isWalletConnected = $derived(cosmosStore.connectionStatus === "connected")
const hasStoredAddress = $derived(Option.isSome(storedStargazeAddress))
const hasVerifiedWallet = $derived(hasStargazeWallet && hasStoredAddress)

const isVerifying = $derived(shouldVerify)

const verificationStepMessage = $derived(
  !isVerifying
    ? Option.none()
    : Option.some(Option.match(currentState, {
      onNone: () => "Preparing verification...",
      onSome: (state) =>
        AddStargazeWalletState.$match(state, {
          SwitchChain: () => "Switching chain to Stargaze...",
          Signing: () => "Please sign the message in your wallet...",
          Verifying: () => "Submitting verification to server...",
          Updating: () => "Updating wallet data...",
        }),
    })),
)

function startVerificationProcess() {
  const program = Effect.gen(function*() {
    const cosmosWalletClient = yield* getCosmosWalletClient

    // Start with SwitchChain to ensure proper setup using Stargaze mainnet
    let currentStateLocal: Option.Option<AddStargazeWalletState> = Option.some(
      AddStargazeWalletState.SwitchChain({ chainId: "stargaze-1" }),
    )
    message = "Switching to Stargaze chain..."

    while (Option.isSome(currentStateLocal)) {
      const result: StargazeWalletResult = yield* addStargazeWallet(
        currentStateLocal.value,
        cosmosWalletClient,
      )

      if (Option.isSome(result.error)) {
        console.error("Error in state machine:", result.error.value)
        message = result.message
        hasError = true
        shouldVerify = false
        return
      }

      message = result.message

      if (result.completed) {
        currentStateLocal = Option.none()
        walletConnected = true
        showSuccessAnimation = true

        // Hide success animation after 2 seconds
        setTimeout(() => {
          showSuccessAnimation = false
        }, 2000)

        Option.match(dashboard.airdrop, {
          onNone: () => {},
          onSome: (store) => store.refresh(),
        })
        shouldVerify = false
        return
      }

      if (Option.isSome(result.nextState)) {
        currentStateLocal = result.nextState
      } else {
        currentStateLocal = Option.none()
      }
    }
  })

  runPromise(program)
}

function connectCosmos() {
  uiStore.openWalletModal()
}

function startVerification() {
  if (!isWalletConnected) {
    message = "Please connect your wallet first"
    hasError = true
    return
  }

  if (shouldVerify) {
    return // Already running
  }

  if (showAddressMismatch) {
    message = "You're connected to a different address. This will update your verified wallet."
  }

  hasError = false
  if (!showAddressMismatch) {
    message = "Preparing verification..."
  }

  shouldVerify = true
  startVerificationProcess()
}

function retry() {
  hasError = false
  message = ""
  currentState = Option.none()
  startVerification()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-3 sm:p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-semibold">
              Add Cosmos Wallet
            </h1>
            <div class="px-2 py-1 bg-accent/20 border border-accent/30 rounded-md flex items-center justify-center">
              <span class="text-xs font-medium text-accent uppercase">Optional</span>
            </div>
          </div>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect your Cosmos wallet to verify Wandering Whale Shark NFT ownership.
          </p>
        </div>
      </div>

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="space-y-3">
        <!-- Wallet Connection Status -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800 cursor-pointer hover:bg-zinc-900/50 transition-colors"
          onclick={connectCosmos}
        >
          {#if hasVerifiedWallet}
            <!-- Verified Stargaze wallet from database -->
            <div class="space-y-2">
              <div class="flex gap-3">
                <!-- Wallet Icon -->
                {#if isWalletConnected && Option.isSome(connectedCosmosWallet)}
                  <img
                    src={connectedCosmosWallet.value.icon}
                    alt="{connectedCosmosWallet.value.name ?? 'Cosmos Wallet'} icon"
                    class="size-8 rounded-lg flex-shrink-0"
                  />
                {:else}
                  <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                    <svg
                      class="w-4 h-4 text-zinc-400"
                      fill="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                    </svg>
                  </div>
                {/if}

                <!-- Wallet name and address -->
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-white">
                    {#if isWalletConnected && Option.isSome(connectedCosmosWallet)}
                      {connectedCosmosWallet.value.name}
                    {:else}
                      Verified Stargaze Wallet
                    {/if}
                  </div>
                  <div class="text-xs text-zinc-400 font-mono">
                    {#if hasStoredAddress}
                      {
                        Option.match(storedStargazeAddress, {
                          onNone: () => "No address",
                          onSome: (addr) => `${addr.slice(0, 6)}...${addr.slice(-4)}`,
                        })
                      }
                    {:else}
                      No address
                    {/if}
                    {#if !isWalletConnected}
                      <span class="text-zinc-500 ml-1">(not connected)</span>
                    {/if}
                  </div>
                </div>

                <!-- Status indicator -->
                <div class="flex items-center">
                  {#if isVerifying}
                    <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                    </div>
                  {:else if showAddressMismatch}
                    <div class="w-4 h-4 bg-amber-400/20 border border-amber-400/40 rounded-full flex items-center justify-center">
                      <svg
                        class="w-2.5 h-2.5 text-amber-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                        />
                      </svg>
                    </div>
                  {:else}
                    <div class="w-4 h-4 bg-accent/20 border border-accent/40 rounded-full flex items-center justify-center">
                      <svg
                        class="w-2.5 h-2.5 text-accent"
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
                  {/if}
                </div>
              </div>

              <!-- Messages and warnings -->
              {#if showAddressMismatch || message}
                <div class="space-y-1 pl-0">
                  {#if showAddressMismatch}
                    <div class="text-xs text-amber-400">
                      ⚠️ Connected wallet differs from verified address
                    </div>
                  {/if}
                  {#if message}
                    <div class="text-xs {hasError ? 'text-red-400' : isVerifying ? 'text-accent' : hasStargazeWallet ? 'text-accent' : 'text-accent'}">
                      {message}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {:else}
            <!-- No wallet verified yet, but may be connected -->
            <div class="flex items-center gap-3">
              <!-- Show connected wallet icon if available, otherwise generic icon -->
              {#if isWalletConnected && Option.isSome(connectedCosmosWallet)}
                <img
                  src={connectedCosmosWallet.value.icon}
                  alt="{connectedCosmosWallet.value.name ?? 'Cosmos Wallet'} icon"
                  class="size-8 rounded-lg flex-shrink-0"
                />
              {:else}
                <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                  <svg
                    class="w-4 h-4 text-zinc-400"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                  </svg>
                </div>
              {/if}
              <div>
                <div class="text-sm font-medium text-zinc-400">
                  {#if isWalletConnected && Option.isSome(connectedCosmosWallet)}
                    {connectedCosmosWallet.value.name}
                  {:else if isWalletConnected}
                    Wallet connected
                  {:else}
                    No wallet connected
                  {/if}
                </div>
                <div class="text-xs text-zinc-500">
                  {#if isVerifying}
                    <div class="flex items-center gap-2">
                      <div class="w-3 h-3 border-2 border-accent border-t-transparent rounded-full animate-spin">
                      </div>
                      <span class="text-accent">
                        {
                          Option.getOrElse(verificationStepMessage, () =>
                          "Verifying wallet ownership...")
                        }
                      </span>
                    </div>
                  {:else if isWalletConnected}
                    {#if Option.isSome(stargazeDisplayAddress)}
                      {
                        Option.match(stargazeDisplayAddress, {
                          onNone: () => "Verify your wallet to continue",
                          onSome: (address) =>
                            `${address.slice(0, 6)}...${
                              address.slice(-4)
                            } • Verify to continue`,
                        })
                      }
                    {:else}
                      Verify your wallet to continue
                    {/if}
                  {:else}
                    Connect your Cosmos wallet to continue
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Action Buttons Row -->
        <div class="flex gap-3">
          {#if hasVerifiedWallet && !showAddressMismatch}
            <!-- User completed the optional step and no mismatch -->
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={onNext}
            >
              Continue
            </Button>
          {:else}
            <!-- User hasn't completed optional step - encourage action but allow skip -->
            {#if !isWalletConnected}
              <Button
                variant="primary"
                class="flex flex-1 items-center justify-center gap-3"
                onclick={connectCosmos}
              >
                Connect
              </Button>
            {:else if hasError}
              <Button
                variant="danger"
                class="flex flex-1 items-center justify-center gap-3"
                onclick={retry}
              >
                Retry
              </Button>
            {:else if !hasStargazeWallet || showAddressMismatch}
              <Button
                variant="primary"
                class="flex flex-1 items-center justify-center gap-3"
                onclick={startVerification}
                disabled={isVerifying}
              >
                {#if isVerifying}
                  <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                  </div>
                  Verifying...
                {:else if showAddressMismatch}
                  Update Wallet
                {:else}
                  Verify Wallet
                {/if}
              </Button>
            {/if}

            <Button
              variant="secondary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={onSkip}
            >
              Skip
            </Button>
          {/if}
        </div>

        <!-- Back button always at bottom -->
        {#if onBack}
          <Button
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onBack}
          >
            ← Back
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-3 sm:p-4">
      <!-- Mobile Title - shown above the content on mobile -->
      <div class="block lg:hidden mb-4 px-1">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-semibold">
            Add Cosmos Wallet
          </h1>
          <div class="px-2 py-1 bg-accent/20 border border-accent/30 rounded-md flex items-center justify-center">
            <span class="text-xs font-medium text-accent uppercase">Optional</span>
          </div>
        </div>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Connect your Cosmos wallet to verify Wandering Whale Sharks NFT ownership.
        </p>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <img
          src="/airdrop/wws.png"
          alt="WWS"
          class="w-full h-full object-cover"
        />
      </div>
    </div>
  {/snippet}
</StepLayout>
