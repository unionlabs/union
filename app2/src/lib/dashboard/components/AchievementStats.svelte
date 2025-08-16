<script lang="ts">
import { page } from "$app/stores"
import Card from "$lib/components/ui/Card.svelte"
import ProgressBar from "$lib/components/ui/ProgressBar.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"

let totalXP = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(
        achievements.available,
        (availableAchievements) =>
          Option.some(
            availableAchievements.filter(a => !achievements.isAchievementExpired(a)).reduce(
              (sum: number, a: any) => sum + (a.xp || 0),
              0,
            ),
          ),
      ),
  ),
)

let earnedXP = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(
        achievements.achieved,
        (achievedAchievements) =>
          Option.flatMap(achievements.available, (availableAchievements) =>
            Option.some(achievedAchievements.reduce((sum: number, ua: any) => {
              // Only count XP if the achievement is completed (progression >= threshold)
              if (
                ua.progression === undefined || ua.threshold === undefined
                || ua.progression < ua.threshold
              ) {
                return sum
              }
              const achievement = availableAchievements.find((a: any) =>
                a.id === ua.achievement_id
              )
              return sum + (achievement?.xp || 0)
            }, 0))),
      ),
  ),
)

// Calculate completed achievements count - only count those that have met their threshold
let completedCount = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.achieved, (achievedAchievements) =>
        Option.some(
          achievedAchievements.filter(ua =>
            ua.progression !== undefined
            && ua.threshold !== undefined
            && ua.progression >= ua.threshold
          ).length,
        )),
  ),
)

// Calculate total achievements count (excluding expired ones)
let totalCount = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(
        achievements.available,
        (availableAchievements) =>
          Option.some(
            availableAchievements.filter(a => !achievements.isAchievementExpired(a)).length,
          ),
      ),
  ),
)

// Calculate completion rate based on achievement count
let progress = $derived(
  Option.flatMap(totalCount, (total) => {
    if (total === 0) {
      return Option.none()
    }
    return Option.flatMap(
      completedCount,
      (completed) => Option.some(Math.min((completed / total) * 100, 100)),
    )
  }),
)

// Check if we're on the achievements page
let isOnAchievementsPage = $derived($page.url.pathname === "/dashboard/achievements")
</script>

<Card class="flex flex-col flex-1">
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-zinc-200">Achievement Stats</h3>
      {#if !isOnAchievementsPage}
        <a
          href="/dashboard/achievements"
          class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded cursor-pointer"
        >
          View all
        </a>
      {/if}
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
      {#if Option.isNone(dashboard.achievements)}
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
        <!-- Achievement Count -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Achievements</div>
          <div class="flex items-center gap-2">
            {#if Option.isNone(completedCount) || Option.isNone(totalCount)}
              <Skeleton class="h-6 w-12" />
              <Skeleton class="h-3 w-8" />
            {:else}
              <div class="text-lg font-medium">
                {completedCount.value.toString()}
              </div>
              <div class="text-xs text-zinc-500">
                / {totalCount.value.toString()}
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
              <div class="text-lg font-medium">
                {earnedXP.value.toString()}
              </div>
              <div class="text-xs text-zinc-500">
                / {totalXP.value.toString()}
              </div>
            {/if}
          </div>
        </div>

        <!-- Completion Rate -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Completion Rate</div>
          {#if Option.isNone(progress)}
            <Skeleton class="h-6 w-12" />
          {:else}
            <div class="text-lg font-medium">
              {`${Math.round(progress.value)}%`}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {#if !Option.isNone(dashboard.achievements)}
      <ProgressBar
        progress={Option.match(progress, {
          onNone: () => 0,
          onSome: (p) => p,
        })}
      />
    {/if}
  </div>
</Card>
