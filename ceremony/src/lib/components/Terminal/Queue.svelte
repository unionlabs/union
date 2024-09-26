<script lang="ts">
import { onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import LoadingBar from "$lib/components/Terminal/LoadingBar.svelte"

const { contributor, terminal } = getState()

onMount(() => {
  terminal.updateHistory("You are in queue")
})
</script>

<Print>Your position:  {contributor.queueState.position}</Print>
<Print>Queue length: {contributor.queueState.count}</Print>
<Print>Estimated waiting time: {contributor.queueState.estimatedTime} minutes</Print>
<LoadingBar max={contributor.queueState.count} current={contributor.queueState.position}/>
<Print>Your MPC Client is connected.</Print>
<Print>Do not close this tab or your Terminal.</Print>
<Print>Ensure you have a reliable internet connection and that your computer does not go to sleep.</Print>

