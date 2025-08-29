<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { Effect, Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

// Individual checkbox states for each agreement point
let agreements = $state({
  termsAndConditions: false,
  notProhibitedJurisdiction: false,
  taxCompliance: false,
  walletOwnership: false,
  riskUnderstanding: false,
})

let shouldAcceptTerms = $state(false)
let showSuccessState = $state(false)

// Derived state to check if all agreements are accepted
const allAgreementsAccepted = $derived(
  Object.values(agreements).every(Boolean),
)

// Derived state to count completed agreements
const completedAgreements = $derived(
  Object.values(agreements).filter(Boolean).length,
)

const totalAgreements = 5

const hasAcceptedTerms = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.hasAcceptedTerms,
  }),
)

// Use runPromiseExit$ for automatic loading state tracking
const acceptTermsResult = runPromiseExit$(() =>
  shouldAcceptTerms
    ? Effect.gen(function*() {
      // Add artificial delay to show loading state
      yield* Effect.sleep("1.5 seconds")

      const success = yield* Option.match(dashboard.airdrop, {
        onNone: () => Effect.succeed(false),
        onSome: (store) => Effect.tryPromise(() => store.acceptTermsOfService()),
      })

      if (success) {
        // Show success state for 2 seconds before continuing
        showSuccessState = true
        yield* Effect.sleep("2 seconds")
        onNext()
      } else {
        console.error("Failed to accept terms")
      }

      shouldAcceptTerms = false
    })
    : Effect.void
)

// Derived state for button loading (matches Step 3 pattern)
const isAcceptingTerms = $derived(shouldAcceptTerms)

function toggleAgreement(key: keyof typeof agreements) {
  agreements[key] = !agreements[key]
}

function acceptTerms() {
  if (!allAgreementsAccepted || isAcceptingTerms) {
    return // Already running or not all checkboxes checked
  }
  shouldAcceptTerms = true
}

function openTermsLink() {
  window.open("https://union.build/terms-of-service", "_blank")
}

function openPrivacyLink() {
  window.open("https://union.build/privacy-policy", "_blank")
}

function openAirdropTermsLink() {
  window.open("https://union.build/airdrop-terms-and-conditions", "_blank")
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 flex-1 hidden lg:block">
        <!-- Header -->
        <div>
          <div class="flex items-center justify-between">
            <h1 class="text-xl font-semibold">
              U Drop Agreement
            </h1>
            <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
              <span class="text-xs font-medium text-red-400 uppercase">Required</span>
            </div>
          </div>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Welcome to the Union. By participating in the U Drop program, you represent and warrant
            the following to Union and its affiliates:
          </p>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="space-y-3">
        <!-- Status Box - Always present to prevent layout shifts -->
        <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
          <div class="flex items-center gap-3">
            <div class="size-8 rounded-lg flex items-center justify-center flex-shrink-0 {hasAcceptedTerms || showSuccessState ? 'bg-accent/20 border border-accent/40' : isAcceptingTerms ? 'bg-accent/20' : 'bg-zinc-700'}">
              {#if hasAcceptedTerms || showSuccessState}
                <svg
                  class="w-4 h-4 text-accent"
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
              {:else if isAcceptingTerms}
                <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                </div>
              {:else}
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
                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              {/if}
            </div>
            <div class="flex-1">
              {#if hasAcceptedTerms}
                <div class="text-sm font-medium text-white">Agreement Accepted</div>
                <div class="text-xs text-accent mt-1">Terms and conditions accepted</div>
              {:else if showSuccessState}
                <div class="text-sm font-medium text-white">Agreement Complete</div>
                <div class="text-xs text-accent mt-1">Successfully accepted terms</div>
              {:else if isAcceptingTerms}
                <div class="text-sm font-medium text-white">Processing Agreement</div>
                <div class="text-xs text-accent mt-1">Accepting terms and conditions...</div>
              {:else}
                <div class="flex items-center justify-between mb-2">
                  <div class="text-sm font-medium text-white">Agreement Progress</div>
                  <div class="text-xs font-mono text-zinc-400">
                    {completedAgreements}/{totalAgreements}
                  </div>
                </div>

                <!-- Progress bar -->
                <div class="w-full bg-zinc-800 rounded-full h-1.5">
                  <div
                    class="bg-accent h-1.5 rounded-full transition-all duration-300 ease-out"
                    style="width: {(completedAgreements / totalAgreements) * 100}%"
                  >
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <!-- Button - Always present to prevent layout shifts -->
        <Button
          variant="primary"
          class="flex w-full items-center justify-center gap-3"
          disabled={hasAcceptedTerms
          ? false
          : (showSuccessState || isAcceptingTerms || !allAgreementsAccepted)}
          onclick={hasAcceptedTerms ? onNext : acceptTerms}
        >
          {#if hasAcceptedTerms}
            Continue
          {:else if isAcceptingTerms}
            <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
            </div>
            Processing...
          {:else if showSuccessState}
            <svg
              class="w-4 h-4 text-current"
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
            Success!
          {:else}
            Accept & Continue
          {/if}
        </Button>

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
          <h1 class="text-xl font-semibold">
            Union Airdrop Agreement
          </h1>
          <div class="px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-md flex items-center justify-center">
            <span class="text-xs font-medium text-red-400 uppercase">Required</span>
          </div>
        </div>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Welcome to the Union. By participating in the Union airdrop program, you represent and
          warrant the following to Union and its affiliates:
        </p>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col">
        {#if hasAcceptedTerms}
          <!-- Already accepted state -->
          <div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-accent/10 to-accent/20">
            <div class="text-center">
              <div class="size-16 rounded-full bg-accent/20 border border-accent/30 flex items-center justify-center mx-auto mb-4">
                <svg
                  class="w-8 h-8 text-accent"
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
              </div>
              <div class="text-lg font-medium text-accent mb-2">Agreement Completed</div>
              <div class="text-sm text-zinc-400">All requirements have been accepted</div>
            </div>
          </div>
        {:else}
          <!-- Agreement Checkboxes -->
          <div class="flex flex-col h-full">
            <!-- Scrollable Checkboxes -->
            <div class="flex-1 overflow-y-auto p-4">
              <div class="space-y-4">
                <!-- Agreement 1: Terms, Privacy, Airdrop Terms -->
                <label class="flex items-start gap-3 cursor-pointer group">
                  <div class="relative mt-0.5 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={agreements.termsAndConditions}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded border-2 transition-colors {agreements.termsAndConditions ? 'bg-accent border-accent' : 'border-zinc-600 bg-zinc-800 group-hover:border-zinc-500'}">
                      {#if agreements.termsAndConditions}
                        <svg
                          class="w-full h-full text-white"
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
                      {/if}
                    </div>
                  </div>
                  <div class="text-xs text-zinc-300 leading-relaxed">
                    I have read, understand, and agree to be bound by Union's
                    <button
                      onclick={openTermsLink}
                      class="text-accent hover:text-accent/80 underline underline-offset-2"
                    >
                      Terms of Use
                    </button>,
                    <button
                      onclick={openPrivacyLink}
                      class="text-accent hover:text-accent/80 underline underline-offset-2"
                    >
                      Privacy Policy
                    </button>, and the
                    <button
                      onclick={openAirdropTermsLink}
                      class="text-accent hover:text-accent/80 underline underline-offset-2"
                    >
                      Airdrop Program Terms & Conditions
                    </button>, including the binding arbitration and waiver of jury trial provisions
                    set forth therein;
                  </div>
                </label>

                <!-- Agreement 2: Not Prohibited Jurisdiction -->
                <label class="flex items-start gap-3 cursor-pointer group">
                  <div class="relative mt-0.5 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={agreements.notProhibitedJurisdiction}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded border-2 transition-colors {agreements.notProhibitedJurisdiction ? 'bg-accent border-accent' : 'border-zinc-600 bg-zinc-800 group-hover:border-zinc-500'}">
                      {#if agreements.notProhibitedJurisdiction}
                        <svg
                          class="w-full h-full text-white"
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
                      {/if}
                    </div>
                  </div>
                  <div class="text-xs text-zinc-300 leading-relaxed">
                    I am not a citizen or resident of a Prohibited Jurisdiction (as defined in our
                    Airdrop Program Terms & Conditions) or any other jurisdiction where my
                    participation in this airdrop program unlawful pursuant to applicable law;
                  </div>
                </label>

                <!-- Agreement 3: Tax Compliance -->
                <label class="flex items-start gap-3 cursor-pointer group">
                  <div class="relative mt-0.5 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={agreements.taxCompliance}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded border-2 transition-colors {agreements.taxCompliance ? 'bg-accent border-accent' : 'border-zinc-600 bg-zinc-800 group-hover:border-zinc-500'}">
                      {#if agreements.taxCompliance}
                        <svg
                          class="w-full h-full text-white"
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
                      {/if}
                    </div>
                  </div>
                  <div class="text-xs text-zinc-300 leading-relaxed">
                    I am solely responsible for complying with applicable law in my respective
                    jurisdiction and for reporting and paying any taxes associated with my Airdrop;
                  </div>
                </label>

                <!-- Agreement 4: Wallet Ownership -->
                <label class="flex items-start gap-3 cursor-pointer group">
                  <div class="relative mt-0.5 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={agreements.walletOwnership}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded border-2 transition-colors {agreements.walletOwnership ? 'bg-accent border-accent' : 'border-zinc-600 bg-zinc-800 group-hover:border-zinc-500'}">
                      {#if agreements.walletOwnership}
                        <svg
                          class="w-full h-full text-white"
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
                      {/if}
                    </div>
                  </div>
                  <div class="text-xs text-zinc-300 leading-relaxed">
                    I am the sole owner of the digital asset wallet I am using to claim my Airdrop;
                    and
                  </div>
                </label>

                <!-- Agreement 5: Risk Understanding -->
                <label class="flex items-start gap-3 cursor-pointer group">
                  <div class="relative mt-0.5 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={agreements.riskUnderstanding}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded border-2 transition-colors {agreements.riskUnderstanding ? 'bg-accent border-accent' : 'border-zinc-600 bg-zinc-800 group-hover:border-zinc-500'}">
                      {#if agreements.riskUnderstanding}
                        <svg
                          class="w-full h-full text-white"
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
                      {/if}
                    </div>
                  </div>
                  <div class="text-xs text-zinc-300 leading-relaxed">
                    I understand that I am participating in this Airdrop program at my own risk and
                    I fully understand the risks associated with digital assets, staking and
                    slashing, private key security, and blockchain technology generally.
                  </div>
                </label>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/snippet}
</StepLayout>
