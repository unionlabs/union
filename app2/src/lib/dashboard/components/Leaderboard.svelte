<script lang="ts">
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Option } from "effect";
  import { truncate } from "$lib/utils/format";
  import { generateAvatar, isValidImageUrl } from "$lib/utils/avatar";
  import Card from "$lib/components/ui/Card.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import { page } from "$app/stores";
  import type { IntRange } from "$lib/types/number";


  /** Number of entries to show. Must be between 1 and 50 */
  const { show = 10 } = $props<{ show?: IntRange<1, 51> }>();

  // Ensure show value is between 1 and 50
  let validatedShow = $derived(
    show === undefined ? 10 : Math.min(Math.max(show, 1), 50)
  );

  // Track validated image URLs
  const imageCache = new Map<string, boolean>();
  let validatedPfps: Record<string, string> = $state({});

  // Debug logging for leaderboard data
  $effect(() => {
    console.log("[leaderboard] Current state:", {
      hasLeaderboard: Option.isSome(dashboard.leaderboard),
      hasData: Option.isSome(dashboard.leaderboard) && Option.isSome(dashboard.leaderboard.value.leaderboard),
      data: Option.isSome(dashboard.leaderboard) ? dashboard.leaderboard.value.leaderboard : null,
      show
    });
  });

  // Validate image URL and cache the result
  async function validateAndCacheImage(url: string): Promise<boolean> {
    if (imageCache.has(url)) {
      return imageCache.get(url)!;
    }
    const isValid = await isValidImageUrl(url);
    imageCache.set(url, isValid);
    return isValid;
  }

  $effect(() => {
    Option.match(dashboard.leaderboard, {
      onNone: () => {
        console.log("[leaderboard] No leaderboard data available");
      },
      onSome: (store) => {
        Option.match(store.leaderboard, {
          onNone: () => {
            console.log("[leaderboard] No leaderboard entries available");
          },
          onSome: (entries) => {
            console.log("[leaderboard] Processing leaderboard entries:", entries);
            entries.forEach(async (entry) => {
              if (entry?.user_id && entry?.pfp) {
                const isValid = await validateAndCacheImage(entry.pfp);
                validatedPfps[entry.user_id] = isValid ? entry.pfp : generateAvatar(entry.user_id);
              }
            });
          }
        });
      }
    });
  });

  // Check if we're on the leaderboard page
  let isOnLeaderboardPage = $derived($page.url.pathname === '/dashboard/leaderboard');
</script>

<Card class="flex flex-col gap-4 p-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-zinc-200">Leaderboard</h3>
    {#if !isOnLeaderboardPage}
      <a 
        href="/dashboard/leaderboard" 
        class="text-xs text-zinc-400 hover:text-white transition-colors border border-zinc-800 hover:border-zinc-700 px-2 py-1 rounded"
      >
        View more
      </a>
    {/if}
  </div>

  {#if Option.isNone(dashboard.leaderboard) || Option.isNone(dashboard.leaderboard.value.leaderboard)}
    <!-- Loading State -->
    <div class="flex flex-col gap-3">
      {#each Array(validatedShow) as _}
        <div class="flex gap-3 items-center">
          <Skeleton class="w-5 h-5" />
          <Skeleton class="w-8 h-8" />
          <div class="flex flex-col gap-1">
            <Skeleton class="h-3.5 w-28" />
            <Skeleton class="h-3 w-20" />
          </div>
          <div class="flex-1"></div>
          <Skeleton class="h-3.5 w-14" />
          <Skeleton class="w-6 h-6" />
        </div>
      {/each}
    </div>
  {:else}
    <ul class="list-none p-0 flex flex-col gap-3">
      {#each (validatedShow ? dashboard.leaderboard.value.leaderboard.value.slice(0, validatedShow) : dashboard.leaderboard.value.leaderboard.value) as entry, rank}
        {#if entry?.user_id}
          <li
            class="opacity-0 transform animate-fade-in flex gap-3 items-center py-2 px-2 transition-colors {rank === 0 ? 'bg-accent/5 hover:bg-accent/10' : rank === 1 ? 'bg-yellow-500/5 hover:bg-yellow-500/10' : rank === 2 ? 'bg-amber-700/5 hover:bg-amber-700/10' : 'hover:bg-zinc-900/50'}"
            style="--index: {rank}"
          >
            <div
              class="font-supermolot font-bold text-[13px] w-6 h-6 flex justify-center items-center {rank === 0 ? 'text-accent bg-accent/10' : rank === 1 ? 'text-yellow-500 bg-yellow-500/10' : rank === 2 ? 'text-amber-700 bg-amber-700/10' : 'text-zinc-300 bg-zinc-800/50'}"
            >
              {rank + 1}
            </div>
            <img
              alt={`Photo of ${entry.user_id}`}
              class="w-8 h-8 border {rank === 0 ? 'border-accent/20' : rank === 1 ? 'border-yellow-500/20' : rank === 2 ? 'border-amber-700/20' : 'border-zinc-800'} object-cover"
              src={validatedPfps[entry.user_id] || generateAvatar(entry.user_id)}
            />
            <div class="flex flex-col gap-0.5">
              <div class="text-sm {rank === 0 ? 'text-accent' : rank === 1 ? 'text-yellow-500' : rank === 2 ? 'text-amber-700' : 'text-zinc-100'}">{truncate(entry.user_id, 24, "end")}</div>
              <div class="text-xs text-zinc-400">
                Level {entry.level ?? 1}
                {#if entry.title}
                  <span class="hidden md:inline-block text-zinc-500 ml-1.5">
                    {entry.title}
                  </span>
                {/if}
              </div>
            </div>
            <div class="flex-1"></div>
            <div class="flex items-center gap-2">
              <div class="text-xs {rank === 0 ? 'text-accent' : rank === 1 ? 'text-yellow-500' : rank === 2 ? 'text-amber-700' : 'text-zinc-200'}">{(entry.total_xp ?? 0).toLocaleString()} XP</div>
              <img src="/badges/{entry.level ?? 1}.svg" alt="level badge" class="size-6" />
            </div>
          </li>
        {/if}
      {/each}
    </ul>
  {/if}
</Card>

<style>
  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-fade-in {
    animation: fadeIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  li:nth-child(n) {
    animation-delay: calc(var(--index) * 0.05s);
  }
</style> 