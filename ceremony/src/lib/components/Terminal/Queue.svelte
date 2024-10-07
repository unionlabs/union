<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import LoadingBar from "$lib/components/Terminal/LoadingBar.svelte"
import { getAverageTimes, type TimeResult } from "$lib/supabase"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { formatWaitTime } from "$lib/utils/utils.js"

const { contributor, terminal } = getState()

let waitingTime = $state<number>(0)
let averages = $state<TimeResult>()

onMount(async () => {
  terminal.setStep(8)
  terminal.updateHistory({ text: "You are in queue" })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_queue" }])
  averages = await getAverageTimes()
  await contributor.checkUserWallet(contributor.userId)
})

onDestroy(() => {
  terminal.clearHistory()
})

$effect(() => {
  if (averages && contributor.queueState.position) {
    waitingTime = (contributor.queueState.position * averages.totalMs) / 1000 / 60
  }
})
</script>

<Print>Your position:  {contributor.queueState.position}</Print>
<Print>Queue length: {contributor.queueState.count}</Print>
<Print>Estimated waiting time: {formatWaitTime(waitingTime)}</Print>
<Print><br></Print>
<LoadingBar max={contributor.queueState.count} current={contributor.queueState.position}/>
<Print><br></Print>
<Print><span class="text-green-400">✓</span> MPC Client connected.</Print>
{#if contributor.userWallet && contributor.userWallet !== "SKIPPED"}
  <Print><span class="text-green-400">✓</span> Wallet registered and valid.</Print>
{/if}
<Print><span class="text-green-400">✓</span> Ready to contribute and awaiting slot.</Print>
<Print><br></Print>
<Print class="!text-[#FD6363]">Do not close this tab or your Terminal. Ensure you have a reliable internet connection
  and that your computer does not go to sleep.
</Print>

