<script lang="ts">
import { beforeNavigate } from "$app/navigation"
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { Option } from "effect"
import { onDestroy, onMount } from "svelte"
import { slide } from "svelte/transition"
import { navigation } from "../AppNav/config"

interface Props {
  onclose: () => void
}

const { onclose }: Props = $props()

let isNewUser = $state(false)
let intervalId: ReturnType<typeof setInterval>
let expandedSections = $state<Set<string>>(new Set())

function toggleSection(itemTitle: string) {
  if (expandedSections.has(itemTitle)) {
    expandedSections.delete(itemTitle)
  } else {
    expandedSections.add(itemTitle)
  }
  expandedSections = new Set(expandedSections) // Trigger reactivity
}

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

beforeNavigate(onclose)
</script>

<div
  class="md:hidden border-b border-zinc-900 bg-zinc-950"
  transition:slide={{ duration: 200 }}
>
  <!-- Profile Section -->
  {#if uiStore.edition === "app"}
    {#if Option.isSome(dashboard.user)}
      <div class="relative border-b border-zinc-900">
        {#if isNewUser}
          <a
            href="/dashboard"
            class="absolute inset-0 bg-zinc-925/5 backdrop-blur-lg z-50 flex items-center justify-center cursor-pointer transition-all hover:bg-zinc-700/10 hover:backdrop-blur-md group"
            onclick={onclose}
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
          class="hover:bg-zinc-900 flex items-center gap-4 px-4 py-4"
          onclick={onclose}
        >
          <!-- Avatar with Circular Progress Ring -->
          <div class="relative">
            {#if Option.isSome(dashboard.identity.avatar)}
              <!-- Progress Ring Background -->
              <svg
                class="absolute inset-0 w-14 h-14 -rotate-90 -top-1 -left-1"
                viewBox="0 0 56 56"
              >
                <circle
                  cx="28"
                  cy="28"
                  r="26"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  class="text-zinc-700"
                />
                {#if Option.isSome(dashboard.experience)
              && Option.isSome(dashboard.experience.value.progress)}
                  <!-- Progress Ring -->
                  <circle
                    cx="28"
                    cy="28"
                    r="26"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-dasharray={`${2 * Math.PI * 26}`}
                    stroke-dashoffset={`${
                      2 * Math.PI * 26 * (1 - dashboard.experience.value.progress.value / 100)
                    }`}
                    class="text-accent transition-all duration-500 ease-out"
                  />
                {/if}
              </svg>
              <img
                src={dashboard.identity.avatar.value}
                alt=""
                class="relative w-12 h-12 rounded-full z-10"
              />
            {:else}
              <div class="relative">
                <svg
                  class="absolute inset-0 w-14 h-14 -rotate-90 -top-1 -left-1"
                  viewBox="0 0 56 56"
                >
                  <circle
                    cx="28"
                    cy="28"
                    r="26"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-zinc-700"
                  />
                </svg>
                <Skeleton class="relative w-12 h-12 rounded-full z-10" />
              </div>
            {/if}
            {#if isNewUser}
              <div class="absolute -top-0.5 -right-0.5 w-3 h-3 bg-accent rounded-full animate-pulse z-20">
              </div>
            {/if}
          </div>

          <div class="flex flex-col gap-1 min-w-0 flex-1">
            <div class="h-5">
              {#if Option.isSome(dashboard.identity.username)}
                <p class="text-sm font-semibold uppercase text-white">
                  {dashboard.identity.username.value}
                </p>
              {:else}
                <Skeleton class="h-5 w-32" />
              {/if}
            </div>
            <div class="flex flex-col gap-1">
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
                        ((dashboard.experience.value.current.value.current_xp ?? 0)
                        + (dashboard.experience.value.current.value.xp_required ?? 0))
                        .toLocaleString()
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
        </a>
      </div>
    {:else}
      <!-- Unauthenticated state -->
      <div class="relative overflow-hidden border-b border-zinc-900">
        <div class="absolute inset-0 bg-gradient-to-br from-accent/5 via-transparent to-accent/10">
        </div>
        <div class="absolute inset-0 bg-gradient-to-t from-zinc-900/20 to-transparent"></div>

        <div class="relative px-4 py-4">
          <a
            href="/auth/sign-in"
            class="flex items-center gap-4 hover:bg-zinc-800/50 rounded-lg p-2 transition-colors"
            onclick={onclose}
          >
            <div class="w-12 h-12 rounded-full bg-gradient-to-br from-accent/20 to-accent/40 flex items-center justify-center flex-shrink-0">
              <svg
                class="w-6 h-6 text-accent"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                />
              </svg>
            </div>
            <div class="flex flex-col gap-1">
              <h3 class="text-sm font-semibold text-white">Start Earning</h3>
              <p class="text-xs text-zinc-300">Join the community and unlock rewards</p>
            </div>
          </a>
        </div>
      </div>
    {/if}
  {/if}

  <!-- Navigation Section -->
  <nav class="px-2 py-4 space-y-1">
    {#each navigation as section}
      {#if section.title !== "Developer" || uiStore.showDeveloperPages}
        {#if section.title && section.title !== "More Union"}
          <div class="px-3 py-2 text-xs font-semibold text-zinc-400 uppercase tracking-wide">
            {section.title}
          </div>
        {/if}

        {#each section.items as item}
          {#if item.subroutes}
            <!-- Expandable item with subroutes -->
            <button
              class="flex items-center gap-3 px-3 py-2 text-sm text-zinc-300 hover:text-white hover:bg-zinc-800 rounded-md transition-colors w-full text-left"
              onclick={() => toggleSection(item.title)}
            >
              <item.icon class="w-5 h-5 flex-shrink-0" />
              <span class="flex-1">{item.title}</span>
              <!-- Chevron icon -->
              <svg
                class="w-4 h-4 transition-transform duration-200 {expandedSections.has(item.title) ? 'rotate-180' : ''}"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>

            {#if expandedSections.has(item.title)}
              <div
                class="ml-8 space-y-1 overflow-hidden"
                transition:slide={{ duration: 200 }}
              >
                {#each item.subroutes as subroute}
                  {#if !subroute.editions || subroute.editions.includes(uiStore.edition)}
                    <a
                      href={subroute.path}
                      class="flex items-center gap-2 px-3 py-1.5 text-sm text-zinc-400 hover:text-zinc-300 hover:bg-zinc-800/50 rounded-md transition-colors"
                      onclick={onclose}
                    >
                      <span>{subroute.title}</span>
                      {#if subroute.new}
                        <span
                          class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
                        >
                          New
                        </span>
                      {/if}
                    </a>
                  {/if}
                {/each}
              </div>
            {/if}
          {:else}
            <!-- Regular navigation item -->
            <a
              href={item.path}
              target={item.external ? "_blank" : undefined}
              rel={item.external ? "noopener noreferrer" : undefined}
              class="flex items-center gap-3 px-3 py-2 text-sm text-zinc-300 hover:text-white hover:bg-zinc-800 rounded-md transition-colors"
              onclick={onclose}
            >
              <item.icon class="w-5 h-5 flex-shrink-0" />
              <span>{item.title}</span>
              {#if item.external}
                <svg
                  class="w-3 h-3 ml-auto"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M4.25 5.5a.75.75 0 00-.75.75v8.5c0 .414.336.75.75.75h8.5a.75.75 0 00.75-.75v-4a.75.75 0 011.5 0v4A2.25 2.25 0 0112.75 17h-8.5A2.25 2.25 0 012 14.75v-8.5A2.25 2.25 0 014.25 4h5a.75.75 0 010 1.5h-5z"
                    clip-rule="evenodd"
                  />
                  <path
                    fill-rule="evenodd"
                    d="M6.194 12.753a.75.75 0 001.06.053L16.5 4.44v2.81a.75.75 0 001.5 0v-4.5a.75.75 0 00-.75-.75h-4.5a.75.75 0 000 1.5h2.553l-9.056 8.194a.75.75 0 00-.053 1.06z"
                    clip-rule="evenodd"
                  />
                </svg>
              {/if}
            </a>
          {/if}
        {/each}
      {/if}
    {/each}
  </nav>
</div>
