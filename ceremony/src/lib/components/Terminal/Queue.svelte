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
  terminal.setStep(6)
  terminal.updateHistory({ text: "You are in queue" })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_queue" }])
  averages = await getAverageTimes()
  console.log(averages)
})

onDestroy(() => {
  terminal.clearHistory()
})

$effect(() => {
  if (averages && contributor.queueState.count) {
    waitingTime = (contributor.queueState.count * averages.totalMs) / 1000 / 60
  }
})
</script>

<!--TODO add new time-->
<Print>Your position:  {contributor.queueState.position ?? 1}</Print>
<Print>Queue length: {contributor.queueState.count ?? 2}</Print>
<Print>Estimated waiting time: {formatWaitTime(waitingTime)}</Print>
<Print><br></Print>
<LoadingBar max={contributor.queueState.count} current={contributor.queueState.position}/>
<Print><br></Print>
<Print>Your MPC Client is connected.</Print>
<Print class="!text-[#FD6363]">Do not close this tab or your Terminal. Ensure you have a reliable internet connection
  and that your computer does not go to sleep.
</Print>

