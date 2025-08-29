<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import {
  addAirdropWallet,
  AddAirdropWalletState,
  type AirdropWalletResult,
} from "$lib/dashboard/services/add-airdrop-wallet"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { uiStore } from "$lib/stores/ui.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import { evmWalletsInformation, sepoliaStore } from "$lib/wallet/evm/config.svelte"
import { createViemWalletClient } from "@unionlabs/sdk/evm"
import { Effect, Option, pipe } from "effect"
import { custom } from "viem"
import { mainnet } from "viem/chains"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

let shouldVerify = $state(false)
let message = $state("")
let hasError = $state(false)
let currentState = $state(Option.none<AddAirdropWalletState>())
let walletConnected = $state(false)
let showSuccessAnimation = $state(false)

// Use mainnet for production, sepolia for testing
const targetChain = mainnet
const chainId = targetChain.id === 1 ? "1" : "11155111"

const connectedEvmWallet = $derived(
  sepoliaStore.connectionStatus === "connected" && sepoliaStore.connectedWallet
    && evmWalletsInformation
    ? Option.fromNullable(
      evmWalletsInformation.find((wallet: { id: string }) =>
        wallet.id === sepoliaStore.connectedWallet
      ),
    )
    : Option.none(),
)

// Get the stored EVM address from database
const storedEvmAddress = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => Option.none(),
    onSome: (store) =>
      Option.match(store.allocation, {
        onNone: () => Option.none(),
        onSome: (alloc) => Option.fromNullable(alloc.evm_wallet),
      }),
  }),
)

// Get the current connected EVM address from wallets store (display address)
const connectedEvmAddress = $derived(
  Option.map(wallets.evmAddress, (evmDisplay) => evmDisplay.address),
)

// Check if current connected address matches stored address (only when both exist)
const addressMatches = $derived(
  Option.isNone(connectedEvmAddress) || Option.isNone(storedEvmAddress)
    ? true // No comparison needed if either is missing
    : connectedEvmAddress.value.toLowerCase() === storedEvmAddress.value.toLowerCase(),
)

// Show address mismatch warning when user has verified wallet but connected to different address
const showAddressMismatch = $derived(
  sepoliaStore.connectionStatus === "connected"
    && Option.isSome(storedEvmAddress)
    && !addressMatches,
)

const hasEvmWallet = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.hasEvmWallet,
  }),
)

// Simplified derived states following WalletDialog pattern
const isWalletConnected = $derived(sepoliaStore.connectionStatus === "connected")
const hasStoredAddress = $derived(Option.isSome(storedEvmAddress))
const hasVerifiedWallet = $derived(hasEvmWallet && hasStoredAddress)

// Derived state for button loading
const isVerifying = $derived(shouldVerify)

// Verification progress message
const verificationStepMessage = $derived(
  !isVerifying
    ? Option.none()
    : Option.some(Option.match(currentState, {
      onNone: () => "Preparing verification...",
      onSome: (state) =>
        AddAirdropWalletState.$match(state, {
          SwitchChain: () => "Please switch to the correct network in your wallet",
          Signing: () => "Please sign the message in your wallet",
          Verifying: () => "Submitting verification to server...",
        }),
    })),
)

const verificationResult = runPromiseExit$(() =>
  shouldVerify
    ? Effect.gen(function*() {
      const connectorClient = yield* getWagmiConnectorClient
      const walletClient = yield* createViemWalletClient({
        account: connectorClient.account,
        chain: targetChain,
        transport: custom(connectorClient),
      })

      // Start with chain switching to ensure user is on correct network
      currentState = Option.some(AddAirdropWalletState.SwitchChain({
        targetChainId: chainId as "1" | "11155111",
      }))

      while (Option.isSome(currentState)) {
        const result: AirdropWalletResult = yield* addAirdropWallet(
          currentState.value,
          walletClient,
        )

        if (Option.isSome(result.error)) {
          message = result.message
          hasError = true
          currentState = Option.none()
          shouldVerify = false
          return
        }

        message = result.message

        if (result.completed) {
          currentState = Option.none()
          walletConnected = true
          showSuccessAnimation = true

          // Hide success animation after 2 seconds
          setTimeout(() => {
            showSuccessAnimation = false
          }, 2000)

          // Manually refresh allocation data to update hasEvmWallet
          Option.match(dashboard.airdrop, {
            onNone: () => {},
            onSome: (store) => store.refresh(),
          })
          shouldVerify = false
          return
        }

        if (Option.isSome(result.nextState)) {
          currentState = result.nextState
        } else {
          currentState = Option.none()
        }
      }

      shouldVerify = false
    })
    : Effect.void
)

function connectWallet() {
  uiStore.openWalletModal()
}

function startVerification() {
  if (sepoliaStore.connectionStatus !== "connected") {
    message = "Please connect your wallet first"
    hasError = true
    return
  }

  if (Option.isNone(verificationResult.current)) {
    return // Already running
  }

  // Handle address mismatch case
  if (showAddressMismatch) {
    message = "You're connected to a different address. This will update your verified wallet."
    // Continue with verification to update the stored address
  }

  hasError = false
  if (!showAddressMismatch) {
    message = "Preparing verification..."
  }

  shouldVerify = true
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
              Add EVM Wallet
            </h1>
            <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
              <span class="text-xs font-medium text-red-400 uppercase">Required</span>
            </div>
          </div>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect your Ethereum wallet to verify your humanity and claim rewards.
          </p>
        </div>
      </div>

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="space-y-3">
        <!-- Wallet Connection Status -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800 cursor-pointer hover:bg-zinc-900/50 transition-colors"
          onclick={connectWallet}
        >
          {#if hasVerifiedWallet}
            <!-- Verified EVM wallet from database -->
            <div class="space-y-2">
              <div class="flex gap-3">
                <!-- Wallet Icon -->
                {#if isWalletConnected && Option.isSome(connectedEvmWallet)}
                  <img
                    src={connectedEvmWallet.value.icon}
                    alt="{connectedEvmWallet.value.name ?? 'EVM Wallet'} icon"
                    class="size-8 rounded-lg bg-white p-1 flex-shrink-0"
                  />
                {:else}
                  <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                    <svg
                      class="w-4 h-4 text-zinc-400"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M4 4a2 2 0 00-2 2v4a2 2 0 002 2V6h10a2 2 0 00-2-2H4zm2 6a2 2 0 012-2h8a2 2 0 012 2v4a2 2 0 01-2 2H8a2 2 0 01-2-2v-4zm6 4a2 2 0 100-4 2 2 0 000 4z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </div>
                {/if}

                <!-- Wallet name and address -->
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-white">
                    {#if isWalletConnected && Option.isSome(connectedEvmWallet)}
                      {connectedEvmWallet.value.name}
                    {:else}
                      Verified EVM Wallet
                    {/if}
                  </div>
                  <div class="text-xs text-zinc-400 font-mono">
                    {#if hasStoredAddress}
                      {
                        Option.match(storedEvmAddress, {
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
                    <div class="text-xs {hasError ? 'text-red-400' : isVerifying ? 'text-blue-400' : hasEvmWallet ? 'text-accent' : 'text-blue-400'}">
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
              {#if isWalletConnected && Option.isSome(connectedEvmWallet)}
                <img
                  src={connectedEvmWallet.value.icon}
                  alt="{connectedEvmWallet.value.name ?? 'EVM Wallet'} icon"
                  class="size-8 rounded-lg bg-white p-1 flex-shrink-0"
                />
              {:else}
                <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                  <svg
                    class="w-4 h-4 text-zinc-400"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M4 4a2 2 0 00-2 2v4a2 2 0 002 2V6h10a2 2 0 00-2-2H4zm2 6a2 2 0 012-2h8a2 2 0 012 2v4a2 2 0 01-2 2H8a2 2 0 01-2-2v-4zm6 4a2 2 0 100-4 2 2 0 000 4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
              {/if}
              <div>
                <div class="text-sm font-medium text-zinc-400">
                  {#if isWalletConnected && Option.isSome(connectedEvmWallet)}
                    {connectedEvmWallet.value.name}
                  {:else if isWalletConnected}
                    Wallet connected
                  {:else}
                    No wallet connected
                  {/if}
                </div>
                <div class="text-xs text-zinc-500">
                  {#if isVerifying}
                    <div class="flex items-center gap-2">
                      <div class="w-3 h-3 border-2 border-blue-400 border-t-transparent rounded-full animate-spin">
                      </div>
                      <span class="text-blue-400">
                        {
                          Option.getOrElse(verificationStepMessage, () =>
                          "Verifying wallet ownership...")
                        }
                      </span>
                    </div>
                  {:else if isWalletConnected}
                    {#if sepoliaStore.address}
                      {
                        `${sepoliaStore.address.slice(0, 6)}...${
                          sepoliaStore.address.slice(-4)
                        }`
                      } • Verify to continue
                    {:else}
                      Verify your wallet to continue
                    {/if}
                  {:else}
                    Connect your EVM wallet to continue
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Action Buttons Row -->
        <div class="flex gap-3">
          {#if !isWalletConnected}
            <Button
              variant={hasVerifiedWallet ? "secondary" : "primary"}
              class="flex flex-1 items-center justify-center gap-3"
              onclick={connectWallet}
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
          {:else if !hasEvmWallet || showAddressMismatch}
            <Button
              variant={showAddressMismatch ? "secondary" : "primary"}
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
            variant="primary"
            class="flex flex-1 items-center justify-center gap-3"
            disabled={!walletConnected && !hasEvmWallet}
            onclick={() => {
              if (!walletConnected && !hasEvmWallet) {
                console.log("Continue blocked: EVM wallet verification required")
                return
              }
              onNext()
            }}
          >
            Continue
          </Button>
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
            Add Wallet
          </h1>
          <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
            <span class="text-xs font-medium text-red-400 uppercase">Required</span>
          </div>
        </div>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Connect your Ethereum wallet to verify your humanity and claim rewards.
        </p>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Ethereum Logo as full background -->
        <div class="w-full h-full flex items-center justify-center {hasVerifiedWallet ? 'bg-gradient-to-br from-accent/10 to-accent/20' : 'bg-gradient-to-br from-blue-600/20 to-purple-600/20'}">
          <svg
            class="w-32 h-32 {hasVerifiedWallet ? 'text-accent' : 'text-white'} opacity-80"
            fill="currentColor"
            viewBox="0 0 24 24"
          >
            <path d="M12 1.75L5.75 12.25L12 16L18.25 12.25L12 1.75ZM5.75 13.5L12 22.25L18.25 13.5L12 17.25L5.75 13.5Z" />
          </svg>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
