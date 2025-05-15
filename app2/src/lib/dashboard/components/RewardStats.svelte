<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import ProgressBar from "$lib/components/ui/ProgressBar.svelte";
  import { page } from "$app/stores";

  // Get reward statistics
  let stats = $derived(
    Option.flatMap(dashboard.rewards, (rewardsStore) =>
      Option.flatMap(rewardsStore.earned, (_earnedData) => 
        Option.flatMap(rewardsStore.availableRewards, (_availableData) => 
          Option.some(rewardsStore.stats)
        )
      )
    )
  );

  // Check if we're on the rewards page
  let isOnRewardsPage = $derived($page.url.pathname === '/dashboard/rewards');
</script>

<Card class="flex flex-col flex-1">
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-zinc-200">Reward Stats</h3>
      {#if !isOnRewardsPage}
        <a 
          href="/dashboard/rewards" 
          class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-0.5 rounded cursor-pointer"
        >
          View all
        </a>
      {/if}
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
      {#if Option.isNone(dashboard.rewards)}
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
        <!-- Claimed Rewards -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Claimed Rewards</div>
          <div class="flex items-center gap-2">
            {#if Option.isNone(stats)}
              <Skeleton class="h-6 w-12" />
              <Skeleton class="h-3 w-8" />
            {:else}
              <div class="text-lg font-medium">
                {stats.value.claimed.toString()}
              </div>
              <div class="text-xs text-zinc-500">
                / {stats.value.total.toString()}
              </div>
            {/if}
          </div>
        </div>

        <!-- Queued Rewards -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Queued Rewards</div>
          {#if Option.isNone(stats)}
            <Skeleton class="h-6 w-12" />
          {:else}
            <div class="text-lg font-medium">
              {stats.value.queued.toString()}
            </div>
          {/if}
        </div>

        <!-- Claim Rate -->
        <div class="flex flex-col gap-1">
          <div class="text-xs text-zinc-500">Claim Rate</div>
          {#if Option.isNone(stats)}
            <Skeleton class="h-6 w-12" />
          {:else}
            <div class="text-lg font-medium">
              {`${Math.round(stats.value.claimRate)}%`}
            </div>
          {/if}
        </div>
      {/if}
    </div>
    {#if !Option.isNone(dashboard.rewards)}
      <ProgressBar 
        progress={Option.match(stats, {
          onNone: () => 0,
          onSome: (s) => s.claimRate
        })} 
      />
    {/if}
  </div>
</Card> 