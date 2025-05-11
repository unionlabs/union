<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import Mission from "./Mission.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Card from "$lib/components/ui/Card.svelte";

  let activeTab = $state("active");
  let initialTabSet = $state(false);
  
  $effect(() => {
    if (!initialTabSet && Option.isSome(dashboard.missions)) {
      activeTab = "active";
      initialTabSet = true;
    }
  });
</script>

<Card class="flex flex-col gap-4 p-4 lg:p-6">
  <div class="flex flex-col gap-4">
    <!-- Tabs -->
    <div class="flex gap-1 relative w-fit" aria-label="Tabs">
      <button
        onclick={() => activeTab = 'active'}
        class="
          px-3 py-1.5 text-xs font-medium rounded-md transition-colors relative cursor-pointer min-w-[80px]
          {activeTab === 'active'
          ? 'bg-zinc-800 text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-1.5">
          <span>Active</span>
          {#if Option.isSome(dashboard.missions)}
            <span
              class="
                w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
                dashboard.missions.value.enhanced.some(m => !m.completed && m.isCurrent && m.started)
                ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
                : 'bg-white/10 dark:bg-white/5 backdrop-blur-sm ring-white/20'}
              "
            >
            </span>
          {/if}
        </div>
      </button>
      <button
        onclick={() => activeTab = 'upcoming'}
        class="
          px-3 py-1.5 text-xs font-medium rounded-md transition-colors relative cursor-pointer min-w-[80px]
          {activeTab === 'upcoming'
          ? 'bg-zinc-800 text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-1.5">
          <span>Upcoming</span>
          {#if Option.isSome(dashboard.missions)}
            <span
              class="
                w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
                dashboard.missions.value.enhanced.some(m => !m.completed && m.isFuture)
                ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
                : 'bg-white/10 dark:bg-white/5 backdrop-blur-sm ring-white/20'}
              "
            >
            </span>
          {/if}
        </div>
      </button>
      <button
        onclick={() => activeTab = 'completed'}
        class="
          px-3 py-1.5 text-xs font-medium rounded-md transition-colors relative cursor-pointer min-w-[80px]
          {activeTab === 'completed'
          ? 'bg-zinc-800 text-white'
          : 'text-zinc-500 hover:text-zinc-900 dark:hover:text-white'}
        "
      >
        <div class="flex items-center justify-center gap-1.5">
          <span>Completed</span>
          {#if Option.isSome(dashboard.missions)}
            <span
              class="
                w-1.5 h-1.5 rounded-full transition-all duration-300 ring-1 ring-opacity-20 {
                dashboard.missions.value.enhanced.some(m => m.completed)
                ? 'bg-green-500 animate-pulse ring-green-500 shadow-[0_0_6px_0px_rgba(34,197,94,0.6)]'
                : 'bg-white/10 dark:bg-white/5 backdrop-blur-sm ring-white/20'}
              "
            >
            </span>
          {/if}
        </div>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="flex flex-col gap-4">
      {#if Option.isSome(dashboard.missions)}
        {#if activeTab === 'active'}
          {#if dashboard.missions.value.enhanced.filter(m => !m.completed && m.isCurrent && m.started).length > 0}
            {#each dashboard.missions.value.enhanced.filter(m => !m.completed && m.isCurrent && m.started) as mission}
              <Mission mission={mission} userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []} />
            {/each}
          {:else}
            <div class="relative w-full">
              <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
                <div class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center">
                  <div class="text-center">
                    <div class="text-zinc-400 mb-2">No active missions</div>
                    <div class="text-sm text-zinc-500">Check back later for new missions</div>
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
        {:else if activeTab === 'upcoming'}
          {#if dashboard.missions.value.enhanced.filter(m => !m.completed && m.isFuture).length > 0}
            {#each dashboard.missions.value.enhanced.filter(m => !m.completed && m.isFuture) as mission}
              <Mission mission={mission} userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []} />
            {/each}
          {:else}
            <div class="relative w-full">
              <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
                <div class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center">
                  <div class="text-center">
                    <div class="text-zinc-400 mb-2">No upcoming missions</div>
                    <div class="text-sm text-zinc-500">Check back later for new missions</div>
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
        {:else if activeTab === 'completed'}
          {#if dashboard.missions.value.enhanced.filter(m => m.completed).length > 0}
            {#each dashboard.missions.value.enhanced.filter(m => m.completed) as mission}
              <Mission mission={mission} userMissions={Option.getOrNull(dashboard.missions.value.progress) ?? []} />
            {/each}
          {:else}
            <div class="relative w-full">
              <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
                <div class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center">
                  <div class="text-center">
                    <div class="text-zinc-400 mb-2">No completed missions</div>
                    <div class="text-sm text-zinc-500">Complete missions to see them here</div>
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
        {/if}
      {:else}
        <div class="relative w-full">
          <div class="absolute inset-0 flex items-center justify-center z-10 w-full h-full">
            <div class="bg-zinc-900/1 backdrop-blur-sm w-full h-full flex items-center justify-center">
              <div class="text-center">
                <div class="text-zinc-400 mb-2">Loading missions...</div>
                <div class="text-sm text-zinc-500">Please wait while we fetch your missions</div>
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