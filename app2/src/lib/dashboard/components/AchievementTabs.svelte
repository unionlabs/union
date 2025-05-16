<script lang="ts">
import { page } from "$app/state"
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import type { Achievement as AchievementType } from "../stores/achievements.svelte"
import type { UserAchievement } from "../stores/achievements.svelte"
import Achievement from "./Achievement.svelte"

// Get category and subcategory from URL
let selectedCategory = $derived(page.url.searchParams.get("category") ?? null)
let selectedSubcategory = $derived(page.url.searchParams.get("subcategory") ?? null)

// Get available categories and subcategories
let availableCategories = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.available, (availableAchievements) =>
        Option.some([
          ...new Set(availableAchievements.map(a => a.category?.title ?? "")),
        ])),
  ),
)

let availableSubcategories = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.available, (availableAchievements) =>
        Option.some([
          ...new Set(
            availableAchievements
              .filter(a => a.category?.title === selectedCategory)
              .map(a => a.subcategory?.title ?? ""),
          ),
        ])),
  ),
)

// Get category and subcategory counts
let categoryCounts = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.available, (availableAchievements) => {
        const counts: Record<string, number> = {}
        for (const achievement of availableAchievements) {
          const category = achievement.category?.title ?? ""
          counts[category] = (counts[category] || 0) + 1
        }
        return Option.some(counts)
      }),
  ),
)

let subcategoryCounts = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.available, (availableAchievements) => {
        const counts: Record<string, number> = {}
        for (
          const achievement of availableAchievements.filter(a =>
            a.category?.title === selectedCategory
          )
        ) {
          const subcategory = achievement.subcategory?.title ?? ""
          counts[subcategory] = (counts[subcategory] || 0) + 1
        }
        return Option.some(counts)
      }),
  ),
)

// Helper function to safely get array length from Option
function getOptionArrayLength<T>(option: Option.Option<T[]> | undefined): number {
  return Option.match(option ?? Option.none(), {
    onNone: () => 0,
    onSome: (arr) => arr?.length ?? 0,
  })
}

// Helper function to safely get achieved achievements
function getAchievedAchievements(
  achievements: { achieved?: Option.Option<UserAchievement[]> } | null,
): UserAchievement[] {
  return Option.match(achievements?.achieved ?? Option.none(), {
    onNone: () => [],
    onSome: (achieved) => achieved,
  })
}

// Helper function to organize chain for display
function organizeChain(chain: AchievementType[], userAchievements: UserAchievement[]): {
  completed: AchievementType[]
  current: AchievementType | null
  upcoming: AchievementType[]
} {
  const completed: AchievementType[] = []
  let current: AchievementType | null = null
  const upcoming: AchievementType[] = []

  for (const achievement of chain) {
    const userAchievement = userAchievements.find(ua => ua.achievement_id === achievement.id)
    if (!userAchievement || userAchievement.progression < userAchievement.threshold) {
      if (!current) {
        current = achievement
      } else {
        upcoming.push(achievement)
      }
    } else {
      completed.push(achievement)
    }
  }

  return { completed, current, upcoming }
}

let achievementChains = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(achievements.available, (availableAchievements) => {
        let filteredAchievements = availableAchievements

        // Filter by category if selected
        if (selectedCategory) {
          filteredAchievements = filteredAchievements.filter(a =>
            a.category?.title === selectedCategory
          )
        }

        // Filter by subcategory if selected
        if (selectedSubcategory) {
          filteredAchievements = filteredAchievements.filter(a =>
            a.subcategory?.title === selectedSubcategory
          )
        }

        const achievementsMap = new Map(
          filteredAchievements.map((achievement: AchievementType) => [achievement.id, achievement]),
        )

        // Helper function to find the first achievement in a chain
        const findChainStart = (achievement: AchievementType): AchievementType => {
          const previous = filteredAchievements.find((a: AchievementType) =>
            a.next === achievement.id
          )
          return previous ? findChainStart(previous) : achievement
        }

        const chains = filteredAchievements.reduce(
          (chains: AchievementType[][], achievement: AchievementType) => {
            // Skip if achievement is already in any chain
            if (
              chains.some((chain: AchievementType[]) =>
                chain.some((a: AchievementType) => a.id === achievement.id)
              )
            ) {
              return chains
            }

            // Find the actual start of this chain
            const chainStart = findChainStart(achievement)

            // Build chain from the start
            const chain: AchievementType[] = []
            let current: AchievementType | null = chainStart

            while (current) {
              chain.push(current)
              const nextId: number | null = current.next
              const nextAchievement: AchievementType | null = nextId !== null
                ? achievementsMap.get(nextId) ?? null
                : null
              current = nextAchievement
            }

            chains.push(chain)
            return chains
          },
          [],
        )

        return Option.some(chains)
      }),
  ),
)

let totalXP = $derived(
  Option.flatMap(
    dashboard.achievements,
    (achievements) =>
      Option.flatMap(
        achievements.available,
        (availableAchievements) =>
          Option.some(
            availableAchievements.reduce((sum: number, a: AchievementType) => sum + (a.xp || 0), 0),
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
            Option.some(achievedAchievements.reduce((sum: number, ua: UserAchievement) => {
              const achievement = availableAchievements.find((a: AchievementType) =>
                a.id === ua.achievement_id
              )
              return sum + (achievement?.xp || 0)
            }, 0))),
      ),
  ),
)

let progress = $derived(
  Option.flatMap(totalXP, (total) => {
    if (total === 0) {
      return Option.none()
    }
    return Option.flatMap(earnedXP, (earned) => Option.some(Math.min((earned / total) * 100, 100)))
  }),
)

// Add state for expanded chains
let expandedChains = $state<Set<number>>(new Set())

function toggleChain(chainIndex: number) {
  if (expandedChains.has(chainIndex)) {
    expandedChains.delete(chainIndex)
  } else {
    expandedChains.add(chainIndex)
  }
  expandedChains = new Set(expandedChains)
}

// Helper to get chain total XP
function getChainTotalXP(chain: AchievementType[]): number {
  return chain.reduce((sum, a) => sum + (a.xp || 0), 0)
}

// Helper to get chain earned XP
function getChainEarnedXP(chain: AchievementType[], userAchievements: UserAchievement[]): number {
  return chain.reduce((sum, a) => {
    const userAchievement = userAchievements.find(ua => ua.achievement_id === a.id)
    if (
      !userAchievement || userAchievement.progression === undefined
      || userAchievement.threshold === undefined
    ) {
      return sum
    }
    return sum + (userAchievement.progression >= userAchievement.threshold ? (a.xp || 0) : 0)
  }, 0)
}

// Helper to get chain progress percentage
function getChainProgress(chain: AchievementType[], userAchievements: UserAchievement[]): number {
  const completed = chain.filter(a => {
    const userAchievement = userAchievements.find(ua => ua.achievement_id === a.id)
    if (
      !userAchievement || userAchievement.progression === undefined
      || userAchievement.threshold === undefined
    ) {
      return false
    }
    return userAchievement.progression >= userAchievement.threshold
  }).length
  return (completed / chain.length) * 100
}

// Sort chains by progress
let sortedChains = $derived(
  Option.flatMap(achievementChains, (chains) => {
    const userAchievements = getAchievedAchievements(Option.getOrNull(dashboard.achievements))
    return Option.some(
      [...chains].sort((a, b) => {
        const progressA = getChainProgress(a, userAchievements)
        const progressB = getChainProgress(b, userAchievements)

        // If both are complete or both are not started, sort by total XP
        if ((progressA === 100 && progressB === 100) || (progressA === 0 && progressB === 0)) {
          const xpA = getChainTotalXP(a)
          const xpB = getChainTotalXP(b)
          return xpB - xpA // Higher XP first
        }

        // If one is complete and other isn't, complete comes first
        if (progressA === 100) {
          return -1
        }
        if (progressB === 100) {
          return 1
        }

        // Otherwise sort by progress percentage
        return progressB - progressA
      }),
    )
  }),
)
</script>

<Card class="flex flex-col gap-4">
  <div class="flex flex-col gap-4">
    <!-- Category Navigation -->
    <div class="flex items-center gap-2 text-sm overflow-x-auto whitespace-nowrap no-scrollbar p-2">
      <a
        href="?"
        class="font-medium text-white hover:text-neutral-200 transition-colors"
      >
        All ({
          Option.getOrNull(Option.flatMap(dashboard.achievements, (achievements) =>
          Option.flatMap(achievements.available, (availableAchievements) =>
            Option.some(availableAchievements.length)))) ?? 0
        })
      </a>

      {#if !selectedCategory}
        {#each Option.getOrNull(availableCategories) ?? [] as category}
          <div class="breadcrumb-dot"></div>
          <a
            href="?category={encodeURIComponent(category)}"
            class="font-medium text-neutral-400 hover:text-white transition-colors capitalize"
          >
            {category} ({Option.getOrNull(categoryCounts)?.[category] ?? 0})
          </a>
        {/each}
      {:else}
        <div class="breadcrumb-arrow"></div>
        <a
          href="?category={encodeURIComponent(selectedCategory)}"
          class="font-medium text-white transition-colors capitalize"
        >
          {selectedCategory} ({Option.getOrNull(categoryCounts)?.[selectedCategory] ?? 0})
        </a>

        {#if !selectedSubcategory}
          {#each Option.getOrNull(availableSubcategories) ?? [] as subcategory}
            <div class="breadcrumb-dot"></div>
            <a
              href="?category={encodeURIComponent(selectedCategory)}&subcategory={encodeURIComponent(subcategory)}"
              class="font-medium text-neutral-400 hover:text-white transition-colors capitalize"
            >
              {subcategory} ({Option.getOrNull(subcategoryCounts)?.[subcategory] ?? 0})
            </a>
          {/each}
        {:else}
          <div class="breadcrumb-arrow"></div>
          <span class="font-medium text-white capitalize">
            {selectedSubcategory} ({
              Option.getOrNull(subcategoryCounts)?.[selectedSubcategory] ?? 0
            })
          </span>
        {/if}
      {/if}
    </div>

    <!-- Achievement Chains -->
    <div class="flex flex-col gap-4">
      {#if Option.isSome(dashboard.achievements)}
        {#if getOptionArrayLength(achievementChains) > 0}
          {#each Option.getOrNull(sortedChains) ?? [] as chain, chainIndex}
            {@const organizedChain = organizeChain(
          chain,
          getAchievedAchievements(Option.getOrNull(dashboard.achievements)),
        )}
            {@const isExpanded = expandedChains.has(chainIndex)}
            {@const chainTotalXP = getChainTotalXP(chain)}
            {@const chainEarnedXP = getChainEarnedXP(
          chain,
          getAchievedAchievements(Option.getOrNull(dashboard.achievements)),
        )}
            {@const chainProgress = getChainProgress(
          chain,
          getAchievedAchievements(Option.getOrNull(dashboard.achievements)),
        )}

            <Card
              class="flex flex-col gap-2 p-4 bg-zinc-900/50 border border-zinc-800 {chainProgress === 100 ? 'border-accent/20' : ''} transition-all duration-200 hover:border-zinc-600/50"
            >
              <!-- Chain Header - Always visible -->
              <div
                class="flex items-center justify-between cursor-pointer group"
                onclick={() => toggleChain(chainIndex)}
                onkeydown={(e) => e.key === "Enter" && toggleChain(chainIndex)}
                role="button"
                tabindex="0"
              >
                <div class="flex flex-col gap-1">
                  <div class="flex items-center gap-2">
                    <h3 class="text-sm font-medium group-hover:text-white transition-colors">
                      {chain[0]?.title ?? "Chain"}
                    </h3>
                  </div>
                  <div
                    class="flex items-center gap-2 text-xs text-zinc-400 group-hover:text-zinc-300 transition-colors"
                  >
                    <span>Progress</span>
                    <span>{organizedChain.completed.length} / {chain.length} completed</span>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <div class="relative">
                    <div
                      class="px-1.5 py-0.5 rounded-sm bg-zinc-800/80 border border-zinc-700/50 {chainProgress === 100 ? 'scale-110 border-accent/50' : ''} transition-all duration-300 flex items-center justify-center"
                    >
                      <span
                        class="text-[10px] lg:text-xs font-medium {chainProgress === 100 ? 'text-accent' : 'text-white'} transition-all duration-300"
                      >
                        {chainEarnedXP} / {chainTotalXP} XP
                      </span>
                    </div>
                    {#if chainProgress === 100}
                      <div class="absolute inset-0 rounded-sm bg-accent/20 blur-sm animate-pulse">
                      </div>
                    {/if}
                  </div>
                </div>
              </div>

              <!-- Chain Content - Only visible when expanded -->
              {#if isExpanded}
                {@const availableRewards = chain.flatMap(a =>
            a.reward_achievements?.map(r => r.rewards) ?? []
          ).filter(r => {
            if (!r || !Array.isArray(r)) {
              return false
            }
            return r.some(reward =>
              !reward.cutoff || new Date(reward.cutoff) > new Date()
            )
          })}
                {@const earnedRewards = organizedChain.completed.flatMap(a =>
            a.reward_achievements?.map(r => r.rewards) ?? []
          ).filter(r => {
            if (!r || !Array.isArray(r)) {
              return false
            }
            return r.some(reward =>
              !reward.cutoff || new Date(reward.cutoff) > new Date()
            )
          })}
                <div class="flex flex-col gap-2 mt-2 pt-2 border-t border-zinc-800">
                  <!-- Chain Rewards -->
                  {#if availableRewards.length > 0}
                    <div class="flex flex-col gap-1.5">
                      <div class="text-xs text-zinc-400">Available Rewards</div>
                      <div class="flex flex-wrap gap-2">
                        {#each availableRewards as rewards}
                          {#each rewards as reward}
                            <div
                              class="flex items-center gap-1 px-2 py-1 rounded bg-zinc-800/50 text-xs font-medium text-neutral-300 hover:bg-zinc-800/80 transition-colors"
                            >
                              <span>{reward.title}</span>
                              {#if reward.cutoff}
                                <span class="text-[10px] text-zinc-400">
                                  until {new Date(reward.cutoff).toLocaleDateString()}
                                </span>
                              {/if}
                            </div>
                          {/each}
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- User Rewards -->
                  {#if earnedRewards.length > 0}
                    <div class="flex flex-col gap-1.5">
                      <div class="text-xs text-zinc-400">Earned Rewards</div>
                      <div class="flex flex-wrap gap-2">
                        {#each earnedRewards as rewards}
                          {#each rewards as reward}
                            <div
                              class="flex items-center gap-1 px-2 py-1 rounded bg-accent/10 border border-accent/20 text-xs font-medium text-accent"
                            >
                              <span>{reward.title}</span>
                              {#if reward.cutoff}
                                <span class="text-[10px] text-accent/70">
                                  until {new Date(reward.cutoff).toLocaleDateString()}
                                </span>
                              {/if}
                            </div>
                          {/each}
                        {/each}
                      </div>
                    </div>
                  {/if}

                  <!-- Achievement List -->
                  <div class="flex flex-col gap-2">
                    {#if organizedChain.completed.length > 0}
                      <div class="flex flex-col gap-2">
                        {#each organizedChain.completed as achievement}
                          <Achievement
                            achievement={achievement}
                            userAchievements={getAchievedAchievements(Option.getOrNull(dashboard.achievements))}
                            isCompleted={true}
                          />
                        {/each}
                      </div>
                    {/if}

                    {#if organizedChain.current}
                      <div class="flex flex-col gap-2">
                        <Achievement
                          achievement={organizedChain.current}
                          userAchievements={getAchievedAchievements(Option.getOrNull(dashboard.achievements))}
                          isCurrent={true}
                        />
                      </div>
                    {/if}

                    {#if organizedChain.upcoming.length > 0}
                      <div class="flex flex-col gap-2">
                        {#each organizedChain.upcoming as achievement}
                          <Achievement
                            achievement={achievement}
                            userAchievements={getAchievedAchievements(Option.getOrNull(dashboard.achievements))}
                            isNext={true}
                          />
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
              {/if}
            </Card>
          {/each}
        {:else}
          <div class="relative w-full">
            <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
              <div
                class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
              >
                <div class="text-center">
                  <div class="text-zinc-400 mb-2">No achievements available</div>
                  <div class="text-sm text-zinc-500">Complete achievements to unlock more</div>
                </div>
              </div>
            </div>
            <div class="flex flex-col gap-4">
              {#each Array(3) as _}
                <Card class="flex flex-col gap-4 p-4 bg-zinc-900/50 border-zinc-800">
                  <div class="flex flex-col gap-2">
                    <div class="flex items-center justify-between">
                      <Skeleton class="h-6 w-48" />
                      <div class="flex items-center gap-2">
                        <Skeleton class="h-4 w-16" />
                        <Skeleton class="h-4 w-20" />
                      </div>
                    </div>
                    <Skeleton class="h-4 w-full" />
                  </div>

                  <div class="flex flex-col gap-2">
                    <div class="flex justify-between text-xs">
                      <Skeleton class="h-3 w-16" />
                      <Skeleton class="h-3 w-20" />
                    </div>
                    <Skeleton class="h-2 w-full rounded-full" />
                  </div>

                  <div class="flex justify-between">
                    <Skeleton class="h-3 w-32" />
                    <Skeleton class="h-3 w-32" />
                  </div>
                </Card>
              {/each}
            </div>
          </div>
        {/if}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div
              class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
            >
              <div class="text-center">
                <div class="text-zinc-400 mb-2">Loading achievements...</div>
                <div class="text-sm text-zinc-500">
                  Please wait while we fetch your achievements
                </div>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4">
            {#each Array(3) as _}
              <Card class="flex flex-col gap-4 p-4 bg-zinc-900/50 border-zinc-800">
                <div class="flex flex-col gap-2">
                  <div class="flex items-center justify-between">
                    <Skeleton class="h-6 w-48" />
                    <div class="flex items-center gap-2">
                      <Skeleton class="h-4 w-16" />
                      <Skeleton class="h-4 w-20" />
                    </div>
                  </div>
                  <Skeleton class="h-4 w-full" />
                </div>

                <div class="flex flex-col gap-2">
                  <div class="flex justify-between text-xs">
                    <Skeleton class="h-3 w-16" />
                    <Skeleton class="h-3 w-20" />
                  </div>
                  <Skeleton class="h-2 w-full rounded-full" />
                </div>

                <div class="flex justify-between">
                  <Skeleton class="h-3 w-32" />
                  <Skeleton class="h-3 w-32" />
                </div>
              </Card>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</Card>

<style>
.breadcrumb-arrow {
  width: 0;
  height: 0;
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
  border-left: 6px solid white;
  margin: 0 0.5rem;
}

.breadcrumb-dot {
  width: 6px;
  height: 6px;
  background-color: rgb(115 115 115);
  border-radius: 50%;
  margin: 0 0.5rem;
}
</style>
