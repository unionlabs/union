<script lang="ts">
import { onMount, onDestroy } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { getAverageTimes } from "$lib/supabase"

const { contributor } = getState()

let averageTimeSeconds: number
let displayTime = "LOADING"
let fetchInterval: NodeJS.Timeout | null = null
let countdownInterval: NodeJS.Timeout | null = null

async function fetchData() {
  try {
    const time = await getAverageTimes()
    const queueLength = contributor.queueState.count
    averageTimeSeconds = Math.round((time.totalMs / 1000) * (queueLength ?? 137))
    console.log("xx Average time:", formatTime(averageTimeSeconds))
  } catch (error) {
    console.error("Error fetching data:", error)
  }
}

function startCountdown() {
  if (countdownInterval) clearInterval(countdownInterval)
  countdownInterval = setInterval(() => {
    if (averageTimeSeconds > 0) {
      averageTimeSeconds--
      displayTime = formatTime(averageTimeSeconds)
    } else {
      displayTime = "00H 00M 00S"
      if (countdownInterval) clearInterval(countdownInterval)
    }
  }, 1000)
}

function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remainingSeconds = seconds % 60
  return `${hours.toString().padStart(2, "0")}H ${minutes.toString().padStart(2, "0")}M ${remainingSeconds.toString().padStart(2, "0")}S`
}

onMount(async () => {
  await fetchData()
  startCountdown()
  fetchInterval = setInterval(async () => {
    await fetchData()
    startCountdown()
  }, 5000)
})

onDestroy(() => {
  if (fetchInterval) clearInterval(fetchInterval)
  if (countdownInterval) clearInterval(countdownInterval)
})
</script>

<Print class="!text-4xl text-[#FD6363] bg-black/50 backdrop-blur-2xl py-2 px-4 hidden sm:flex">
  {#if contributor.loggedIn}
    {displayTime}
  {/if}
</Print>
