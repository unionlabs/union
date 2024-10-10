<script lang="ts">
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { queryContributionWindow } from "$lib/supabase/queries.ts"
import { user } from "$lib/state/session.svelte.ts"

const { contributor } = getState()

let countdown = $state("LOADING")
let expireTimestamp = $state<number>()

async function fetchTimestamps() {
  const userId = user.session?.user.id
  if (!userId) return
  const window = await queryContributionWindow(userId)
  expireTimestamp = new Date(window.data?.expire).getTime()
}

function updateCountdown() {
  if (!expireTimestamp) return
  const now = Date.now()

  if (now < expireTimestamp) {
    const distance = expireTimestamp - now

    const hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
    const minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60))
    const seconds = Math.floor((distance % (1000 * 60)) / 1000)

    countdown = `${hours}H ${minutes}M ${seconds}S LEFT`
  } else {
    countdown = "EXPIRED"
  }
}

$effect(() => {
  fetchTimestamps()
})

$effect(() => {
  if (!expireTimestamp) return
  const timer = setInterval(updateCountdown, 1000)
  updateCountdown()
  return () => clearInterval(timer)
})

let show = $derived(contributor.contributionState === "contribute")
</script>

{#if show}
  <Print class="!text-4xl text-[#FD6363] bg-black/50 backdrop-blur-2xl py-2 px-4 hidden sm:flex">
    {countdown}
  </Print>
{/if}