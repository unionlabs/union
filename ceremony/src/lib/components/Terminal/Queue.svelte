<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { getNumberSuffix } from "$lib/utils/utils.js"
import { queryContributionWindow } from "$lib/supabase/queries.ts"

const { contributor, terminal } = getState()
let countdown = $state("LOADING")
let startTimestamp = $state<number>()

onMount(async () => {
  console.log("here")
  terminal.setStep(8)
  terminal.updateHistory({ text: "YOU ARE IN QUEUE" })
  terminal.updateHistory({ lineBreak: true, text: "" })
  terminal.updateHistory({
    text: "Do not close this tab or your Terminal. Ensure you have a reliable internet connection and that your computer does not go to sleep.",
    type: "warning"
  })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_queue" }])
  await contributor.checkUserWallet(contributor.userId)
  fetchTimestamps()
})

onDestroy(() => {
  terminal.clearHistory()
})

async function fetchTimestamps() {
  const userId = user.session?.user.id
  if (!userId) return
  const window = await queryContributionWindow(userId)
  startTimestamp = new Date(window.data?.started).getTime()
  console.log(startTimestamp)
}

function updateCountdown() {
  if (!startTimestamp) return
  const now = Date.now()

  if (now < startTimestamp) {
    const distance = startTimestamp - now

    const hours = Math.floor(distance / (1000 * 60 * 60))
    const minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60))
    const seconds = Math.floor((distance % (1000 * 60)) / 1000)

    countdown = `${hours}H ${minutes}M ${seconds}S`
  } else {
    countdown = "STARTING"
  }
}

$effect(() => {
  if (!startTimestamp) return
  const timer = setInterval(updateCountdown, 1000)
  updateCountdown()
  return () => clearInterval(timer)
})
</script>

<Print>Your place in line: <span
        class="text-union-accent-500">{contributor.queueState.position ?? "LOADING"}{getNumberSuffix(contributor.queueState.position)}</span></Print>
<Print>Estimated start time: <span class="text-union-accent-500">{countdown}</span> (may begin sooner)</Print>
<Print><br></Print>
<Print><span class="text-green-400">✓</span> MPC Client connected.</Print>
{#if contributor.userWallet && contributor.userWallet !== "SKIPPED"}
  <Print><span class="text-green-400">✓</span> Wallet registered and valid.</Print>
{/if}
<Print><span class="text-green-400">✓</span> Ready to contribute and awaiting slot.</Print>
<Print><br></Print>

