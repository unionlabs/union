<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import StepLayout from "../StepLayout.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

interface Props {
  onNext: () => void
}

let { onNext }: Props = $props()

// Scanning state
let isChecking = $state(false)
let checkingComplete = $state(false)
let showFlash = $state(false)
let currentStep = $state(0)

const checkingSteps = [
  "Waiting to start..",

  "Digging trhough socials..",
  "Reading achievements..",
  "Checking missions..",
  "Looking at rewards..",
  "done"
]

async function startChecking() {
  isChecking = true
  
  // Step through checking process
  for (let i = 0; i < checkingSteps.length; i++) {
    currentStep = i
    await new Promise(resolve => setTimeout(resolve, 1000))
  }
  
  isChecking = false
  
  // Flash discovery effect
  showFlash = true
  await new Promise(resolve => setTimeout(resolve, 200))
  showFlash = false
  
  // Color reveal
  await new Promise(resolve => setTimeout(resolve, 100))
  checkingComplete = true
}

function handleButtonClick() {
  if (!checkingComplete) {
    startChecking()
  } else {
    onNext()
  }
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col justify-between h-full w-full p-4">
      <div>
        <h1 class="text-2xl font-semibold">Check if eligible</h1> 
       

        <!-- Terminal Output -->
        
          <div class="font-mono text-xs text-green-400 mt-2">
    
            {#each checkingSteps.slice(0, currentStep + 1) as step, index}
              <div class="flex items-center gap-2 text-zinc-300">
                {#if index < currentStep}
                  <span class="text-accent">✓</span>
                  <span>{step}</span>
                {:else if index === currentStep && step !== "done"}
                  <span class="text-yellow-400 animate-pulse">⟳</span>
                  <span>{step}</span>
                {:else if step === "done"}
                  <span class="text-accent">✓</span>
                  <span class="text-accent">Scan complete</span>
                {/if}
              </div>
            {/each}
            
            {#if checkingComplete}
              <div class="mt-2 text-accent">
                <span>→ Profile verified successfully</span>
              </div>
            {/if}
          </div>

      </div>
      
      <Button
        size="md"
        variant="primary"
        class="flex w-full items-center justify-center gap-3"
        disabled={isChecking}
        onclick={handleButtonClick}
      >
        {checkingComplete ? "Continue" : "Start check"}
        {#if isChecking}
          <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
        {/if}
      </Button>
    </div>
  {/snippet}
  
  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-3">
      <!-- Scanning Line (outside grayscale container) -->
      {#if isChecking}
        <div class="absolute inset-3 z-30 pointer-events-none rounded-lg overflow-hidden">
          <div class="scan-line"></div>
        </div>
      {/if}

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative {!checkingComplete ? 'grayscale' : ''} transition-all duration-500">
        
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
            {#if Option.isSome(dashboard.experience) && Option.isSome(dashboard.experience.value.current)}
              <img
                src="/badges/{dashboard.experience.value.current.value.level}.svg"
                alt="level badge"
                class="size-4"
              />
              <span class="text-xs text-zinc-200 font-mono font-medium">LEVEL {dashboard.experience.value.current.value.level}</span>
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
              {#if Option.isSome(dashboard.missions) && Option.isSome(dashboard.missions.value.available) && Option.isSome(dashboard.missions.value.progress)}
                <span class="text-zinc-300">{dashboard.missions.value.stats.completed} Missions</span>
              {:else}
                <Skeleton class="h-3 w-16" />
              {/if}
              
              {#if Option.isSome(dashboard.achievements) && Option.isSome(dashboard.achievements.value.achieved)}
                <span class="text-zinc-300">{dashboard.achievements.value.achieved.value.filter(ua => 
                  ua.progression !== undefined && 
                  ua.threshold !== undefined && 
                  ua.progression >= ua.threshold
                ).length} Achievements</span>
              {:else}
                <Skeleton class="h-3 w-20" />
              {/if}
              
              {#if Option.isSome(dashboard.rewards) && Option.isSome(dashboard.rewards.value.earned) && Option.isSome(dashboard.rewards.value.availableRewards)}
                <span class="text-zinc-300">{dashboard.rewards.value.stats.claimed} Rewards</span>
              {:else}
                <Skeleton class="h-3 w-16" />
              {/if}
            </div>

            <!-- User Stats -->
            <div class="flex flex-col gap-1 text-right">
              {#if Option.isSome(dashboard.experience) && Option.isSome(dashboard.experience.value.current)}
                <span class="text-zinc-300">Rank #{dashboard.experience.value.current.value.rank}</span>
                <span class="text-zinc-300">Level {dashboard.experience.value.current.value.level}</span>
                <span class="text-zinc-300">{(dashboard.experience.value.current.value.total_xp ?? 0).toLocaleString()} XP</span>
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