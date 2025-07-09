<script lang="ts">
  import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
  import Skeleton from "$lib/components/ui/Skeleton.svelte"
  import { dashboard } from "$lib/dashboard/stores/user.svelte"
  import { uiStore } from "$lib/stores/ui.svelte"
  import { Option } from "effect"
  import { onDestroy, onMount } from "svelte"

  let { expanded = false }: { expanded?: boolean } = $props()

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

  $effect(() => {
    if (Option.isSome(dashboard.user)) {
      checkIfNewUser()
    }
  })

  onMount(() => {
    checkIfNewUser()
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
      {#if isNewUser && expanded}
        <a
          href="/dashboard"
          class="absolute inset-0 bg-zinc-925/5 backdrop-blur-lg z-50 flex items-center justify-center cursor-pointer transition-all hover:bg-zinc-700/10 hover:backdrop-blur-md group rounded-lg"
        >
          <div class="relative w-fit transition-transform duration-300 group-hover:scale-[1.02]">
            <div class="px-2 py-0.5 rounded-sm bg-zinc-800/40 border scale-110 border-accent/40 transition-all flex items-center gap-2 group-hover:border-accent/50">
              <SpinnerIcon class="size-3 text-accent animate-spin" />
              <span class="text-sm font-medium text-accent whitespace-nowrap">Processing</span>
            </div>
            <div class="absolute inset-0 rounded-sm bg-accent/10 blur-sm animate-pulse group-hover:bg-accent/15">
            </div>
          </div>
        </a>
      {/if}

      <a
        href="/dashboard"
        class="inline-flex items-center rounded-lg hover:bg-zinc-800 transition-colors duration-200 w-full"
      >
        <!-- Avatar with Circular Progress Ring - fixed position like nav icons -->
        <span class="relative inline-block">
          <div class="m-2 flex h-8 w-8 items-center justify-center flex-shrink-0 relative">
            {#if Option.isSome(dashboard.identity.avatar)}
              <!-- Progress Ring Background - larger than avatar to be visible -->
              <svg class="absolute inset-0 w-10 h-10 -rotate-90 -top-1 -left-1" viewBox="0 0 40 40">
                <circle
                  cx="20"
                  cy="20"
                  r="18"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  class="text-zinc-700"
                />
                {#if Option.isSome(dashboard.experience) && Option.isSome(dashboard.experience.value.progress)}
                  <!-- Progress Ring -->
                  <circle
                    cx="20"
                    cy="20"
                    r="18"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-dasharray={`${2 * Math.PI * 18}`}
                    stroke-dashoffset={`${2 * Math.PI * 18 * (1 - dashboard.experience.value.progress.value / 100)}`}
                    class="text-accent transition-all duration-500 ease-out"
                  />
                {/if}
              </svg>
              <img
                src={dashboard.identity.avatar.value}
                alt=""
                class="relative w-8 h-8 rounded-full z-10"
              />
            {:else}
              <div class="relative">
                <svg class="absolute inset-0 w-10 h-10 -rotate-90 -top-1 -left-1" viewBox="0 0 40 40">
                  <circle
                    cx="20"
                    cy="20"
                    r="18"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-zinc-700"
                  />
                </svg>
                <Skeleton class="relative w-8 h-8 rounded-full z-10" />
              </div>
            {/if}
            {#if isNewUser}
              <div class="absolute -top-0.5 -right-0.5 w-3 h-3 bg-accent rounded-full animate-pulse z-20"></div>
            {/if}
          </div>
        </span>

        <!-- Text content - appears when expanded -->
        {#if expanded}
          <div class="flex w-full flex-col gap-1 whitespace-nowrap px-2 min-w-0">
            {#if Option.isSome(dashboard.identity.username)}
              <p class="text-sm font-semibold uppercase text-white truncate">
                {dashboard.identity.username.value}
              </p>
            {:else}
              <Skeleton class="h-4 w-20" />
            {/if}
            
            {#if Option.isSome(dashboard.experience) && Option.isSome(dashboard.experience.value.current) && Option.isSome(dashboard.experience.value.level)}
              <p class="text-xs text-zinc-400">
                Lvl {dashboard.experience.value.current.value.level} - {dashboard.experience.value.level.value}
              </p>
            {:else}
              <Skeleton class="h-3 w-16" />
            {/if}
          </div>
        {/if}
      </a>
    </div>
  {:else}
    <!-- Unauthenticated state -->
    <div class="overflow-hidden">
      <a
        href="/auth/sign-in"
        class="inline-flex items-center rounded-lg hover:bg-zinc-800 transition-colors duration-200 w-full"
        title="Sign In"
      >
        <!-- Icon - fixed position -->
        <span class="relative inline-block">
          <div class="m-2 flex h-8 w-8 items-center justify-center flex-shrink-0">
            <div class="w-8 h-8 rounded-full bg-gradient-to-br from-accent/20 to-accent/40 flex items-center justify-center">
              <svg class="w-4 h-4 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
          </div>
        </span>

        <!-- Text content - appears when expanded -->
        {#if expanded}
          <div class="flex w-full flex-col gap-1 whitespace-nowrap px-2">
            <p class="text-sm font-semibold text-white">Start Earning</p>
            <p class="text-xs text-zinc-400">Join & unlock rewards</p>
          </div>
        {/if}
      </a>
    </div>
  {/if}
{/if} 