<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { SupabaseClient } from "$lib/dashboard/client"
import RewardStats from "$lib/dashboard/components/RewardStats.svelte"
import { requestRole } from "$lib/dashboard/queries/private"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { Effect, Option, pipe } from "effect"
import { fade } from "svelte/transition"

let rewards = $derived(
  Option.match(dashboard.rewards, {
    onNone: () => [],
    onSome: (store) => {
      // Get all rewards and sort them by status priority
      const allRewards = store.enhanced
        // Filter to only show earned (handled) and claimed rewards
        .filter(reward => reward.status === "handled" || reward.status === "claimed")
      return allRewards.sort((a, b) => {
        // Define status priority (higher number = higher priority)
        const statusPriority: Record<string, number> = {
          handled: 4, // Fully granted rewards
          claimed: 2, // Rewards that have been claimed but not yet processed
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
let loading = $state<Record<string, boolean>>({})

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

const handleRoleRequest = (reward: typeof rewards[number]) =>
  pipe(
    Effect.sync(() => loading[reward.id] = true),
    Effect.flatMap(() => SupabaseClient),
    Effect.flatMap((client) => Effect.tryPromise(() => client.auth.getSession())),
    Effect.flatMap(({ data: { session } }) => {
      if (!session?.user.id) {
        return Effect.fail(new Error("No authenticated user found"))
      }
      return requestRole(session.user.id, reward.id.toString())
    }),
    Effect.ensuring(Effect.sync(() => loading[reward.id] = false)),
  )
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
              <div class="flex items-center gap-2">
                {#if reward.status !== "available"}
                  <div class="flex items-center gap-1.5">
                    <div class="relative">
                      <div class="px-1.5 py-0.5 rounded-sm bg-zinc-800/80 border border-zinc-700/50 hover:scale-110 hover:border-accent/50 transition-all duration-300 flex items-center justify-center">
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

                <!-- Request role button -->
                {#if reward.type === 1}
                  <button
                    class="p-1 text-accent rounded hover:bg-accent/20 transition-colors disabled:opacity-50"
                    disabled={loading[reward.id]}
                    onclick={(e) => {
                      e.stopPropagation()
                      runPromise(handleRoleRequest(reward))
                    }}
                    title="Request role"
                    aria-label="Request role"
                  >
                    {#if loading[reward.id]}
                      <svg
                        class="w-4 h-4 animate-spin"
                        fill="none"
                        viewBox="0 0 24 24"
                      >
                        <circle
                          class="opacity-25"
                          cx="12"
                          cy="12"
                          r="10"
                          stroke="currentColor"
                          stroke-width="4"
                        >
                        </circle>
                        <path
                          class="opacity-75"
                          fill="currentColor"
                          d="m4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                        >
                        </path>
                      </svg>
                    {:else}
                      <svg
                        class="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                        >
                        </path>
                      </svg>
                    {/if}
                  </button>
                {/if}
              </div>
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
