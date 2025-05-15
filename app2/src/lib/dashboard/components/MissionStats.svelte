<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import ProgressBar from "$lib/components/ui/ProgressBar.svelte";
  import { page } from "$app/stores";

  let stats = $derived(
    Option.flatMap(dashboard.missions, (missions) => 
      Option.flatMap(missions.available, (availableMissions) =>
        Option.flatMap(missions.progress, (userMissions) =>
          Option.some(missions.stats)
        )
      )
    )
  );

  // Calculate total possible XP from all missions
  let totalXP = $derived(
    Option.flatMap(dashboard.missions, (missions) => 
      Option.flatMap(missions.available, (availableMissions) => 
        Option.some(availableMissions.reduce((sum, mission) => sum + (mission.xp || 0), 0))
      )
    )
  );

  // Calculate earned XP from completed missions
  let earnedXP = $derived(
    Option.flatMap(dashboard.missions, (missions) => 
      Option.flatMap(missions.progress, (userMissions) => 
        Option.flatMap(missions.available, (availableMissions) => 
          Option.some(userMissions.reduce((sum, userMission) => {
            const mission = availableMissions.find(m => m.id === userMission.mission_id);
            if (!mission || userMission.progression < userMission.threshold) return sum;
            return sum + (mission.xp || 0);
          }, 0))
        )
      )
    )
  );

  // Check if we're on the missions page
  let isOnMissionsPage = $derived($page.url.pathname === '/dashboard/missions');
</script>

<Card class="flex flex-col flex-1">
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-zinc-200">Mission Stats</h3>
      {#if !isOnMissionsPage}
        <a 
          href="/dashboard/missions" 
          class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded cursor-pointer"
        >
          View all
        </a>
      {/if}
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
      {#if Option.isNone(dashboard.missions)}
        <!-- Loading State -->
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-16" />
          <div class="flex items-center gap-2">
            <Skeleton class="h-6 w-12" />
            <Skeleton class="h-3 w-8" />
          </div>
        </div>
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-24" />
          <Skeleton class="h-6 w-12" />
        </div>
        <div class="flex flex-col gap-1">
          <Skeleton class="h-3 w-20" />
          <div class="flex items-center gap-2">
            <Skeleton class="h-6 w-8" />
            <Skeleton class="h-3 w-8" />
          </div>
        </div>
      {:else}
        <!-- Completed Missions -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Completed Missions</div>
          <div class="flex items-center gap-2">
            {#if Option.isNone(stats)}
              <Skeleton class="h-6 w-12" />
              <Skeleton class="h-3 w-8" />
            {:else}
              <div class="text-lg font-medium">
                {stats.value.completed.toString()}
              </div>
              <div class="text-xs text-zinc-500">
                / {stats.value.total.toString()}
              </div>
            {/if}
          </div>
        </div>

        <!-- Total XP -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Total XP</div>
          <div class="flex items-center gap-2">
            {#if Option.isNone(earnedXP) || Option.isNone(totalXP)}
              <Skeleton class="h-6 w-12" />
              <Skeleton class="h-3 w-8" />
            {:else}
              <span class="text-lg font-medium">
                {earnedXP.value.toString()}
              </span>
              <span class="text-xs text-zinc-500">
                / {totalXP.value.toString()}
              </span>
            {/if}
          </div>
        </div>

        <!-- Completion Rate -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Completion Rate</div>
          {#if Option.isNone(stats)}
            <Skeleton class="h-6 w-12" />
          {:else}
            <div class="text-lg font-medium">
              {`${Math.round(stats.value.completionRate)}%`}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {#if !Option.isNone(dashboard.missions) && !Option.isNone(stats)}
      <ProgressBar 
        progress={stats.value.completionRate} 
      />
    {/if}
  </div>
</Card> 