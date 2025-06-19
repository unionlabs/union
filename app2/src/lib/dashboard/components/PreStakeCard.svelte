<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Modal from "$lib/components/ui/Modal.svelte"
import type { UserAllocationCalculated } from "$lib/dashboard/stores/airdrop.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

let preStakeAmount = $state(0)
let showConfirmModal = $state(false)

// Get data directly from dashboard store - clean like achievements approach
let allocation = $derived(
  Option.getOrElse(
    Option.flatMap(dashboard.airdrop, (store) => store.calculatedAllocation),
    (): UserAllocationCalculated | null => null,
  ),
)

let isHuman = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.isHuman,
  }),
)

// Get pre-staking state from store
let isPreStaking = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.isStaking,
  }),
)

// Get current incentives percentage
let incentivesPercentage = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => 0,
    onSome: (store) => store.incentivesPercentage,
  }),
)

// Calculate projected yearly earnings from prestaked amount + slider amount
let projectedYearlyEarnings = $derived(
  allocation && incentivesPercentage > 0
    ? Math.floor((allocation.tokens_prestaked + preStakeAmount) * (incentivesPercentage / 100))
    : 0,
)

// Maximum tokens they can pre-stake (based on allocation_percentage)
let maxPreStakeableTokens = $derived(
  allocation && allocation.allocation_percentage
    ? Math.floor(allocation.total_tokens * allocation.allocation_percentage)
    : allocation?.total_tokens || 0,
)

// Progress calculations based on token percentage (for visual progress bar)
let stakingProgress = $derived(
  allocation && allocation.total_tokens > 0
    ? (allocation.tokens_prestaked / allocation.total_tokens) * 100
    : 0,
)

let previewProgress = $derived(
  allocation && allocation.total_tokens > 0
    ? ((allocation.tokens_prestaked + preStakeAmount) / allocation.total_tokens) * 100
    : 0,
)

// Maximum reachable percentage based on allocation limit (yellow line)
let maxReachableProgress = $derived(
  allocation && allocation.total_tokens > 0
    ? Math.min(
      (allocation.tokens_available_total / allocation.total_tokens) * 100,
      100,
    )
    : 0,
)

// Functions
function handlePreStakeAmountChange(amount: number) {
  preStakeAmount = amount
}

function handleShowConfirmModal() {
  if (
    !allocation || preStakeAmount <= 0
    || preStakeAmount > allocation.tokens_remaining_for_prestaking
  ) {
    return
  }
  showConfirmModal = true
}

function handleCloseConfirmModal() {
  showConfirmModal = false
}

async function handleConfirmPreStaking() {
  if (
    !allocation || preStakeAmount <= 0
    || preStakeAmount > allocation.tokens_remaining_for_prestaking
  ) {
    return
  }

  const success = await Option.match(dashboard.airdrop, {
    onNone: () => Promise.resolve(false),
    onSome: (store) => store.updateStaking(preStakeAmount),
  })

  if (success) {
    preStakeAmount = 0 // Reset the form on success
    showConfirmModal = false
  }
}
</script>

<!-- Staking Progress Bar -->
<Card>
  <div class="flex flex-col gap-4 md:gap-0">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold text-white">Pre-staking</h3>
      <div class="text-sm text-zinc-400">
        {
          allocation
          ? `${allocation.tokens_prestaked.toLocaleString()} / ${allocation.total_tokens.toLocaleString()} U`
          : "Loading..."
        }
      </div>
    </div>

    <!-- Mobile Incentives Box (shown after header on mobile) -->
    <div class="block md:hidden p-6 bg-gradient-to-br from-accent/10 to-accent/20 border border-accent/30 rounded-lg">
      <div class="text-center">
        <!-- Incentives Value -->
        <div>
          <div class="text-xs font-mono uppercase tracking-widest text-zinc-400 mb-1">
            YEARLY INCENTIVES
          </div>
          {#if incentivesPercentage > 0}
            <div class="text-5xl font-bold text-accent mb-1">
              {Math.round(incentivesPercentage).toLocaleString()}%
            </div>
            {#if projectedYearlyEarnings > 0}
              <div class="text-sm text-zinc-300 font-medium">
                ~{projectedYearlyEarnings.toLocaleString()} U
              </div>
            {/if}
          {:else}
            <div class="text-4xl font-bold text-zinc-500 mb-1">
              Loading...
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Mobile Info Section (shown after incentives on mobile) -->
    <div class="block md:hidden bg-zinc-800/50 rounded-lg p-4 border border-zinc-700/50">
      <div class="flex items-start gap-3">
        <div class="w-8 h-8 rounded-full bg-accent/20 flex items-center justify-center flex-shrink-0 mt-0.5">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="text-accent"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
            />
            <path d="m9 12 2 2 4-4" />
          </svg>
        </div>
        <div class="flex-1">
          <h4 class="text-sm font-semibold text-white mb-2">How Pre-Staking Works</h4>
          <ul class="text-xs text-zinc-300 space-y-1">
            <li>
              • Pre-staked tokens earn yearly incentives
            </li>
            <li>• Unlock additional rewards at 25%, 50%, 75%, and 100% pre-staked</li>
            <li>• Your pre-stake allocation can be increased by claiming referral codes</li>
            <li>• Share unused allocation with others to earn community bonuses</li>
          </ul>

          {#if allocation && maxReachableProgress < 100}
            <div class="mt-3 p-3 bg-zinc-800/30 border border-zinc-700/50 rounded-md">
              <div class="flex items-start gap-2">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  class="text-accent flex-shrink-0 mt-0.5"
                >
                  <circle
                    cx="12"
                    cy="12"
                    r="10"
                  />
                  <path d="m9 12 2 2 4-4" />
                </svg>
                <div>
                  <p class="text-xs font-medium text-zinc-200 mb-1">Want to reach 100%?</p>
                  <p class="text-xs text-zinc-400">
                    Hunt referral codes from the community to increase your allocation and unlock
                    all milestone rewards!
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Segmented Progress Bar -->
    <div class="relative mb-4">
      <!-- Desktop: Horizontal Progress Bar -->
      <div class="hidden md:block">
        <!-- Background bar -->
        <div class="w-full h-3 bg-zinc-800 rounded-full relative overflow-hidden">
          <!-- Preview progress (background layer) -->
          {#if preStakeAmount > 0}
            <div
              class="absolute inset-0 h-full bg-accent/30 transition-all duration-300 rounded-full"
              style="width: {previewProgress}%"
            >
            </div>
          {/if}

          <!-- Current progress (foreground layer) -->
          <div
            class="relative h-full bg-accent transition-all duration-500 rounded-full z-10"
            style="width: {stakingProgress}%"
          >
          </div>

          <!-- Gap masks to create segments -->
          <div class="absolute inset-0 flex pointer-events-none z-20">
            <div class="w-1/4"></div>
            <div class="w-1 h-full bg-zinc-900"></div>
            <div class="w-1/4"></div>
            <div class="w-1 h-full bg-zinc-900"></div>
            <div class="w-1/4"></div>
            <div class="w-1 h-full bg-zinc-900"></div>
            <div class="w-1/4"></div>
          </div>

          <!-- Maximum reachable line -->
          <div
            class="absolute top-0 bottom-0 w-0.5 pointer-events-none z-30 {allocation && allocation.tokens_remaining_for_prestaking <= 0 ? 'bg-red-400' : 'bg-yellow-400'}"
            style="left: {Math.min(maxReachableProgress, 100)}%"
          >
          </div>

          <!-- Debug info (remove later) -->
          <div class="absolute -top-8 left-0 text-xs text-yellow-400">
            Staked: {allocation?.tokens_prestaked || 0}/{allocation?.total_tokens || 0} | Progress:
            {stakingProgress.toFixed(1)}% | MaxReach: {maxReachableProgress.toFixed(1)}%
          </div>
        </div>

        <!-- Threshold markers -->
        <div class="absolute -bottom-2 left-0 right-0 text-xs">
          <span class="absolute left-0 {previewProgress >= 0 ? 'text-white' : 'text-zinc-500'}"
          >0%</span>
          <span
            class="absolute left-1/4 -translate-x-1/2 {previewProgress >= 25 ? 'text-white' : 'text-zinc-500'}"
          >25%</span>
          <span
            class="absolute left-1/2 -translate-x-1/2 {previewProgress >= 50 ? 'text-white' : 'text-zinc-500'}"
          >50%</span>
          <span
            class="absolute left-3/4 -translate-x-1/2 {previewProgress >= 75 ? 'text-white' : 'text-zinc-500'}"
          >75%</span>
          <span class="absolute right-0 {previewProgress >= 100 ? 'text-white' : 'text-zinc-500'}"
          >100%</span>
        </div>
      </div>

      <!-- Mobile: Vertical Progress Bar -->
      <div class="md:hidden">
        <!-- Mobile Layout: Progress Bar + Cards in Row -->
        <div class="flex gap-6 mt-6 items-stretch">
          <!-- Vertical Progress Bar -->
          <div class="flex-shrink-0 flex items-stretch">
            <div class="relative w-2">
              <!-- Background bar -->
              <div class="w-2 h-full bg-zinc-800 rounded-full relative overflow-hidden">
                <!-- Preview progress (background layer) -->
                {#if preStakeAmount > 0}
                  <div
                    class="absolute top-0 left-0 right-0 bg-accent/30 transition-all duration-300 rounded-full"
                    style="height: {previewProgress}%"
                  >
                  </div>
                {/if}

                <!-- Current progress (foreground layer) -->
                <div
                  class="absolute top-0 left-0 right-0 bg-accent transition-all duration-500 rounded-full z-10"
                  style="height: {stakingProgress}%"
                >
                </div>

                <!-- Gap masks to create segments -->
                <div class="absolute inset-0 flex flex-col pointer-events-none z-20">
                  <div class="h-1/4"></div>
                  <div class="h-1 w-full bg-zinc-900"></div>
                  <div class="h-1/4"></div>
                  <div class="h-1 w-full bg-zinc-900"></div>
                  <div class="h-1/4"></div>
                  <div class="h-1 w-full bg-zinc-900"></div>
                  <div class="h-1/4"></div>
                </div>

                <!-- Maximum reachable line -->
                <div
                  class="absolute left-0 right-0 h-0.5 pointer-events-none z-30 {allocation && allocation.tokens_remaining_for_prestaking <= 0 ? 'bg-red-400' : 'bg-yellow-400'}"
                  style="bottom: {100 - Math.min(maxReachableProgress, 100)}%"
                >
                </div>

                <!-- Debug info (remove later) -->
                <div class="absolute -left-24 top-0 text-xs text-yellow-400 transform -rotate-90 origin-left">
                  Staked: {allocation?.tokens_prestaked || 0}/{allocation?.total_tokens || 0} |
                  MaxReach: {maxReachableProgress.toFixed(1)}%
                </div>
              </div>
            </div>
          </div>

          <!-- Mobile Milestone Cards -->
          <div class="flex-1 grid grid-cols-1 gap-6 h-full">
            <!-- 25% Milestone - Level 1 -->
            <Card class="p-4">
              <div class="flex items-center gap-3 {previewProgress >= 25 ? '' : 'grayscale opacity-60'}">
                <div class="w-8 h-8 rounded-full bg-accent/20 flex items-center justify-center flex-shrink-0">
                  <img
                    src="/airdrop/1.svg"
                    alt=""
                    class="w-4 h-4"
                  >
                </div>
                <div>
                  <h4 class="text-sm font-semibold text-white mb-1">Level 1</h4>
                  <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
                    <li>25% Season 2 XP Boost</li>
                  </ul>
                </div>
              </div>
            </Card>

            <!-- 50% Milestone - Level 2 -->
            <Card class="p-4">
              <div class="flex items-center gap-3 {previewProgress >= 50 ? '' : 'grayscale opacity-60'}">
                <div class="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center flex-shrink-0">
                  <img
                    src="/airdrop/2.svg"
                    alt=""
                    class="w-4 h-4"
                  >
                </div>
                <div>
                  <h4 class="text-sm font-semibold text-white mb-1">Level 2</h4>
                  <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
                    <li>25% Season 2 XP Boost</li>
                    <li>Eligible for unclaimed rewards</li>
                  </ul>
                </div>
              </div>
            </Card>

            <!-- 75% Milestone - Level 3 -->
            <Card class="p-4">
              <div class="flex items-center gap-3 {previewProgress >= 75 ? '' : 'grayscale opacity-60'}">
                <div class="w-8 h-8 rounded-full bg-purple-500/20 flex items-center justify-center flex-shrink-0">
                  <img
                    src="/airdrop/3.svg"
                    alt=""
                    class="w-4 h-4"
                  >
                </div>
                <div>
                  <h4 class="text-sm font-semibold text-white mb-1">Level 3</h4>
                  <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
                    <li>50% Season 2 XP Boost</li>
                    <li>Eligible for unclaimed rewards</li>
                    <li>Free uname Handle + Priority Access</li>
                  </ul>
                </div>
              </div>
            </Card>

            <!-- 100% Milestone - Level 4 -->
            <Card class="p-4">
              <div class="flex items-center gap-3 {previewProgress >= 100 ? '' : 'grayscale opacity-60'}">
                <div class="w-8 h-8 rounded-full bg-orange-500/20 flex items-center justify-center flex-shrink-0">
                  <img
                    src="/airdrop/4.svg"
                    alt=""
                    class="w-4 h-4"
                  >
                </div>
                <div>
                  <h4 class="text-sm font-semibold text-white mb-1">Level 4</h4>
                  <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
                    <li>50% Season 2 XP Boost</li>
                    <li>Eligible for unclaimed rewards</li>
                    <li>Free uname Handle + Priority Access</li>
                    <li>Chance to meet the Union core team</li>
                    <li>A date with Jessica?</li>
                  </ul>
                </div>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>

    <!-- Desktop Reward Milestone Cards -->
    <div class="hidden md:grid md:grid-cols-2 lg:grid-cols-4 gap-6 mt-6 mb-4">
      <!-- 25% Milestone - Level 1 -->
      <Card class="p-4">
        <div class="flex flex-col items-center text-center gap-3 {previewProgress >= 25 ? '' : 'grayscale opacity-60'}">
          <div class="w-12 h-12 rounded-full bg-accent/20 flex items-center justify-center">
            <img
              src="/airdrop/1.svg"
              alt=""
            >
          </div>
          <div>
            <h4 class="text-sm font-semibold text-white mb-1">Season 2 Boost</h4>
            <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
              <li>25% Season 2 XP Boost</li>
            </ul>
          </div>
        </div>
      </Card>

      <!-- 50% Milestone - Level 2 -->
      <Card class="p-4">
        <div class="flex flex-col items-center text-center gap-3 {previewProgress >= 50 ? '' : 'grayscale opacity-60'}">
          <div class="w-12 h-12 rounded-full bg-blue-500/20 flex items-center justify-center">
            <img
              src="/airdrop/2.svg"
              alt=""
            >
          </div>
          <div>
            <h4 class="text-sm font-semibold text-white mb-1">Enhanced Rewards</h4>
            <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
              <li>25% Season 2 XP Boost</li>
              <li>Eligible for unclaimed rewards</li>
            </ul>
          </div>
        </div>
      </Card>

      <!-- 75% Milestone - Level 3 -->
      <Card class="p-4">
        <div class="flex flex-col items-center text-center gap-3 {previewProgress >= 75 ? '' : 'grayscale opacity-60'}">
          <div class="w-12 h-12 rounded-full bg-purple-500/20 flex items-center justify-center">
            <img
              src="/airdrop/3.svg"
              alt=""
            >
          </div>
          <div>
            <h4 class="text-sm font-semibold text-white mb-1">Maximum Benefits</h4>
            <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
              <li>50% Season 2 XP Boost</li>
              <li>Eligible for unclaimed rewards</li>
              <li>Free uname Handle + Priority Access</li>
            </ul>
          </div>
        </div>
      </Card>

      <!-- 100% Milestone - Level 4 -->
      <Card class="p-4">
        <div class="flex flex-col items-center text-center gap-3 {previewProgress >= 100 ? '' : 'grayscale opacity-60'}">
          <div class="w-12 h-12 rounded-full bg-orange-500/20 flex items-center justify-center">
            <img
              src="/airdrop/4.svg"
              alt=""
            >
          </div>
          <div>
            <h4 class="text-sm font-semibold text-white mb-1">Max Rewards</h4>
            <ul class="text-xs text-zinc-400 list-disc list-inside space-y-0.5">
              <li>50% Season 2 XP Boost</li>
              <li>Eligible for unclaimed rewards</li>
              <li>Free uname Handle + Priority Access</li>
              <li>Chance to meet the Union core team</li>
              <li>A date with Jessica?</li>
            </ul>
          </div>
        </div>
      </Card>
    </div>

    <!-- Info & Incentives Section (50/50 layout) -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-4">
      <!-- Selling Point / Info Section (hidden on mobile, shown in 50/50 on desktop) -->
      <div class="hidden md:block bg-zinc-800/50 rounded-lg p-4 border border-zinc-700/50">
        <div class="flex items-start gap-3">
          <div class="w-8 h-8 rounded-full bg-accent/20 flex items-center justify-center flex-shrink-0 mt-0.5">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="text-accent"
            >
              <circle
                cx="12"
                cy="12"
                r="10"
              />
              <path d="m9 12 2 2 4-4" />
            </svg>
          </div>
          <div class="flex-1">
            <h4 class="text-sm font-semibold text-white mb-2">How Pre-Staking Works</h4>
            <ul class="text-xs text-zinc-300 space-y-1">
              <li>
                • Pre-staked tokens earn yearly incentives
              </li>
              <li>• Unlock additional rewards at 25%, 50%, 75%, and 100% pre-staked</li>
              <li>• Your pre-stake allocation can be increased by claiming referral codes</li>
              <li>• Share unused allocation with others to earn community bonuses</li>
            </ul>

            {#if allocation && maxReachableProgress < 100}
              <div class="mt-3 p-3 bg-zinc-800/30 border border-zinc-700/50 rounded-md">
                <div class="flex items-start gap-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="text-accent flex-shrink-0 mt-0.5"
                  >
                    <circle
                      cx="12"
                      cy="12"
                      r="10"
                    />
                    <path d="m9 12 2 2 4-4" />
                  </svg>
                  <div>
                    <p class="text-xs font-medium text-zinc-200 mb-1">Want to reach 100%?</p>
                    <p class="text-xs text-zinc-400">
                      Hunt referral codes from the community to increase your allocation and unlock
                      all milestone rewards!
                    </p>
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Incentives Box (hidden on mobile, shown in 50/50 on desktop) -->
      <Card
        class="hidden md:flex p-6 bg-gradient-to-br from-accent/10 to-accent/20 border-accent/30 h-full flex-col justify-center"
      >
        <div class="text-center">
          <!-- Incentives Value -->
          <div>
            <div class="text-xs font-mono uppercase tracking-widest text-zinc-400 mb-1">
              YEARLY INCENTIVES
            </div>
            {#if incentivesPercentage > 0}
              <div class="text-4xl md:text-5xl font-bold text-accent mb-1">
                {Math.round(incentivesPercentage).toLocaleString()}%
              </div>
              {#if projectedYearlyEarnings > 0}
                <div class="text-sm text-zinc-300 font-medium">
                  ~{projectedYearlyEarnings.toLocaleString()} U
                </div>
              {/if}
            {:else}
              <div class="text-3xl md:text-4xl font-bold text-zinc-500 mb-1">
                Loading...
              </div>
            {/if}
          </div>
        </div>
      </Card>
    </div>

    <!-- Input Section -->
    <div>
      <!-- Input Section (full width) -->
      <div>
        <Card>
          <div class="flex flex-col gap-6">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-lg font-semibold text-white">Add to Your Pre-Stake</h3>
                <p class="text-sm text-zinc-400">
                  Choose additional amount to pre-stake for more rewards
                </p>
              </div>
              <div class="text-right">
                <div class="text-sm text-zinc-400">Available to Add</div>
                <div class="text-lg font-semibold text-white">
                  {
                    allocation
                    ? allocation.tokens_remaining_for_prestaking
                      .toLocaleString()
                    : "0"
                  } U
                </div>
              </div>
            </div>

            <!-- Max Allowance Warning -->
            {#if allocation && allocation.tokens_remaining_for_prestaking <= 0}
              {#if allocation.tokens_prestaked >= allocation.total_tokens}
                <!-- User has staked all their tokens -->
                <div class="p-4 bg-accent/10 border border-accent/30 rounded-lg">
                  <div class="flex items-start gap-3">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="text-accent flex-shrink-0 mt-0.5"
                    >
                      <circle
                        cx="12"
                        cy="12"
                        r="10"
                      />
                      <path d="m9 12 2 2 4-4" />
                    </svg>
                    <div>
                      <p class="text-sm font-medium text-accent mb-1">
                        Congratulations! You've pre-staked all your tokens.
                      </p>
                      <p class="text-xs text-accent/80">
                        You've maximized your pre-staking rewards.
                      </p>
                    </div>
                  </div>
                </div>
              {:else}
                <!-- User has reached their allocation limit but hasn't staked all tokens -->
                <div class="p-4 bg-amber-500/10 border border-amber-500/30 rounded-lg">
                  <div class="flex items-start gap-3">
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="text-amber-400 flex-shrink-0 mt-0.5"
                    >
                      <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
                      <path d="M12 9v4" />
                      <path d="m12 17 .01 0" />
                    </svg>
                    <div>
                      <p class="text-sm font-medium text-amber-300 mb-1">
                        You've staked your maximum allowance. Claim codes to pre-stake more!
                      </p>
                      <p class="text-xs text-amber-200">
                        Hunt referral codes from the community to increase your allocation and
                        unlock more milestone rewards.
                      </p>
                    </div>
                  </div>
                </div>
              {/if}
            {/if}

            <!-- Range Slider -->
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <label
                  for="stake-range"
                  class="text-sm font-medium text-zinc-300"
                >
                  Additional Amount to Add
                </label>
                <div class="text-sm text-accent font-medium">
                  +{preStakeAmount.toLocaleString()} U
                </div>
              </div>

              <input
                id="stake-range"
                type="range"
                min="0"
                max={allocation ? allocation.tokens_remaining_for_prestaking : 0}
                bind:value={preStakeAmount}
                class="w-full h-3 bg-gradient-to-r from-zinc-800 via-zinc-700 to-zinc-800 rounded-lg appearance-none cursor-pointer slider"
              />

              <!-- Quick Amount Buttons -->
              <div class="flex gap-2">
                <button
                  class="px-3 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md transition-colors"
                  onclick={() =>
                  handlePreStakeAmountChange(
                    allocation
                      ? Math.floor(
                        allocation.tokens_remaining_for_prestaking * 0.25,
                      )
                      : 0,
                  )}
                >
                  25%
                </button>
                <button
                  class="px-3 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md transition-colors"
                  onclick={() =>
                  handlePreStakeAmountChange(
                    allocation
                      ? Math.floor(
                        allocation.tokens_remaining_for_prestaking * 0.5,
                      )
                      : 0,
                  )}
                >
                  50%
                </button>
                <button
                  class="px-3 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md transition-colors"
                  onclick={() =>
                  handlePreStakeAmountChange(
                    allocation
                      ? Math.floor(
                        allocation.tokens_remaining_for_prestaking * 0.75,
                      )
                      : 0,
                  )}
                >
                  75%
                </button>
                <button
                  class="px-3 py-1 text-xs bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-md transition-colors"
                  onclick={() =>
                  handlePreStakeAmountChange(
                    allocation ? allocation.tokens_remaining_for_prestaking : 0,
                  )}
                >
                  Max
                </button>
              </div>
            </div>

            <!-- Confirm Button -->
            <div class="flex gap-6 justify-start">
              <Button
                variant="primary"
                type="button"
                onclick={handleShowConfirmModal}
                disabled={!isHuman || isPreStaking || preStakeAmount <= 0}
              >
                {#if isPreStaking}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="animate-spin mr-2"
                  >
                    <path d="M21 12a9 9 0 11-6.219-8.56" />
                  </svg>
                  Adding to Pre-Stake...
                {:else}
                  Add to Pre-Stake
                {/if}
              </Button>

              {#if preStakeAmount > 0}
                <Button
                  variant="outline"
                  type="button"
                  onclick={() => handlePreStakeAmountChange(0)}
                  disabled={isPreStaking}
                >
                  Reset
                </Button>
              {/if}
            </div>
          </div>
        </Card>
      </div>
    </div>
  </div>
</Card>

<!-- Confirmation Modal -->
<Modal
  isOpen={showConfirmModal}
  onClose={handleCloseConfirmModal}
  class="w-full max-w-md"
>
  <div>
    <h3 class="text-xl font-semibold text-white mb-4">Confirm Pre-Stake</h3>

    <div class="space-y-4 mb-6">
      <div class="bg-zinc-800/50 rounded-lg p-4">
        <div class="flex justify-between items-center mb-2">
          <span class="text-sm text-zinc-400">Amount to Pre-Stake:</span>
          <span class="text-lg font-semibold text-white">{preStakeAmount.toLocaleString()} U</span>
        </div>

        <div class="flex justify-between items-center mb-2">
          <span class="text-sm text-zinc-400">Current Pre-Staked:</span>
          <span class="text-sm text-zinc-300">
            {allocation ? allocation.tokens_prestaked.toLocaleString() : "0"} U
          </span>
        </div>

        <div class="flex justify-between items-center border-t border-zinc-700 pt-2">
          <span class="text-sm font-medium text-zinc-300">New Total Pre-Staked:</span>
          <span class="text-lg font-semibold text-accent">
            {
              allocation
              ? (allocation.tokens_prestaked + preStakeAmount).toLocaleString()
              : preStakeAmount.toLocaleString()
            } U
          </span>
        </div>
      </div>

      <!-- Info Box -->
      <div class="p-4 bg-accent/10 border border-accent/30 rounded-lg">
        <div class="flex items-start gap-3">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="text-accent flex-shrink-0 mt-0.5"
          >
            <circle
              cx="12"
              cy="12"
              r="10"
            />
            <path d="m9 12 2 2 4-4" />
          </svg>
          <div>
            <p class="text-sm font-medium text-accent mb-1">
              Ready to pre-stake!
            </p>
            <p class="text-xs text-accent/80">
              I understand that this U will be pre-staked and locked until mainnet launch.
            </p>
          </div>
        </div>
      </div>
    </div>

    <div class="flex gap-6">
      <Button
        variant="outline"
        type="button"
        onclick={handleCloseConfirmModal}
        disabled={isPreStaking}
        class="flex-1"
      >
        Cancel
      </Button>

      <Button
        variant="primary"
        type="button"
        onclick={handleConfirmPreStaking}
        disabled={isPreStaking}
        class="flex-1"
      >
        {#if isPreStaking}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="animate-spin mr-2"
          >
            <path d="M21 12a9 9 0 11-6.219-8.56" />
          </svg>
          Confirming...
        {:else}
          Confirm Pre-Stake
        {/if}
      </Button>
    </div>
  </div>
</Modal>

<style>
/* Range Slider Styling with Gradients */
.slider {
  background: linear-gradient(90deg, #27272a 0%, #3f3f46 50%, #27272a 100%);
  border-radius: 12px;
  outline: none;
  transition: all 0.3s ease;
}

.slider:hover {
  background: linear-gradient(90deg, #3f3f46 0%, #52525b 50%, #3f3f46 100%);
  box-shadow: 0 0 20px rgba(64, 165, 255, 0.1);
}

.slider::-webkit-slider-thumb {
  appearance: none;
  height: 24px;
  width: 24px;
  border-radius: 50%;
  background: linear-gradient(135deg, oklch(72.2% 0.099 205.4) 0%, oklch(82.2% 0.129 215.4) 50%, oklch(72.2% 0.099 205.4) 100%);
  cursor: pointer;
  box-shadow: 
    0 0 0 3px rgba(64, 165, 255, 0.2),
    0 4px 12px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
  transition: all 0.2s ease;
  border: 2px solid rgba(255, 255, 255, 0.1);
}

.slider::-webkit-slider-thumb:hover {
  transform: scale(1.1);
  background: linear-gradient(135deg, oklch(82.2% 0.129 215.4) 0%, oklch(92.2% 0.159 225.4) 50%, oklch(82.2% 0.129 215.4) 100%);
  box-shadow: 
    0 0 0 4px rgba(64, 165, 255, 0.3),
    0 6px 16px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.slider::-webkit-slider-thumb:active {
  transform: scale(0.95);
  box-shadow: 
    0 0 0 2px rgba(64, 165, 255, 0.4),
    0 2px 8px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.slider::-webkit-slider-track {
  width: 100%;
  height: 12px;
  cursor: pointer;
  background: linear-gradient(90deg, #18181b 0%, #27272a 25%, #3f3f46 50%, #27272a 75%, #18181b 100%);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
}

/* Firefox Styles */
.slider::-moz-range-thumb {
  height: 24px;
  width: 24px;
  border-radius: 50%;
  background: linear-gradient(135deg, oklch(72.2% 0.099 205.4) 0%, oklch(82.2% 0.129 215.4) 50%, oklch(72.2% 0.099 205.4) 100%);
  cursor: pointer;
  border: 2px solid rgba(255, 255, 255, 0.1);
  box-shadow: 
    0 0 0 3px rgba(64, 165, 255, 0.2),
    0 4px 12px rgba(0, 0, 0, 0.3);
  transition: all 0.2s ease;
}

.slider::-moz-range-thumb:hover {
  transform: scale(1.1);
  background: linear-gradient(135deg, oklch(82.2% 0.129 215.4) 0%, oklch(92.2% 0.159 225.4) 50%, oklch(82.2% 0.129 215.4) 100%);
  box-shadow: 
    0 0 0 4px rgba(64, 165, 255, 0.3),
    0 6px 16px rgba(0, 0, 0, 0.4);
}

.slider::-moz-range-track {
  width: 100%;
  height: 12px;
  cursor: pointer;
  background: linear-gradient(90deg, #18181b 0%, #27272a 25%, #3f3f46 50%, #27272a 75%, #18181b 100%);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
}

.slider::-moz-range-progress {
  background: linear-gradient(90deg, oklch(72.2% 0.099 205.4) 0%, oklch(82.2% 0.129 215.4) 50%, oklch(72.2% 0.099 205.4) 100%);
  height: 12px;
  border-radius: 12px;
  box-shadow: 
    0 0 8px rgba(64, 165, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

/* Focus states */
.slider:focus {
  box-shadow: 0 0 0 3px rgba(64, 165, 255, 0.2);
}

.slider:focus::-webkit-slider-thumb {
  box-shadow: 
    0 0 0 4px rgba(64, 165, 255, 0.4),
    0 4px 12px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}
</style>
