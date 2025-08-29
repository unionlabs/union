<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { Effect, Option } from "effect"
import CelebrationOverlay from "../CelebrationOverlay.svelte"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

// Scanning state
let isScanning = $state(false)
let checkingComplete = $state(false)
let showFlash = $state(false)
let currentStep = $state(0)

// Derived eligibility and allocation data
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

const checkingSteps = [
  "Starting the engine…",
  "Stalking your dms…",
  "Aura and vibe check…",
  "Conscripting goblins…",
  "Counting zkgms…",
  "Damn that's a lot bro…",
  "Wandering with the whale sharks…",
  "Reticulating splines…",
  "Done",
]

async function startChecking() {
  if (isScanning) {
    return
  }

  isScanning = true

  try {
    await runPromise(Effect.gen(function*() {
      // Step through checking process
      for (let i = 0; i < checkingSteps.length - 1; i++) {
        currentStep = i
        yield* Effect.sleep("1 second")
      }

      // Actually scan for allocation
      currentStep = checkingSteps.length - 1 // "done"
      const success = yield* Option.match(dashboard.airdrop, {
        onNone: () => Effect.succeed(false),
        onSome: (store) => Effect.tryPromise(() => store.scanForMyAllocation()),
      })

      // Flash discovery effect
      showFlash = true
      yield* Effect.sleep("200 millis")
      showFlash = false

      // Color reveal
      yield* Effect.sleep("100 millis")
      checkingComplete = true
    }))
  } finally {
    isScanning = false
  }
}

let showCelebration = $state(false)

function handleButtonClick() {
  if (!checkingComplete) {
    startChecking()
  } else {
    // Show celebration if eligible, otherwise go to next step
    if (isEligible && totalAllocationTokens > 0) {
      showCelebration = true
    } else {
      onNext()
    }
  }
}

function handleCelebrationClose() {
  showCelebration = false
  onNext()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            Profile Scan
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            We'll scan your profile to check your eligibility and calculate your allocation.
          </p>
        </div>
      </div>

      <!-- Scanning Status & Action Buttons -->
      <div class="space-y-3">
        <!-- Scanning Status -->
        <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
          {#if checkingComplete}
            <div class="flex items-center gap-3">
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
                    stroke-width="3"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-white">Scan Complete</div>
                <div class="text-xs text-accent mt-1">Profile verified successfully</div>
              </div>
            </div>
          {:else if isScanning}
            <div class="flex items-center gap-3">
              <div class="size-8 rounded-lg bg-accent/20 flex items-center justify-center flex-shrink-0">
                <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                </div>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-white">Scanning Profile</div>
                <div class="text-xs text-accent mt-1">
                  {
                    currentStep < checkingSteps.length - 1
                    ? checkingSteps[currentStep]
                    : "Finalizing..."
                  }
                </div>
              </div>
            </div>
          {:else}
            <div class="flex items-center gap-3">
              <div class="size-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
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
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-white">Ready to Scan</div>
                <div class="text-xs text-zinc-400 mt-1">
                  Click start to begin profile verification
                </div>
              </div>
            </div>
          {/if}
        </div>

        <Button
          variant="primary"
          class="flex w-full items-center justify-center gap-3"
          disabled={isScanning}
          onclick={handleButtonClick}
        >
          {checkingComplete ? "Check results" : "Start"}
          {#if isScanning}
            <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
            </div>
          {/if}
        </Button>

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
    <div class="relative w-full h-full flex flex-col p-3">
      <!-- Mobile Title - shown above the square on mobile -->
      <div class="block lg:hidden mb-4 px-1">
        <h1 class="text-2xl font-semibold">
          Profile Scan
        </h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          We'll scan your profile to check your eligibility and calculate your allocation.
        </p>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative {!checkingComplete ? 'grayscale' : ''} transition-all duration-500">
        <!-- Scanning Line (inside profile card) -->
        {#if isScanning}
          <div class="absolute inset-0 z-30 pointer-events-none rounded-lg overflow-hidden">
            <div class="scan-line"></div>
          </div>
        {/if}

        <!-- Flash Effect -->
        {#if showFlash}
          <div class="absolute inset-0 z-40 bg-white opacity-60 flash-overlay"></div>
        {/if}

        <!-- Union Logo -->
        <div class="absolute top-3 left-3 z-20">
          <img
            src="/images/union-logo-glyph.svg"
            alt="Union logo"
            class="w-6 h-6 opacity-60"
          />
        </div>

        <!-- Level Badge -->
        <div class="absolute top-3 right-3 z-20">
          <div class="flex items-center gap-2 bg-zinc-900/80 backdrop-blur-sm px-3 py-1.5 rounded-lg border border-zinc-700/60">
            {#if Option.isSome(dashboard.experience)
              && Option.isSome(dashboard.experience.value.current)
              && Option.isSome(dashboard.experience.value.level)}
              <img
                src="/badges/{dashboard.experience.value.current.value.level}.svg"
                alt="level badge"
                class="size-4"
              />
              <span class="text-xs text-zinc-200 font-mono font-medium">{
                dashboard.experience.value.level.value
              }</span>
            {:else}
              <Skeleton class="size-4 rounded" />
              <Skeleton class="h-3 w-6" />
            {/if}
          </div>
        </div>

        <!-- Avatar Body -->
        <div class="relative flex-1 flex items-center justify-center overflow-hidden">
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

            <!-- Main avatar -->
            <div class="relative z-10">
              <img
                src={dashboard.identity.avatar.value}
                alt="Profile avatar"
                class="w-32 h-32 object-cover rounded-full border-2 border-zinc-600/80 shadow-xl"
              />
            </div>
          {:else}
            <Skeleton class="w-32 h-32 rounded-full" />
          {/if}
        </div>

        <!-- Stats Footer -->
        <div class="bg-black/40 border-t border-zinc-700/60 px-3 py-2">
          <!-- Username -->
          <div class="text-left mb-2">
            {#if Option.isSome(dashboard.identity.username)}
              <div class="text-sm font-bold text-white font-mono uppercase tracking-widest">
                {dashboard.identity.username.value}
              </div>
            {:else}
              <Skeleton class="h-4 w-20" />
            {/if}
          </div>

          <!-- Stats Grid -->
          <div class="flex items-start justify-between text-xs font-mono">
            <!-- Activity Stats -->
            <div class="flex flex-col gap-1 text-left">
              {#if Option.isSome(dashboard.missions)
                && Option.isSome(dashboard.missions.value.available)
                && Option.isSome(dashboard.missions.value.progress)}
                <span class="text-zinc-300">{dashboard.missions.value.stats.completed}
                  Missions</span>
              {:else}
                <Skeleton class="h-3 w-16" />
              {/if}

              {#if Option.isSome(dashboard.achievements)
                && Option.isSome(dashboard.achievements.value.achieved)}
                <span class="text-zinc-300">{
                    dashboard.achievements.value.achieved.value.filter(ua =>
                      ua.progression !== undefined
                      && ua.threshold !== undefined
                      && ua.progression >= ua.threshold
                    ).length
                  } Achievements</span>
              {:else}
                <Skeleton class="h-3 w-20" />
              {/if}

              {#if Option.isSome(dashboard.rewards)
                && Option.isSome(dashboard.rewards.value.earned)
                && Option.isSome(dashboard.rewards.value.availableRewards)}
                <span class="text-zinc-300">{dashboard.rewards.value.stats.claimed} Rewards</span>
              {:else}
                <Skeleton class="h-3 w-16" />
              {/if}
            </div>

            <!-- User Stats -->
            <div class="flex flex-col gap-1 text-right">
              {#if Option.isSome(dashboard.experience)
                && Option.isSome(dashboard.experience.value.current)}
                <span class="text-zinc-300">Rank #{
                    dashboard.experience.value.current.value.rank
                  }</span>
                <span class="text-zinc-300">Level {
                    dashboard.experience.value.current.value.level
                  }</span>
                <span class="text-zinc-300">{
                    (dashboard.experience.value.current.value.total_xp ?? 0)
                    .toLocaleString()
                  } XP</span>
              {:else}
                <Skeleton class="h-3 w-16 ml-auto" />
                <Skeleton class="h-3 w-12 ml-auto" />
                <Skeleton class="h-3 w-12 ml-auto" />
              {/if}
            </div>
          </div>
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
  animation: scanAnimation 6s linear infinite;
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
    top: -2px;
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

/* Flash Effect */
.flash-overlay {
  animation: flashEffect 200ms ease-out;
}

@keyframes flashEffect {
  0% {
    opacity: 0;
  }
  50% {
    opacity: 0.8;
  }
  100% {
    opacity: 0.6;
  }
}

/* Filters */
.grayscale {
  filter: grayscale(1);
}
</style>

<!-- Celebration Overlay -->
<CelebrationOverlay
  show={showCelebration}
  totalTokens={totalAllocationTokens}
  stakingPercentage={Number(stakingPercentage)}
  onClose={handleCelebrationClose}
/>
