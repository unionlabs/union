<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import Mission from "./Mission.svelte"
</script>

<div class="flex flex-col gap-4">
  {#if Option.isSome(dashboard.missions)}
    <!-- Active Missions Card -->
    <Card class="flex flex-col gap-4 p-4 lg:p-6">
      <h2 class="text-lg font-semibold flex items-center gap-2">
        Active
      </h2>
      {#if dashboard.missions.value.enhanced.filter(m => !m.completed && m.isCurrent && m.started)
        .length > 0}
        {#each dashboard.missions.value.enhanced.filter(m =>
        !m.completed && m.isCurrent && m.started
      ) as
          mission
        }
          <Mission
            mission={mission}
            userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []}
          />
        {/each}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div
              class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
            >
              <div class="text-center">
                <div class="text-zinc-400 mb-2">No active missions</div>
                <div class="text-sm text-zinc-500">Check back later for new missions</div>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4 opacity-30 pointer-events-none">
            {#each Array(2) as _}
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
    </Card>

    <!-- Upcoming Missions Card -->
    <Card class="flex flex-col gap-4 p-4 lg:p-6">
      <h2 class="text-lg font-semibold flex items-center gap-2">
        Upcoming
      </h2>
      {#if dashboard.missions.value.enhanced.filter(m => !m.completed && m.isFuture).length > 0}
        {#each dashboard.missions.value.enhanced.filter(m => !m.completed && m.isFuture) as mission}
          <Mission
            mission={mission}
            userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []}
          />
        {/each}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div
              class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
            >
              <div class="text-center">
                <div class="text-zinc-400 mb-2">No upcoming missions</div>
                <div class="text-sm text-zinc-500">Check back later for new missions</div>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4 opacity-30 pointer-events-none">
            {#each Array(2) as _}
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
    </Card>

    <!-- Completed Missions Card -->
    <Card class="flex flex-col gap-4 p-4 lg:p-6">
      <h2 class="text-lg font-semibold flex items-center gap-2">
        Completed
      </h2>
      {#if dashboard.missions.value.enhanced.filter(m => m.completed).length > 0}
        {#each dashboard.missions.value.enhanced.filter(m => m.completed) as mission}
          <Mission
            mission={mission}
            userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []}
          />
        {/each}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div
              class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
            >
              <div class="text-center">
                <div class="text-zinc-400 mb-2">No completed missions</div>
                <div class="text-sm text-zinc-500">Complete missions to see them here</div>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4 opacity-30 pointer-events-none">
            {#each Array(2) as _}
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
    </Card>

    <!-- Expired & Uncompleted Missions Card -->
    <Card class="flex flex-col gap-4 p-4 lg:p-6">
      <h2 class="text-lg font-semibold">Expired</h2>
      {#if dashboard.missions.value.expiredUncompleted
        && dashboard.missions.value.expiredUncompleted.length > 0}
        {#each dashboard.missions.value.expiredUncompleted as mission}
          <Mission
            mission={mission}
            userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []}
          />
        {/each}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div
              class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center"
            >
              <div class="text-center">
                <div class="text-zinc-400 mb-2">No expired missions</div>
                <div class="text-sm text-zinc-500">
                  Missions that passed their deadline without completion will appear here.
                </div>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-4 opacity-30 pointer-events-none">
            {#each Array(1) as _}
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
    </Card>
  {:else}
    <!-- Loading State for all cards -->
    <div class="flex flex-col gap-4">
      {#each Array(3) as _}
        <Card class="flex flex-col gap-4 p-4 lg:p-6">
          <Skeleton class="h-6 w-32 mb-2" />
          <div class="flex flex-col gap-4">
            {#each Array(1) as __}
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
        </Card>
      {/each}
      <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
        <div class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center">
          <div class="text-center">
            <div class="text-zinc-400 mb-2">Loading missions...</div>
            <div class="text-sm text-zinc-500">Please wait while we fetch your missions</div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
