<script lang="ts">
import { goto } from "$app/navigation"
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
import Button from "$lib/components/ui/Button.svelte"

import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { Option } from "effect"
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

{#if uiStore.edition === "app"}
  {#if Option.isSome(dashboard.user)}
    <div class="relative">
      {#if isNewUser}
        <a
          href="/dashboard"
          class="absolute inset-0 bg-zinc-925/5 backdrop-blur-lg z-50 flex items-center justify-center cursor-pointer transition-all hover:bg-zinc-700/10 hover:backdrop-blur-md group"
        >
          <div class="relative w-fit transition-transform duration-300 group-hover:scale-[1.02]">
            <div class="px-2 py-0.5 rounded-sm bg-zinc-800/40 border scale-110 border-accent/40 transition-all flex items-center gap-2 group-hover:border-accent/50">
              <SpinnerIcon class="size-3 text-accent animate-spin" />
              <span class="text-sm font-medium text-accent">Processing</span>
            </div>
            <div class="absolute inset-0 rounded-sm bg-accent/10 blur-sm animate-pulse group-hover:bg-accent/15">
            </div>
          </div>
        </a>
      {/if}
      <a
        href="/dashboard"
        class="hover:bg-zinc-900 flex flex-col gap-4 px-6 py-4 border-b border-zinc-900"
      >
        <div class="flex items-center gap-3">
          <!-- Avatar with Circular Progress Ring -->
          <div class="relative">
            {#if Option.isSome(dashboard.identity.avatar)}
              <!-- Progress Ring Background -->
              <svg
                class="absolute inset-0 w-12 h-12 -rotate-90"
                viewBox="0 0 48 48"
              >
                <circle
                  cx="24"
                  cy="24"
                  r="22"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  class="text-zinc-800"
                />
                {#if Option.isSome(dashboard.experience)
              && Option.isSome(dashboard.experience.value.progress)}
                  <!-- Progress Ring -->
                  <circle
                    cx="24"
                    cy="24"
                    r="22"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-dasharray={`${2 * Math.PI * 22}`}
                    stroke-dashoffset={`${
                      2 * Math.PI * 22 * (1 - dashboard.experience.value.progress.value / 100)
                    }`}
                    class="text-accent transition-all duration-500 ease-out"
                  />
                {/if}
              </svg>
              <img
                src={dashboard.identity.avatar.value}
                alt=""
                class="relative w-12 h-12 rounded-full"
              />
            {:else}
              <div class="relative">
                <svg
                  class="absolute inset-0 w-12 h-12 -rotate-90"
                  viewBox="0 0 48 48"
                >
                  <circle
                    cx="24"
                    cy="24"
                    r="22"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-zinc-800"
                  />
                </svg>
                <Skeleton class="relative w-12 h-12 rounded-full" />
              </div>
            {/if}
          </div>

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
      </a>
    </div>
  {:else}
    <div class="relative overflow-hidden">
      <!-- Gradient background -->
      <div class="absolute inset-0 bg-gradient-to-br from-accent/5 via-transparent to-accent/10">
      </div>
      <div class="absolute inset-0 bg-gradient-to-t from-zinc-900/20 to-transparent"></div>

      <div class="relative flex flex-col gap-4 px-6 py-4 border-b border-zinc-900/50">
        <div class="flex flex-col justify-between h-[88px]">
          <div class="flex flex-col gap-1">
            <h3 class="text-sm font-semibold bg-gradient-to-r from-white to-zinc-200 bg-clip-text text-transparent">
              Start Earning
            </h3>
            <p class="text-xs text-zinc-300">Join the community and unlock rewards</p>
          </div>

          <div class="flex items-end">
            <a
              href="/auth/sign-in"
              class="group text-xs inline-flex items-center gap-1.5 text-accent hover:text-white transition-all duration-300 font-medium tracking-wide"
            >
              <span>Get Started</span>
              <svg
                class="w-3.5 h-3.5 transition-transform duration-300 group-hover:translate-x-0.5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M13 7l5 5m0 0l-5 5m5-5H6"
                />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </div>
  {/if}
{/if}
