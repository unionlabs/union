<script lang="ts">
import { goto } from "$app/navigation"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

const isEligible = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.isEligible,
  }),
)

const allocation = $derived(
  Option.flatMap(dashboard.airdrop, (store) => store.calculatedAllocation),
)

const totalAllocationTokens = $derived(
  Option.match(allocation, {
    onNone: () => 0,
    onSome: (alloc) => alloc.total_tokens,
  }),
)

const stakingPercentage = $derived(
  Option.match(allocation, {
    onNone: () => 0,
    onSome: (alloc) => ((alloc.allocation_percentage || 0) * 100).toFixed(0),
  }),
)

// Celebration is now triggered from Step6 when clicking "Check results"
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-3 sm:p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            {#if isEligible}
              Your Allocation
            {:else}
              Scan Complete
            {/if}
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            {#if isEligible}
              Here's your U Drop allocation based on your activity.
            {:else}
              Sorry, you're ineligible for U Drop Season 1. Come back for Season 2.
            {/if}
          </p>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="space-y-3">
        {#if isEligible}
          <Button
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onNext}
          >
            Continue to Verification
          </Button>
        {:else}
          <Button
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            onclick={() => goto("/dashboard")}
          >
            Back to Dashboard
          </Button>
        {/if}

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
        <h1 class="text-2xl font-semibold">
          {#if isEligible}
            Your Allocation
          {:else}
            Scan Complete
          {/if}
        </h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          {#if isEligible}
            Here's your U drop allocation based on your activity.
          {:else}
            Sorry, you're ineligible for U Drop Season 1. Come back for Season 2.
          {/if}
        </p>
      </div>

      {#if isEligible && Option.isSome(allocation)}
        <!-- U Drop Allocation Card -->
        <Card class="flex flex-col flex-1 relative overflow-hidden">
          <!-- Gradient Background -->
          <div class="absolute inset-0">
            <div class="w-full h-full bg-gradient-to-br from-accent/10 to-accent/20"></div>
          </div>

          <div class="flex flex-col gap-2 h-full relative z-10">
            <!-- Centered Content -->
            <div class="flex-1 flex flex-col items-center justify-center text-center gap-6">
              <!-- Token Allocation -->
              <div class="flex flex-col items-center gap-2">
                <div class="text-sm text-zinc-400 font-medium uppercase tracking-wider">
                  TOKEN ALLOCATION
                </div>
                <div class="text-[clamp(2rem,6vw,3rem)] font-bold text-white leading-none">
                  {totalAllocationTokens.toLocaleString()} U
                </div>
              </div>

              <!-- Staking Allowance -->
              <div class="flex flex-col items-center gap-2">
                <div class="text-sm text-zinc-400 font-medium uppercase tracking-wider">
                  STAKING ALLOWANCE
                </div>
                <div class="text-[clamp(2rem,6vw,3rem)] font-bold text-accent leading-none">
                  {stakingPercentage}%
                </div>
              </div>
            </div>
          </div>
        </Card>
      {:else}
        <!-- No allocation fallback -->
        <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
          <!-- Union Logo -->
          <div class="absolute top-3 left-3 z-20">
            <img
              src="/images/union-logo-glyph.svg"
              alt="Union logo"
              class="w-6 h-6 opacity-60"
            />
          </div>

          <!-- Main Content Area -->
          <div class="relative flex-1 flex flex-col items-center justify-center p-6 overflow-hidden">
            <!-- Mock blurred background -->
            <div class="absolute inset-0">
              <div class="w-full h-full bg-gradient-to-br from-accent/20 to-purple-500/20 blur-xl opacity-70 scale-110">
              </div>
              <div class="absolute inset-0 bg-zinc-950/40"></div>
            </div>

            <div class="text-center relative z-10">
              <!-- No allocation -->
              <div class="text-4xl font-bold text-zinc-300 mb-2">
                Thank You 🙏
              </div>
              <div class="text-sm text-zinc-400 mt-4">
                Come back for Season 2
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/snippet}
</StepLayout>
