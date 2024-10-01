<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { queryContributionWindow } from "$lib/supabase/queries.ts"
import { user } from "$lib/state/session.svelte.ts"

const { contributor } = getState()

let countdown = $state("LOADING")
let startTimestamp = $state<number>()
let expireTimestamp = $state<number>()

async function fetchTimestamps() {
  const userId = user.session?.user.id
  if (!userId) return
  const window = await queryContributionWindow(userId)
  startTimestamp = new Date(window.data?.started).getTime()
  expireTimestamp = new Date(window.data?.expire).getTime()
}

function updateCountdown() {
  if (!startTimestamp || !expireTimestamp) return
  const now = Date.now()

  let targetTime: number
  let prefix: string

  if (now < startTimestamp) {
    targetTime = startTimestamp
    prefix = ""
  } else if (now < expireTimestamp) {
    targetTime = expireTimestamp
    prefix = ""
  } else {
    countdown = "EXPIRED"
    return
  }

  const distance = targetTime - now

  const hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60))
  const seconds = Math.floor((distance % (1000 * 60)) / 1000)

  countdown = `${hours}H ${minutes}M ${seconds}S`
}

$effect(() => {
  fetchTimestamps()
})

$effect(() => {
  if (!startTimestamp || !expireTimestamp) return
  const timer = setInterval(updateCountdown, 1000)
  updateCountdown()
  return () => clearInterval(timer)
})

let show = $derived(
  contributor.contributionState === "contribute" ||
    contributor.contributionState === "verifying" ||
    contributor.queueState.position !== null
)
</script>

{#if show}
  <Print class="!text-4xl text-[#FD6363] bg-black/50 backdrop-blur-2xl py-2 px-4 hidden sm:flex">
    {countdown}
  </Print>
{/if}