<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import LoadingBar from "$lib/components/Terminal/LoadingBar.svelte"

const { contributor, terminal } = getState()

onMount(() => {
  terminal.updateHistory("You are in queue")
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>

<Print>Your position:  {contributor.queueState.position ?? 1}</Print>
<Print>Queue length: {contributor.queueState.count ?? 2}</Print>
<Print>Estimated waiting time: {contributor.queueState.estimatedTime} minutes</Print>
<Print><br></Print>
<LoadingBar max={contributor.queueState.count} current={contributor.queueState.position}/>
<Print><br></Print>
<Print class="text-union-accent-500">Your MPC Client is connected.</Print>
<Print class="text-[#FD6363] uppercase">Do not close this tab or your Terminal.</Print>
<Print class="text-[#FD6363] uppercase">Ensure you have a reliable internet connection and that your computer does not go to sleep.</Print>

