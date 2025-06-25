<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StepLayout from "../StepLayout.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { cosmosStore, cosmosWalletsInformation } from "$lib/wallet/cosmos/config.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "@unionlabs/sdk/schema"
import { UniversalChainId } from "@unionlabs/sdk/schema"
import { runPromise } from "$lib/runtime"
import { Effect, Match, Option } from "effect"
import { signMessageForClaim, SignMessageState, type StateResult } from "../services/sign-message"

interface Props {
  onNext: () => void
}

let { onNext }: Props = $props()

// Drawer state
let showBreakdown = $state(false)

// Claiming state
let isProcessing = $state(false)
let currentState: SignMessageState | null = $state(null)
let message = $state("")
let hasError = $state(false)
let error = $state<Option.Option<Error>>(Option.none())
let claimCompleted = $state(false)

const totalAmount = 15842
const buckets = [
  { name: "WWS", eligible: true, amount: 5000 },
  { name: "Sloths", eligible: true, amount: 3500 },
  { name: "Kaito", eligible: false, amount: 0 },
  { name: "TSC", eligible: true, amount: 7342 },
]

const connectedCosmosWallet = $derived.by(() => {
  if (
    cosmosStore.connectionStatus === "connected" && cosmosStore.connectedWallet
    && cosmosWalletsInformation
  ) {
    return cosmosWalletsInformation.find((wallet: { id: string }) =>
      wallet.id === cosmosStore.connectedWallet
    )
  }
  return null
})

const statusMessage = $derived(
  hasError 
    ? (message || "An error occurred")
    : isProcessing 
    ? (message || "Processing...")
    : claimCompleted 
    ? "Airdrop claimed successfully!" 
    : "Ready to claim"
)

const canClaim = $derived(
  cosmosStore.connectionStatus === "connected" && !isProcessing && !claimCompleted
)

function connectWallet() {
  uiStore.openWalletModal()
}

function claimAirdrop() {
  if (!canClaim) return

  isProcessing = true
  hasError = false
  error = Option.none()
  message = "Initiating claim process..."

  const program = Effect.gen(function*() {
    if (Option.isNone(chains.data)) {
      message = "Chain data not available."
      hasError = true
      isProcessing = false
      return
    }

    const targetChain = getChain(
      chains.data.value,
      UniversalChainId.make("babylon.bbn-test-5"),
    )

    if (Option.isNone(targetChain)) {
      message = "Required chain not found in configuration."
      hasError = true
      isProcessing = false
      return
    }

    currentState = SignMessageState.SwitchChain({ chain: targetChain.value })
    message = "Switching to Union testnet..."

    while (currentState !== null) {
      const result: StateResult = yield* signMessageForClaim(currentState)

      if (Option.isSome(result.error)) {
        hasError = true
        error = result.error
        currentState = null
        isProcessing = false
        // Set a more descriptive message based on the error
        const err = result.error.value
        if ('operation' in err) {
          switch (err.operation) {
            case 'switchChain':
              message = "Failed to switch to Union network. Please try again."
              break
            case 'sign':
              message = "Transaction was rejected or signing failed. Please try again."
              break
            case 'claim':
              message = "Claim transaction failed. Please check your connection and try again."
              break
            default:
              message = result.message || "An error occurred during claiming."
          }
        } else {
          message = result.message || "An error occurred during claiming."
        }
        return
      }

      message = result.message
      if (Option.isSome(result.nextState)) {
        currentState = result.nextState.value
      } else {
        currentState = null
        claimCompleted = true
        isProcessing = false
      }
    }
  })

  runPromise(program)
}

function reset() {
  isProcessing = false
  currentState = null
  message = ""
  hasError = false
  error = Option.none()
  claimCompleted = false
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4">
        <div>
          <h1 class="text-2xl font-semibold">
            {claimCompleted ? "Claim Successful!" : "Congratulations!"}
          </h1>
          <p class="text-gray-400 mt-2">
            {claimCompleted 
              ? "Your airdrop has been claimed successfully" 
              : "You're eligible for airdrop!"
            }
          </p>
        </div>


      </div>
      
      <div class="space-y-3">
        <!-- Error Display (only show when error) -->
        {#if hasError}
          <div class="px-3 py-2 bg-red-500/10 border border-red-500/30 rounded-lg">
            <div class="flex items-center gap-3">
              <div class="w-4 h-4 bg-red-500/20 border border-red-500/40 rounded-full flex items-center justify-center flex-shrink-0">
                <svg class="w-2 h-2 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="text-sm font-mono text-red-400">
                {statusMessage}
              </div>
            </div>
          </div>
        {/if}
        <!-- Wallet Status Display -->
        <div class="flex items-center gap-3 px-3 py-2 bg-zinc-950/50 border border-zinc-800/60 rounded-lg">
            {#if cosmosStore.connectionStatus === "connected" && connectedCosmosWallet}
              {#if connectedCosmosWallet.icon}
                <img
                  src={connectedCosmosWallet.icon}
                  alt="{connectedCosmosWallet.name ?? 'Cosmos Wallet'} icon"
                  class="w-5 h-5 rounded flex-shrink-0"
                />
              {/if}
              <div class="text-sm text-zinc-300 font-mono">
                {connectedCosmosWallet.name}
                <span class="text-zinc-500 ml-2">
                  {cosmosStore.address ? `${cosmosStore.address.slice(0, 6)}...${cosmosStore.address.slice(-4)}` : ""}
                </span>
              </div>
              
              <!-- Status indicator on the right -->
              {#if isProcessing}
                <div class="ml-auto w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin flex-shrink-0"></div>
              {:else if hasError}
                <div class="ml-auto w-4 h-4 bg-red-500/20 border border-red-500/40 rounded-full flex items-center justify-center flex-shrink-0">
                  <svg class="w-2 h-2 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                  </svg>
                </div>
              {:else if claimCompleted}
                <div class="ml-auto w-4 h-4 bg-accent/20 border border-accent/40 rounded-full flex items-center justify-center flex-shrink-0">
                  <svg class="w-2 h-2 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                </div>
              {/if}
            {:else}
              <div class="w-5 h-5 rounded bg-zinc-700 flex items-center justify-center flex-shrink-0">
                <svg class="w-3 h-3 text-zinc-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M4 4a2 2 0 00-2 2v4a2 2 0 002 2V6h10a2 2 0 00-2-2H4zm2 6a2 2 0 012-2h8a2 2 0 012 2v4a2 2 0 01-2 2H8a2 2 0 01-2-2v-4zm6 4a2 2 0 100-4 2 2 0 000 4z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="text-sm text-zinc-400 font-mono">
                No wallet connected
              </div>
                        {/if}
        </div>

        {#if claimCompleted}
          <Button
            size="md"
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onNext}
          >
            Continue to Staking
          </Button>
        {:else if cosmosStore.connectionStatus !== "connected"}
          <Button
            size="md"
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={connectWallet}
          >
            Connect Wallet
          </Button>
        {:else}
          <Button
            size="md"
            variant={hasError ? "danger" : "primary"}
            class="flex w-full items-center justify-center gap-3"
            onclick={hasError ? () => { reset(); claimAirdrop(); } : claimAirdrop}
            disabled={!canClaim}
          >
            {hasError ? "Retry Claim" : isProcessing ? "Processing..." : "Claim Airdrop"}
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}
  
  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-3">
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        
        <!-- Union Logo -->
        <div class="absolute top-3 left-3 z-20">
          <img 
            src="/images/union-logo-glyph.svg" 
            alt="Union logo" 
            class="w-6 h-6 opacity-60"
          />
        </div>

        <!-- Info Icon (only show if not processing or completed) -->
        {#if !isProcessing && !claimCompleted}
          <button 
            class="absolute top-3 right-3 z-40 w-8 h-8 flex items-center justify-center rounded-full bg-zinc-900/80 backdrop-blur-sm border border-zinc-700/60 hover:bg-zinc-800/80 transition-colors"
            onclick={() => showBreakdown = !showBreakdown}
          >
            <svg class="w-4 h-4 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10"></circle>
              <path d="M9,9h0a3,3,0,0,1,5.12-2.12A3,3,0,0,1,15,9c0,1-3,3-3,3"></path>
              <circle cx="12" cy="17" r=".5"></circle>
            </svg>
          </button>
        {/if}

        <!-- Main Content Area -->
        <div class="relative flex-1 flex flex-col items-center justify-center p-6 overflow-hidden">
          {#if Option.isSome(dashboard.identity.avatar)}
            <!-- Blurred background -->
            <div class="absolute inset-0">
              <img
                src={dashboard.identity.avatar.value}
                alt=""
                class="w-full h-full object-cover blur-xl opacity-70 scale-110"
              />
              <div class="absolute inset-0 bg-zinc-950/40"></div>
            </div>
          {/if}
          
          <div class="text-center relative z-10">
            {#if claimCompleted}
              <!-- Success State -->
              <div class="text-xs text-zinc-400 font-mono uppercase tracking-wider mb-2 flex items-center justify-center gap-2">
                <svg class="w-3 h-3 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
                Claimed
              </div>
              <div class="text-5xl font-bold text-white mb-2">
                {totalAmount.toLocaleString()}
              </div>
              <div class="text-lg text-zinc-300 font-mono">
                $U
              </div>
            {:else}
              <!-- Rewards Display -->
              <div class="text-xs text-zinc-400 font-mono uppercase tracking-wider mb-2">
                Total Allocation
              </div>
              <div class="text-5xl font-bold text-white mb-2">
                {totalAmount.toLocaleString()}
              </div>
              <div class="text-lg text-zinc-300 font-mono">
                $U
              </div>
            {/if}
          </div>
        </div>

        <!-- Breakdown Drawer (only show if not processing/completed) -->
        {#if showBreakdown && !isProcessing && !claimCompleted}
          <div class="absolute inset-0 z-30 flex items-end">
            <div class="w-full bg-black/80 backdrop-blur-md border-t border-zinc-700/60 px-4 py-3 drawer-slide-in">
              <div class="text-xs text-zinc-400 font-mono uppercase tracking-wider mb-3 text-center">
                Eligibility Breakdown
              </div>
              
              <div class="grid grid-cols-2 gap-2">
                {#each buckets as bucket}
                  <div class="flex items-center justify-between bg-zinc-900/50 rounded-lg px-3 py-2 border border-zinc-800">
                    <span class="text-sm font-mono text-zinc-200">{bucket.name}</span>
                    {#if bucket.eligible}
                      <span class="text-accent text-sm">✓</span>
                    {:else}
                      <span class="text-red-400 text-sm">✗</span>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
</StepLayout>

<style>
  /* Drawer Animation */
  .drawer-slide-in {
    animation: drawerSlideIn 300ms ease-out forwards;
  }

  @keyframes drawerSlideIn {
    from {
      transform: translateY(100%);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style> 