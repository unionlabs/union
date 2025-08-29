<script lang="ts">
import { goto } from "$app/navigation"
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { Effect, Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onRestart?: () => void
  onBack?: () => void
}

let { onRestart, onBack }: Props = $props()

let shouldVerify = $state(false)
let hasError = $state(false)
let verificationComplete = $state(false)
let message = $state("")

const isLoadingAllocation = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => true,
    onSome: (store) => store.isLoadingAllocation,
  }),
)

const isAlreadyVerified = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) =>
      Option.match(store.allocation, {
        onNone: () => false,
        onSome: (alloc) => alloc.is_human,
      }),
  }),
)

function openAuthenaWebsite() {
  window.open("https://authena.xyz/union", "_blank")
}

// Auto-advance if already verified
$effect(() => {
  if (isAlreadyVerified && !isLoadingAllocation) {
    verificationComplete = true
  }
})

// Use runPromiseExit$ for automatic loading state tracking
const verificationResult = runPromiseExit$(() => {
  // Never run verification if user is already verified or data is loading
  if (isAlreadyVerified || isLoadingAllocation || !shouldVerify) {
    return Effect.void
  }

  return Effect.gen(function*() {
    hasError = false
    message = "Checking your humanity with Authena..."

    const success = yield* Option.match(dashboard.airdrop, {
      onNone: () => Effect.succeed(false),
      onSome: (store) => Effect.tryPromise(() => store.verifyHumanWithAuthena()),
    })

    if (success) {
      message = "Human verification successful!"
      verificationComplete = true
    } else {
      message = "Verification failed. Please try again or visit Authena directly."
      hasError = true
    }

    shouldVerify = false
  })
})

// Derived state for button loading (matches other steps pattern)
const isVerifying = $derived(shouldVerify)

function verifyHuman() {
  if (isVerifying || isLoadingAllocation || isAlreadyVerified) {
    return // Don't do anything if already running, loading, or verified
  }

  hasError = false
  message = ""
  shouldVerify = true
}

function retry() {
  hasError = false
  message = ""
  verifyHuman()
}

function continueToDashboard() {
  goto("/udrop")
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-3 sm:p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-semibold">
              Prove Your Humanity
            </h1>
            <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
              <span class="text-xs font-medium text-red-400 uppercase">Required</span>
            </div>
          </div>
          <div class="text-sm text-zinc-400 leading-relaxed mt-3 space-y-2">
            <p>Verify that you're a real person using Authena's Proof of Humanity system.</p>
            <ol class="list-decimal list-inside space-y-1 text-xs">
              <li>Click Visit Authena Website below.</li>
              <li>Follow the process on Authena's website.</li>
              <li>Reach XX Humanity Points to be eligible.</li>
              <li>If eligible, mint your Union Proof of Humanity NFT on Base.</li>
              <li>Return to this page and click Verify with Authena.</li>
            </ol>
            <p class="text-xs text-red-400 font-medium">
              You will not be able to claim your allocation without completing this step.
            </p>
          </div>
        </div>
      </div>

      <!-- Verification Status & Action Buttons -->
      <div class="space-y-3">
        <!-- Verification Status -->
        <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
          {#if verificationComplete || isAlreadyVerified}
            <div class="space-y-2">
              <!-- Main status row -->
              <div class="flex gap-3">
                <!-- Icon -->
                <div class="size-8 rounded-lg bg-accent/20 border border-accent/40 flex items-center justify-center flex-shrink-0">
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
                      d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                    />
                  </svg>
                </div>

                <!-- Status info -->
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-white">Human Verification</div>
                  <div class="text-xs text-zinc-400 font-mono">Verified</div>
                </div>

                <!-- Status indicator -->
                <div class="flex items-center">
                  <div class="w-4 h-4 bg-accent/20 border border-accent/40 rounded-full flex items-center justify-center">
                    <svg
                      class="w-2 h-2 text-accent"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="3"
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <div class="space-y-2">
              <!-- Main status row -->
              <div class="flex gap-3">
                <!-- Icon -->
                <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                  <svg
                    class="w-4 h-4 text-zinc-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                    />
                  </svg>
                </div>

                <!-- Status info -->
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-zinc-400">Human Verification</div>
                  <div class="text-xs text-zinc-500">
                    Complete Authena verification to continue
                  </div>
                </div>

                <!-- Status indicator -->
                <div class="flex items-center">
                  {#if isVerifying}
                    <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                    </div>
                  {:else}
                    <div class="w-4 h-4 bg-zinc-600/20 border border-zinc-600/40 rounded-full flex items-center justify-center">
                      <div class="w-2 h-2 bg-zinc-500 rounded-full"></div>
                    </div>
                  {/if}
                </div>
              </div>

              <!-- Status messages - full width row -->
              {#if message}
                <div class="space-y-1 pl-0">
                  <div class="text-xs {hasError ? 'text-red-400' : verificationComplete ? 'text-accent' : 'text-accent'}">
                    {message}
                  </div>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Action Buttons -->
        {#if verificationComplete || isAlreadyVerified}
          <!-- Verification complete - go to dashboard -->
          <Button
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={continueToDashboard}
          >
            Continue to Dashboard
          </Button>
        {:else}
          <!-- Not verified yet - show verification options -->
          <Button
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={openAuthenaWebsite}
          >
            Visit Authena Website
          </Button>

          <Button
            variant={hasError ? "danger" : "primary"}
            class="flex w-full items-center justify-center gap-3"
            onclick={hasError ? retry : verifyHuman}
            disabled={isVerifying || isAlreadyVerified || isLoadingAllocation}
          >
            {#if isVerifying}
              <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
              </div>
              Loading
            {:else if isLoadingAllocation}
              <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
              </div>
              Loading Status...
            {:else if isAlreadyVerified}
              Already Verified
            {:else if hasError}
              Retry Verification
            {:else}
              Verify with Authena
            {/if}
          </Button>
        {/if}

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
            Prove Your Humanity
          </h1>
          <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
            <span class="text-xs font-medium text-red-400 uppercase">Required</span>
          </div>
        </div>
        <div class="text-sm text-zinc-400 leading-relaxed mt-3 space-y-2">
          <p>Verify that you're a real person using Authena's Proof of Humanity system.</p>
          <ol class="list-decimal list-inside space-y-1 text-xs">
            <li>Click Visit Authena Website below.</li>
            <li>Follow the process on Authena's website.</li>
            <li>Reach 40 Humanity Points to be eligible.</li>
            <li>
              If eligible, mint your Union Proof of Humanity NFT on on Base with the EVM wallet you
              connected earlier in the process..
            </li>
            <li>Return to this page and click Verify with Authena.</li>
          </ol>
        </div>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex-col relative hidden sm:flex">
        <!-- Main Content Area -->
        <div class="relative flex-1 flex flex-col items-center justify-center p-6 overflow-hidden">
          <!-- Background -->
          <div class="absolute inset-0">
            <div class="w-full h-full bg-zinc-925"></div>
            {#if verificationComplete || isAlreadyVerified}
              <!-- Skull background image for verified state -->
              <div class="absolute inset-0 bg-[url('/images/skull.png')] grayscale bg-center bg-no-repeat bg-contain opacity-5">
              </div>
            {/if}
          </div>

          <div class="text-center relative z-10">
            {#if verificationComplete || isAlreadyVerified}
              <!-- Success State -->
              <div class="flex flex-col items-center justify-center space-y-6">
                <div class="flex items-center justify-center">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-8 w-auto text-white"
                    viewBox="0 0 565.22 93.21"
                  >
                    <defs>
                      <style>
                      .cls-1 {
                        fill: #fff;
                      }
                      </style>
                    </defs>
                    <g
                      id="Layer_1-2"
                      data-name="Layer 1"
                    >
                      <g>
                        <path
                          class="cls-1"
                          d="M129.3,74.31l27.31-56.54h9.45l26.27,56.54h-9.13l-6.97-14.74h-30.75l-7.13,14.74h-9.05ZM173.59,53.01c-7.85-16.9-10.25-22.1-12.57-27.63-2.48,5.45-4.72,10.57-12.97,27.63h25.55Z"
                        />
                        <path
                          class="cls-1"
                          d="M249.76,49.65c0,9.29-1.04,14.42-4.89,18.58-4.24,4.65-11.93,7.21-21.86,7.21s-17.86-2.64-21.7-7.53c-3.84-4.81-4.4-9.85-4.4-18.34v-31.8h8.17v33.08c0,4.97.64,8.97,3.04,12.09,2.72,3.6,8.65,5.37,15.22,5.37s12.65-1.52,15.46-5.29c2.32-3.04,2.8-7.21,2.8-11.85V17.77h8.17v31.88Z"
                        />
                        <path
                          class="cls-1"
                          d="M278.75,74.31V24.82h-20.98v-7.05h50.54v7.05h-21.06v49.5h-8.49Z"
                        />
                        <path
                          class="cls-1"
                          d="M316.71,74.31V17.77h8.41v23.63h37.16v-23.63h8.49v56.54h-8.49v-25.79h-37.16v25.79h-8.41Z"
                        />
                        <path
                          class="cls-1"
                          d="M383.26,74.31V17.77h46.05v7.05h-37.72v16.82h34.44v6.97h-34.44v18.66h39.97v7.05h-48.29Z"
                        />
                        <path
                          class="cls-1"
                          d="M441,74.31V17.77h9.45c27.95,33.88,32.6,39.4,37.4,45.65-.24-7.05-.4-14.26-.4-45.65h8.41v56.54h-9.21c-27.71-33.64-31.32-38.04-37.48-45.89.08,5.77.16,12.41.16,45.89h-8.33Z"
                        />
                        <path
                          class="cls-1"
                          d="M502.19,74.31l27.31-56.54h9.45l26.27,56.54h-9.13l-6.97-14.74h-30.75l-7.13,14.74h-9.05ZM546.48,53.01c-7.85-16.9-10.25-22.1-12.57-27.63-2.48,5.45-4.72,10.57-12.97,27.63h25.55Z"
                        />
                        <path
                          class="cls-1"
                          d="M56.71,93.21H0S28.01,5.99,28.01,5.99l28.7,87.22ZM108.43,93.21h-47.14S30.51,0,30.51,0h47.28s30.64,93.21,30.64,93.21Z"
                        />
                      </g>
                    </g>
                  </svg>
                </div>
                <div class="text-center space-y-2">
                  <div class="text-xs text-zinc-500 font-mono uppercase tracking-wider">
                    Human Verification
                  </div>
                  <div class="text-xs text-zinc-500 font-mono uppercase tracking-wider">
                    Successfully Verified
                  </div>
                  <div class="text-xs text-accent font-mono uppercase tracking-wider">
                    You are not Jessica
                  </div>
                </div>
              </div>
            {:else}
              <!-- Unverified State - Partner Logo -->
              <div class="flex flex-col items-center justify-center space-y-6">
                <div class="flex items-center justify-center">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-8 w-auto text-white"
                    viewBox="0 0 565.22 93.21"
                  >
                    <defs>
                      <style>
                      .cls-1 {
                        fill: #fff;
                      }
                      </style>
                    </defs>
                    <g
                      id="Layer_1-2"
                      data-name="Layer 1"
                    >
                      <g>
                        <path
                          class="cls-1"
                          d="M129.3,74.31l27.31-56.54h9.45l26.27,56.54h-9.13l-6.97-14.74h-30.75l-7.13,14.74h-9.05ZM173.59,53.01c-7.85-16.9-10.25-22.1-12.57-27.63-2.48,5.45-4.72,10.57-12.97,27.63h25.55Z"
                        />
                        <path
                          class="cls-1"
                          d="M249.76,49.65c0,9.29-1.04,14.42-4.89,18.58-4.24,4.65-11.93,7.21-21.86,7.21s-17.86-2.64-21.7-7.53c-3.84-4.81-4.4-9.85-4.4-18.34v-31.8h8.17v33.08c0,4.97.64,8.97,3.04,12.09,2.72,3.6,8.65,5.37,15.22,5.37s12.65-1.52,15.46-5.29c2.32-3.04,2.8-7.21,2.8-11.85V17.77h8.17v31.88Z"
                        />
                        <path
                          class="cls-1"
                          d="M278.75,74.31V24.82h-20.98v-7.05h50.54v7.05h-21.06v49.5h-8.49Z"
                        />
                        <path
                          class="cls-1"
                          d="M316.71,74.31V17.77h8.41v23.63h37.16v-23.63h8.49v56.54h-8.49v-25.79h-37.16v25.79h-8.41Z"
                        />
                        <path
                          class="cls-1"
                          d="M383.26,74.31V17.77h46.05v7.05h-37.72v16.82h34.44v6.97h-34.44v18.66h39.97v7.05h-48.29Z"
                        />
                        <path
                          class="cls-1"
                          d="M441,74.31V17.77h9.45c27.95,33.88,32.6,39.4,37.4,45.65-.24-7.05-.4-14.26-.4-45.65h8.41v56.54h-9.21c-27.71-33.64-31.32-38.04-37.48-45.89.08,5.77.16,12.41.16,45.89h-8.33Z"
                        />
                        <path
                          class="cls-1"
                          d="M502.19,74.31l27.31-56.54h9.45l26.27,56.54h-9.13l-6.97-14.74h-30.75l-7.13,14.74h-9.05ZM546.48,53.01c-7.85-16.9-10.25-22.1-12.57-27.63-2.48,5.45-4.72,10.57-12.97,27.63h25.55Z"
                        />
                        <path
                          class="cls-1"
                          d="M56.71,93.21H0S28.01,5.99,28.01,5.99l28.7,87.22ZM108.43,93.21h-47.14S30.51,0,30.51,0h47.28s30.64,93.21,30.64,93.21Z"
                        />
                      </g>
                    </g>
                  </svg>
                </div>
                <div class="text-center space-y-2">
                  <div class="text-xs text-zinc-500 font-mono uppercase tracking-wider">
                    Human Verification Required
                  </div>
                  <div class="text-xs text-zinc-500 font-mono uppercase tracking-wider">
                    Complete verification to continue
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <!-- Scanning Line Animation (only when not verified) -->
          {#if !verificationComplete && !isAlreadyVerified}
            <div class="absolute inset-0 pointer-events-none rounded-lg overflow-hidden">
              <div class="scan-line"></div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>

<style>
/* Scan Line Animation */
.scan-line {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent 0%, oklch(72.2% 0.099 205.4) 50%, transparent 100%);
  box-shadow: 0 0 20px oklch(72.2% 0.099 205.4), 0 0 40px oklch(72.2% 0.099 205.4), 0 0 60px oklch(72.2% 0.099 205.4);
  animation: scanAnimation 3s linear infinite;
  border-radius: 1px;
}

.scan-line::before {
  content: '';
  position: absolute;
  top: -10px;
  left: 50%;
  transform: translateX(-50%);
  width: 100px;
  height: 20px;
  background: linear-gradient(180deg, transparent 0%, oklch(72.2% 0.099 205.4 / 0.3) 50%, transparent 100%);
  border-radius: 50px;
  filter: blur(8px);
}

@keyframes scanAnimation {
  0% {
    top: 0%;
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  90% {
    opacity: 1;
  }
  100% {
    top: 100%;
    opacity: 0;
  }
}
</style>
