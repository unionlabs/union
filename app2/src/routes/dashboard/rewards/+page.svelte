<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import RewardStats from "$lib/dashboard/components/RewardStats.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { fade } from "svelte/transition"

let rewards = $derived(
  Option.match(dashboard.rewards, {
    onNone: () => [],
    onSome: (store) => {
      // Get all rewards and sort them by status priority
      const allRewards = store.enhanced
      return allRewards.sort((a, b) => {
        // Define status priority (higher number = higher priority)
        const statusPriority: Record<string, number> = {
          handled: 4, // Fully granted rewards
          queued: 3, // Rewards being processed
          claimed: 2, // Rewards that have been claimed but not yet processed
          available: 1, // Rewards that can be claimed
        }

        // First sort by status
        const statusDiff = (statusPriority[b.status] || 0) - (statusPriority[a.status] || 0)
        if (statusDiff !== 0) {
          return statusDiff
        }

        // Then sort by claimed_at date (most recent first)
        if (a.claimed_at && b.claimed_at) {
          return new Date(b.claimed_at).getTime() - new Date(a.claimed_at).getTime()
        }

        // If only one has claimed_at, put it first
        if (a.claimed_at) {
          return -1
        }
        if (b.claimed_at) {
          return 1
        }

        // Finally sort by creation date
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
      })
    },
  }),
)

let expandedRewards = $state<number[]>([])

function toggleExpand(rewardId: number) {
  if (expandedRewards.includes(rewardId)) {
    expandedRewards = expandedRewards.filter(id => id !== rewardId)
  } else {
    expandedRewards = [...expandedRewards, rewardId]
  }
}

function getStatusBadge(status: string) {
  switch (status) {
    case "handled":
      return {
        text: "Earned",
        color: "text-accent",
        glow: true,
      }
    case "claimed":
      return {
        text: "Claimed",
        color: "text-yellow-400",
        glow: true,
      }
    case "queued":
      return {
        text: "Processing",
        color: "text-blue-400",
        glow: true,
      }
    default:
      return {
        text: "Available to Claim",
        color: "text-white",
        glow: false,
      }
  }
}

$effect(() => {
  console.log("rewards", rewards)
})
</script>

<RewardStats />

<Card>
  <div class="flex flex-col gap-4">
    {#if Option.isNone(dashboard.rewards)}
      <!-- Loading State -->
      {#each Array(3) as _}
        <Card class="flex flex-col gap-3 p-3 lg:p-4 bg-zinc-900/50 border-zinc-800">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5">
              <Skeleton class="h-4 w-32 lg:h-5 lg:w-40" />
            </div>
            <div class="flex items-center gap-1.5">
              <Skeleton class="h-4 w-16 lg:h-5 lg:w-20" />
            </div>
          </div>
          <Skeleton class="h-3 w-24 lg:h-4 lg:w-32" />
        </Card>
      {/each}
    {:else}
      {#each rewards as reward}
        {#if reward}
          {@const status = getStatusBadge(reward.status)}
          <Card
            class="flex flex-col gap-3 p-3 lg:p-4 bg-zinc-900/50 border-zinc-800 hover:border-accent/50 transition-all duration-300 relative overflow-hidden cursor-pointer"
            onclick={() => toggleExpand(reward.id)}
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-1.5">
                <h3 class="text-sm lg:text-base font-medium lg:font-bold group">
                  {reward.title}
                </h3>
              </div>
              {#if reward.status !== "available"}
                <div class="flex items-center gap-1.5">
                  <div class="relative">
                    <div
                      class="px-1.5 py-0.5 rounded-sm bg-zinc-800/80 border border-zinc-700/50 hover:scale-110 hover:border-accent/50 transition-all duration-300 flex items-center justify-center"
                    >
                      <span
                        class="text-[10px] lg:text-xs font-medium {status.color} transition-all duration-300"
                      >
                        {status.text}
                      </span>
                    </div>
                    {#if status.glow}
                      <div class="absolute inset-0 rounded-sm bg-accent/20 blur-sm"></div>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>

            {#if reward.claimed_at}
              <div class="text-[10px] lg:text-xs text-zinc-400">
                Achieved at {new Date(reward.claimed_at).toLocaleDateString()}
              </div>
            {/if}

            {#if expandedRewards.includes(reward.id)}
              <div
                class="flex flex-col gap-3"
                transition:fade
              >
                <div class="text-xs lg:text-sm text-neutral-300">
                  {reward.description}
                </div>

                {#if reward.cutoff}
                  <div class="flex justify-between text-[10px] lg:text-xs text-zinc-400">
                    <span>Deadline:</span>
                    <span>{new Date(reward.cutoff).toLocaleDateString()}</span>
                  </div>
                {/if}
              </div>
            {/if}
          </Card>
        {/if}
      {/each}
    {/if}
  </div>
</Card>
