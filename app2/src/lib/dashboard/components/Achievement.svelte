<script lang="ts">
import MissionBoxIcon from "$lib/components/icons/MissionBoxIcon.svelte"
import MissionCompletedIcon from "$lib/components/icons/MissionCompletedIcon.svelte"
import ProgressBar from "$lib/components/ui/ProgressBar.svelte"
import { mdToHTML } from "$lib/markdown"
import { fade } from "svelte/transition"
import type { Achievement as AchievementType } from "../stores/achievements.svelte"
import type { UserAchievement } from "../stores/achievements.svelte"

type Props = {
  achievement: AchievementType
  userAchievements: UserAchievement[]
  isCurrent?: boolean
  isNext?: boolean
  isCompleted?: boolean
}

const { achievement, userAchievements, isCurrent = false, isNext = false, isCompleted = false } =
  $props()

let showXp = $state(false)

let userAchievement = $derived(
  userAchievements.find((ua: UserAchievement) => ua.achievement_id === achievement.id),
)
let progress = $derived(
  userAchievement ? (userAchievement.progression / userAchievement.threshold) * 100 : 0,
)
let completed = $derived(
  userAchievement && userAchievement.progression >= userAchievement.threshold,
)

$effect(() => {
  if (completed && !showXp) {
    showXp = true
  }
})

function getStatusColor() {
  if (completed) {
    return "text-accent"
  }
  if (isNext) {
    return "text-zinc-400"
  }
  return "text-accent"
}
</script>

<button
  type="button"
  class="
    flex flex-col gap-3 p-3 rounded-lg transition-all duration-300 cursor-pointer relative w-full text-left
    {isCurrent ? 'bg-zinc-800/50' : 'hover:bg-zinc-800/30'}
  "
>
  {#if !isCompleted && !isNext}
    <div class="absolute -top-3 left-1/2 w-0.5 h-3 bg-zinc-700"></div>
  {/if}
  {#if !isNext}
    <div class="absolute -bottom-3 left-1/2 w-0.5 h-3 bg-zinc-700"></div>
  {/if}

  <div class="flex items-center justify-between">
    <div class="flex items-center gap-1.5">
      {#if completed}
        <MissionCompletedIcon class="size-4 lg:size-5 text-accent" />
      {:else if isNext}
        <MissionBoxIcon class="size-4 lg:size-5 text-zinc-400" />
      {:else}
        <MissionBoxIcon class="size-4 lg:size-5 text-accent" />
      {/if}
      <h3 class="text-sm lg:text-base font-medium lg:font-bold group">
        {achievement.title}
      </h3>
    </div>
    <div class="relative">
      <div class="px-1.5 py-0.5 rounded-sm bg-zinc-800/80 border border-zinc-700/50 {showXp ? 'scale-110 border-accent/50' : ''} transition-all duration-300 flex items-center justify-center">
        <span
          class="text-[10px] lg:text-xs font-medium {showXp ? 'text-accent' : 'text-white'} transition-all duration-300"
        >
          {achievement.xp} XP
        </span>
      </div>
      {#if showXp}
        <div class="absolute inset-0 rounded-sm bg-accent/20 blur-sm animate-pulse"></div>
      {/if}
    </div>
  </div>

  <div class="text-xs lg:text-sm text-neutral-300 [&_a]:text-accent [&_strong]:font-bold [&_strong]:text-white">
    {#await mdToHTML(achievement.description)}
      <p>Loading description...</p>
    {:then vFile}
      {@html vFile.value}
    {/await}
  </div>

  <div class="flex items-center gap-3">
    <img
      src={`https://images.cdn.union.build/achievement-${achievement.id}.png`}
      alt={`${achievement.title} achievement icon`}
      class="size-12 lg:size-14 object-contain"
    />
    <div class="flex-1 flex flex-col gap-1.5">
      <div class="flex justify-between text-[10px] lg:text-xs">
        <span class="text-zinc-400">Progress</span>
        <span class="text-zinc-400">
          <span class="font-medium">
            {userAchievement?.progression ?? 0}
          </span>
          / {userAchievement?.threshold ?? 0}
        </span>
      </div>
      <div class="relative">
        <ProgressBar progress={progress} />
        {#if progress === 100}
          <div class="absolute inset-0 bg-accent/20 animate-pulse rounded-full"></div>
        {/if}
      </div>
      <div class="flex justify-between text-[10px] lg:text-xs text-zinc-400">
        {#if userAchievement?.created_at}
          <span>Started: {new Date(userAchievement.created_at).toLocaleDateString()}</span>
        {/if}
        {#if completed}
          <span>Completed: {
              new Date(userAchievement?.achieved_at ?? "").toLocaleDateString()
            }</span>
        {/if}
      </div>
    </div>
  </div>

  {#if achievement.flavor}
    <p class="text-[10px] lg:text-xs italic text-neutral-400">
      {achievement.flavor}
    </p>
  {/if}

  {#if achievement.reward_achievements}
    <div class="flex flex-col gap-1.5">
      {#each achievement.reward_achievements.map((r: any) => r.rewards).filter((r: any) =>
        !r.cutoff || new Date(r.cutoff) > new Date()
      ) as
        reward
      }
        <div class="flex flex-col rounded bg-zinc-800/50 px-2 py-1 text-xs font-medium text-neutral-300">
          <div>{reward.title}</div>
          {#if reward.cutoff}
            <div class="text-[10px]">
              Available until <b>{new Date(reward.cutoff).toLocaleDateString()}</b>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</button>
