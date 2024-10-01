<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import LoadingBar from "$lib/components/Terminal/LoadingBar.svelte"
import { getAverageTimes, type TimeResult } from "$lib/supabase"

const { contributor, terminal } = getState()

let waitingTime = $state("")
let averages = $state<TimeResult>()

onMount(async () => {
  terminal.updateHistory({ text: "You are in queue" })
  averages = await getAverageTimes()
  console.log(averages)
})

onDestroy(() => {
  terminal.clearHistory()
})

$effect(() => {
  if (averages && contributor.queueState.count) {
    waitingTime = ((contributor.queueState.count * averages.totalMs) / 1000 / 60).toFixed(0)
  }
})
</script>

<!--TODO add new time-->
<Print>Your position:  {contributor.queueState.position ?? 1}</Print>
<Print>Queue length: {contributor.queueState.count ?? 2}</Print>
<Print>Estimated waiting time: {waitingTime} minutes</Print>
<Print><br></Print>
<LoadingBar max={contributor.queueState.count} current={contributor.queueState.position}/>
<Print><br></Print>
<Print>Your MPC Client is connected.</Print>
<Print class="!text-[#FD6363]">Do not close this tab or your Terminal. Ensure you have a reliable internet connection
  and that your computer does not go to sleep.
</Print>

