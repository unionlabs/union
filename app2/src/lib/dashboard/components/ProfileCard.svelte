<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import ProgressBar from "$lib/components/ui/ProgressBar.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { Option } from "effect"
import { Effect } from "effect"
</script>

<Card class="flex flex-col gap-4 flex-1">
  <div class="flex flex-col lg:flex-row gap-4 h-full">
    {#if Option.isSome(dashboard.identity.avatar)}
      <div class="w-full lg:w-48 h-full">
        <div class="aspect-square w-full">
          <img
            src={dashboard.identity.avatar.value}
            alt=""
            class="w-full h-full object-cover object-center rounded-lg"
          />
        </div>
      </div>
    {:else}
      <div class="w-full lg:w-48 h-full">
        <div class="aspect-square w-full">
          <Skeleton class="w-full h-full rounded-lg" />
        </div>
      </div>
    {/if}

    <div class="flex flex-col gap-4 flex-1">
      <div class="flex flex-col gap-2">
        <div class="flex items-start justify-between">
          <div class="flex flex-col gap-1">
            {#if Option.isSome(dashboard.identity.username)}
              <h1 class="text-2xl font-bold uppercase">
                {dashboard.identity.username.value}
              </h1>
              {#if Option.isSome(dashboard.experience)
                && Option.isSome(dashboard.experience.value.current)}
                <div class="text-sm text-zinc-400">
                  Rank #{dashboard.experience.value.current.value.rank} • {
                    dashboard.experience.value.current.value.total_xp?.toLocaleString()
                    ?? 0
                  } XP
                </div>
              {/if}
            {:else}
              <Skeleton class="h-8 w-48" />
              <Skeleton class="h-4 w-16" />
            {/if}
          </div>
          <button
            class="p-2 rounded-lg border border-zinc-800 hover:bg-zinc-900 transition-colors group cursor-pointer"
            onclick={() => runPromise(dashboard.logout())}
            aria-label="Logout"
          >
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
              class="text-zinc-400 group-hover:text-red-500 transition-colors"
            >
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line
                x1="21"
                y1="12"
                x2="9"
                y2="12"
              />
            </svg>
          </button>
        </div>
      </div>

      {#if Option.isSome(dashboard.experience)
          && Option.isSome(dashboard.experience.value.current)}
        <div class="flex flex-col gap-4 w-full mt-auto">
          <div class="flex items-center gap-3">
            {#if Option.isSome(dashboard.experience.value.current)}
              <img
                src="/badges/{dashboard.experience.value.current.value.level}.svg"
                alt="level badge"
                class="size-8 sm:size-12"
              />
            {:else}
              <Skeleton class="size-8 sm:size-12 rounded-lg" />
            {/if}
            <div class="flex flex-col gap-2 flex-1">
              <div class="flex justify-between text-xs items-end">
                <div class="flex flex-col text-neutral-400">
                  {#if Option.isSome(dashboard.experience.value.current)}
                    {#if Option.isSome(dashboard.experience.value.level)}
                      <span class="text-union-400">{dashboard.experience.value.level.value}</span>
                    {/if}
                    <span>Level {dashboard.experience.value.current.value.level}</span>
                  {:else}
                    <Skeleton class="h-4 w-16" />
                    <Skeleton class="h-4 w-24 mt-1" />
                  {/if}
                </div>
                <div class="flex flex-col items-end">
                  {#if Option.isSome(dashboard.experience.value.current)}
                    <div class="flex flex-col text-end">
                      {#if Option.isSome(dashboard.experience.value.next)}
                        <span class="text-neutral-400">→ {
                            dashboard.experience.value.next.value.title
                          }</span>
                      {/if}
                      <span class="text-white">
                        {
                          (dashboard.experience.value.current.value.current_xp
                          ?? 0).toLocaleString()
                        } /
                        {
                          ((dashboard.experience.value.current.value.current_xp
                          ?? 0)
                          + (dashboard.experience.value.current.value
                            .xp_required ?? 0)).toLocaleString()
                        } XP
                      </span>
                    </div>
                  {:else}
                    <Skeleton class="h-4 w-32" />
                    <Skeleton class="h-4 w-24 mt-1" />
                  {/if}
                </div>
              </div>
              {#if Option.isSome(dashboard.experience.value.progress)}
                <ProgressBar progress={dashboard.experience.value.progress.value} />
              {:else}
                <Skeleton class="h-2 w-full rounded-full" />
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <div class="flex flex-col gap-4 w-full mt-auto">
          <div class="flex items-center gap-3">
            <Skeleton class="size-8 sm:size-12 rounded-lg" />
            <div class="flex flex-col gap-2 flex-1">
              <div class="flex justify-between text-xs items-end">
                <div class="flex flex-col">
                  <Skeleton class="h-4 w-16" />
                  <Skeleton class="h-4 w-24 mt-1" />
                </div>
                <div class="flex flex-col items-end">
                  <Skeleton class="h-4 w-32" />
                  <Skeleton class="h-4 w-24 mt-1" />
                </div>
              </div>
              <Skeleton class="h-2 w-full rounded-full" />
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</Card>
