<script lang="ts">
import MissionBoxIcon from "$lib/components/icons/MissionBoxIcon.svelte"
import MissionCompletedIcon from "$lib/components/icons/MissionCompletedIcon.svelte"
import Card from "$lib/components/ui/Card.svelte"
import ProgressBar from "$lib/components/ui/ProgressBar.svelte"
import { mdToHTML } from "$lib/markdown"
import { fade } from "svelte/transition"
import type { Mission } from "../stores/missions.svelte"
import type { UserMission } from "../stores/missions.svelte"

type Props = {
  mission: Mission
  userMissions: UserMission[]
}

const { mission, userMissions } = $props()

let progress = $derived(mission.progress)
let threshold = $derived(mission.threshold)
let percentComplete = $derived(mission.percentComplete)
let completed = $derived(mission.completed)
let started = $derived(mission.started)
let isExpired = $derived(mission.isExpired)
let isFuture = $derived(mission.isFuture)
let isCurrent = $derived(mission.isCurrent)
let showXp = $state(false)
let isExpanded = $state(false)

let userMission = $derived(userMissions.find((um: UserMission) => um.mission_id === mission.id))

$effect(() => {
  if (completed && !showXp) {
    showXp = true
  }
})

function getStatusColor() {
  if (completed) {
    return "text-accent"
  }
  if (isExpired) {
    return "text-red-500"
  }
  if (isFuture) {
    return "text-zinc-400"
  }
  return "text-accent"
}

function toggleExpand() {
  isExpanded = !isExpanded
}
</script>

<Card
  class="flex flex-col gap-4 p-4 bg-zinc-900/50 border-zinc-800 hover:border-accent/50 transition-all duration-300 relative overflow-hidden cursor-pointer"
  onclick={toggleExpand}
>
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      {#if completed}
        <MissionCompletedIcon class="size-5 text-accent" />
      {:else if isExpired}
        <MissionBoxIcon class="size-5 text-red-500" />
      {:else if isFuture}
        <MissionBoxIcon class="size-5 text-zinc-400" />
      {:else}
        <MissionBoxIcon class="size-5 text-accent" />
      {/if}
      <h3 class="text-sm lg:text-base font-medium lg:font-bold group">
        {mission.title}
      </h3>
    </div>
    <div class="flex items-center gap-2">
      <div class="relative">
        <div class="px-2 py-0.5 rounded-sm bg-zinc-800/80 border border-zinc-700/50 {showXp ? 'scale-110 border-accent/50' : ''} transition-all duration-300">
          <span
            class="text-[10px] lg:text-xs font-medium {showXp ? 'text-accent' : 'text-white'} transition-all duration-300"
          >
            {mission.xp} XP
          </span>
        </div>
        {#if showXp}
          <div class="absolute inset-0 rounded-sm bg-accent/20 blur-sm animate-pulse"></div>
        {/if}
      </div>
    </div>
  </div>

  {#if isExpanded}
    <div
      class="flex flex-col gap-4"
      transition:fade
    >
      <div class="text-xs lg:text-sm text-neutral-300 [&_a]:text-accent [&_strong]:font-bold [&_strong]:text-white">
        {#await mdToHTML(mission.description)}
          <p>Loading description...</p>
        {:then vFile}
          {@html vFile.value}
        {/await}
      </div>

      <div class="flex flex-col gap-2">
        <div class="flex justify-between text-[10px] lg:text-xs">
          <span class="text-zinc-400">Progress</span>
          <span class="text-zinc-400">
            <span class="font-medium">
              {progress}
            </span>
            / {threshold}
          </span>
        </div>
        <div class="relative">
          <ProgressBar progress={percentComplete} />
          {#if percentComplete === 100}
            <div class="absolute inset-0 bg-accent/20 animate-pulse rounded-full"></div>
          {/if}
        </div>
      </div>

      <div class="flex justify-between text-[10px] lg:text-xs text-zinc-400">
        <span>Started: {new Date(mission.start).toLocaleDateString()}</span>
        <span>Ends: {new Date(mission.end).toLocaleDateString()}</span>
      </div>
    </div>
  {/if}
</Card>
