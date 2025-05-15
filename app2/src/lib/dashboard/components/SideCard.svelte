<script lang="ts">
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
import Button from "$lib/components/ui/Button.svelte"
import ProgressBar from "$lib/components/ui/ProgressBar.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte.js"
import { uiStore } from "$lib/stores/ui.svelte"
import { Effect, Option } from "effect"
import { onDestroy, onMount } from "svelte"

let isNewUser = $state(false)
let intervalId: ReturnType<typeof setInterval>

function checkIfNewUser() {
  Option.match(dashboard.user, {
    onNone: () => {
      isNewUser = false
    },
    onSome: (user) => {
      const createdAt = new Date(user.created_at).getTime()
      const oneHourAgo = Date.now() - (60 * 10 * 1000)
      isNewUser = createdAt > oneHourAgo
    },
  })
}

// Use $effect to check when user data changes
$effect(() => {
  if (Option.isSome(dashboard.user)) {
    checkIfNewUser()
  }
})

onMount(() => {
  checkIfNewUser()
  // Check every minute
  intervalId = setInterval(checkIfNewUser, 60 * 1000)
})

onDestroy(() => {
  if (intervalId) {
    clearInterval(intervalId)
  }
})
</script>

{#if Option.isSome(dashboard.user) && uiStore.activeEdition === "app"}
  <div class="relative">
    {#if isNewUser}
      <a
        href="/dashboard"
        class="absolute inset-0 bg-zinc-925/5 backdrop-blur-lg z-50 flex items-center justify-center cursor-pointer transition-all hover:bg-zinc-700/10 hover:backdrop-blur-md group"
      >
        <div class="relative w-fit transition-transform duration-300 group-hover:scale-[1.02]">
          <div
            class="px-2 py-0.5 rounded-sm bg-zinc-800/40 border scale-110 border-accent/40 transition-all flex items-center gap-2 group-hover:border-accent/50"
          >
            <SpinnerIcon class="size-3 text-accent animate-spin" />
            <span class="text-sm font-medium text-accent">Processing</span>
          </div>
          <div
            class="absolute inset-0 rounded-sm bg-accent/10 blur-sm animate-pulse group-hover:bg-accent/15"
          >
          </div>
        </div>
      </a>
    {/if}
    <a
      href="/dashboard"
      class="hover:bg-zinc-900 flex flex-col gap-4 px-6 py-4 border-b border-zinc-900"
    >
      <div class="flex items-center gap-3">
        {#if Option.isSome(dashboard.identity.avatar)}
          <img
            src={dashboard.identity.avatar.value}
            alt=""
            class="w-12 h-12 rounded-full ring-1 ring-zinc-700"
          />
        {:else}
          <Skeleton class="w-12 h-12 rounded-full" />
        {/if}
        <div class="flex flex-col gap-1 min-h-[52px] justify-center w-full">
          <div class="h-5">
            {#if Option.isSome(dashboard.identity.username)}
              <p class="text-sm font-semibold uppercase">
                {dashboard.identity.username.value}
              </p>
            {:else}
              <Skeleton class="h-5 w-32" />
            {/if}
          </div>
          <div class="flex flex-col gap-1 min-h-[36px]">
            {#if Option.isSome(dashboard.experience)}
              <div class="flex flex-col gap-1">
                <p class="text-xs text-zinc-400 h-4">
                  {#if Option.isSome(dashboard.experience.value.current)
                  && Option.isSome(dashboard.experience.value.level)}
                    Lvl {dashboard.experience.value.current.value.level} - {
                      dashboard.experience.value.level.value
                    }
                  {:else}
                    <Skeleton class="h-4 w-32" />
                  {/if}
                </p>
                <p class="text-xs text-zinc-500 h-4">
                  {#if Option.isSome(dashboard.experience.value.current)}
                    {
                      (dashboard.experience.value.current.value.current_xp ?? 0)
                      .toLocaleString()
                    }
                    / {
                      (
                        (dashboard.experience.value.current.value.current_xp ?? 0)
                        + (dashboard.experience.value.current.value.xp_required ?? 0)
                      ).toLocaleString()
                    } XP
                  {:else}
                    <Skeleton class="h-4 w-40" />
                  {/if}
                </p>
              </div>
            {:else}
              <div class="flex flex-col gap-1">
                <Skeleton class="h-4 w-32" />
                <Skeleton class="h-4 w-40" />
              </div>
            {/if}
          </div>
        </div>
      </div>
      <div class="h-2">
        {#if Option.isSome(dashboard.experience)
          && Option.isSome(dashboard.experience.value.progress)}
          <ProgressBar progress={dashboard.experience.value.progress.value} />
        {:else}
          <Skeleton class="h-2 w-full" />
        {/if}
      </div>
    </a>
  </div>
{:else}
  <div class="flex flex-col justify-center gap-3 px-6 py-4 border-b border-zinc-900">
    <div class="flex flex-col gap-1">
      <p class="text-sm font-semibold text-zinc-400">Earn Points</p>
      <p class="text-xs text-zinc-500">Connect your wallet to start earning</p>
    </div>
    <Button
      variant="secondary"
      class="w-full"
      onclick={() => window.location.href = "/auth/sign-in"}
    >
      Sign in
    </Button>
  </div>
{/if}
