<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

interface Props {
  isVerified?: boolean
  showVerificationBadge?: boolean
}

let { isVerified = true, showVerificationBadge = false }: Props = $props()
</script>

<div class="relative w-full h-64 flex flex-col">
  <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
    <!-- Union Logo -->
    <div class="absolute top-3 left-3 z-20">
      <img
        src="/images/union-logo-glyph.svg"
        alt="Union logo"
        class="w-6 h-6 opacity-60"
      />
    </div>

    <!-- Level/Verification Badge -->
    <div class="absolute top-3 right-3 z-20">
      <div class="flex items-center gap-2 bg-zinc-900/80 backdrop-blur-sm px-3 py-1.5 rounded-lg border {showVerificationBadge ? (isVerified ? 'border-green-500/60' : 'border-red-500/60') : 'border-zinc-700/60'}">
        {#if showVerificationBadge}
          <!-- Verification Badge -->
          {#if isVerified}
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
              class="text-green-500"
            >
              <path d="M9 12l2 2 4-4" />
              <circle
                cx="12"
                cy="12"
                r="10"
              />
            </svg>
            <span class="text-xs text-green-400 font-mono font-medium">HUMAN</span>
          {:else}
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
              class="text-red-500"
            >
              <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
              <path d="M12 9v4" />
              <path d="m12 17 .01 0" />
            </svg>
            <span class="text-xs text-red-400 font-mono font-medium">NOT HUMAN</span>
          {/if}
        {:else}
          <!-- Level Badge -->
          {#if Option.isSome(dashboard.experience)
            && Option.isSome(dashboard.experience.value.current)}
            <img
              src="/badges/{dashboard.experience.value.current.value.level}.svg"
              alt="level badge"
              class="size-4"
            />
            <span class="text-xs text-zinc-200 font-mono font-medium">LEVEL {
                dashboard.experience.value.current.value.level
              }</span>
          {:else}
            <Skeleton class="size-4 rounded" />
            <Skeleton class="h-3 w-6" />
          {/if}
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
            class="w-24 h-24 object-cover rounded-full border-2 border-zinc-600/80 shadow-xl"
          />
        </div>
      {:else}
        <Skeleton class="w-24 h-24 rounded-full" />
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
            <span class="text-zinc-300">{dashboard.missions.value.stats.completed} Missions</span>
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
            <span class="text-zinc-300">Rank #{dashboard.experience.value.current.value.rank}</span>
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
